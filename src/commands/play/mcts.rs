//! Monte Carlo Tree Search Implementation

use super::*;
use rand::prelude::*;
use std::marker::*;
use std::ops::*;
use std::time::*;

pub const VALUE_WEIGHT: f64 = 0.5;
pub const EXPLORE: f64 = 0.5;
pub const ROLLOUT_REPS: usize = 1;

#[derive(PartialEq, Eq, Clone, Default)]
struct Stat {
    win: usize,
    loss: usize,
    tie: usize,
}

impl AddAssign<&Stat> for Stat {
    fn add_assign(&mut self, other: &Self) {
        self.win += other.win;
        self.tie += other.tie;
        self.loss += other.loss;
    }
}

impl Stat {
    fn invert(&self) -> Self {
        Self {
            win: self.loss,
            loss: self.win,
            tie: self.tie,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
struct Child(usize, Option<Tree>);

#[derive(Default, Clone, PartialEq, Eq)]
struct Tree {
    stat: Stat,
    children: Vec<Child>,
}

impl Tree {
    fn expanded(&self) -> bool {
        self.children.iter().all(|child| child.1.is_some())
    }
    fn visited(&self) -> bool {
        self.stat.win > 0 || self.stat.loss > 0 || self.stat.tie > 0
    }
    fn rating(&self) -> f64 {
        ((self.stat.win - self.stat.loss) as f64)
            / ((self.stat.win + self.stat.loss + self.stat.tie + 1) as f64)
    }
    fn value(&self, total_rollouts: usize) -> f64 {
        let Stat { win, loss, tie } = self.stat;
        let (win, loss, tie) = (win as f64, loss as f64, tie as f64);
        let n = win + loss + tie + 1.0;
        let x = (win - loss) / n;
        let r = ((total_rollouts as f64).ln() / n).powf(0.5);
        x + r * VALUE_WEIGHT
    }
    fn improve<G: PvpGame + Clone>(
        &mut self,
        rollouts: usize,
        rng: &mut ThreadRng,
        game: &mut G,
        player: usize,
    ) -> Stat {
        let mut result;
        if self.expanded() {
            // choose child with best value
            let mut children = self
                .children
                .iter_mut()
                .map(|Child(idx, tree)| (idx, tree.as_mut().unwrap()))
                .collect::<Vec<_>>();

            let mut best_val = f64::MIN;
            let mut child_idx = 0;

            for c in 0..children.len() {
                let val = children[c].1.value(rollouts);
                if val > best_val {
                    best_val = val;
                    child_idx = c;
                }
            }

            let (&mut play, ref mut child) = children[child_idx];
            game.make_move(play, player);
            result = child.improve::<G>(rollouts, rng, game, 1 - player);
        } else {
            if rng.gen::<f64>() < EXPLORE {
                // pick an unvisited child
                let unvisited = (0..self.children.len())
                    .filter(|&i| self.children[i].1.is_none())
                    .collect::<Vec<_>>();
                let pick = unvisited[rng.gen::<usize>() % unvisited.len()];
                let play = self.children[pick].0;

                game.make_move(play, player);

                let mut child = Self::new(game, player);

                result = roll_out(rng, game, 1 - player);
                child.stat += &result;
                self.children[pick].1 = Some(child);
                result = result.invert();
            } else {
                // improve upon this node
                result = roll_out(rng, game, player);
            }
        }
        self.stat += &result;
        result.invert()
    }

    fn new<G: PvpGame + Clone>(game: &G, player: usize) -> Self {
        let mut new = Self::default();
        for i in 0..G::reactions().len() {
            let mut m_game = game.clone();
            if m_game.make_move(i, 1 - player) != GameState::Invalid {
                new.children.push(Child(i, None));
            }
        }
        new
    }
}

pub struct TreeSearchAi<T> {
    /// Time limit in seconds for each move
    time_limit: f64,
    _phantom: PhantomData<T>,
}

impl<T> TreeSearchAi<T> {
    pub fn new(time_limit: f64) -> Self {
        Self {
            time_limit,
            _phantom: PhantomData,
        }
    }
}

fn roll_out<G: PvpGame + Clone>(rng: &mut ThreadRng, game: &G, player: usize) -> Stat {
    let mut stat = Stat::default();
    let moves = G::reactions().len();
    for _ in 0..ROLLOUT_REPS {
        let mut game = game.clone();
        let mut active_player = player;
        loop {
            match game.make_move(rng.gen::<usize>() % moves, active_player) {
                GameState::Win(p) => {
                    stat.win += (p == player) as usize;
                    stat.loss += (p != player) as usize;
                    break;
                }
                GameState::Tie => {
                    stat.tie += 1;
                    break;
                }
                GameState::Running => {
                    active_player = 1 - active_player;
                }
                GameState::Invalid => {}
            }
        }
    }
    stat
}

impl<T: PvpGame + Clone> AiPlayer<T> for TreeSearchAi<T> {
    fn make_move(&mut self, game: &T, player: usize) -> usize {
        let begin = Instant::now();
        let mut tree = Tree::new(game, player);

        let mut rng = thread_rng();
        let mut rollouts = 0;
        while begin.elapsed().as_secs_f64() < self.time_limit {
            let mut game = game.clone();
            tree.improve::<T>(rollouts, &mut rng, &mut game, player);
            rollouts += ROLLOUT_REPS;
        }

        // choose best move
        let mut moves = tree
            .children
            .iter()
            .map(|Child(_, tree)| match tree {
                None => 0.0,
                Some(t) => t.rating(),
            })
            .enumerate()
            .collect::<Vec<_>>();
        moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut game = game.clone();
        for (m, _) in moves.into_iter() {
            if game.make_move(m, player) != GameState::Invalid {
                return m;
            }
        }

        // somehow no valid move, return 0
        unreachable!()
    }
}
