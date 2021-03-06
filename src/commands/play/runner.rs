use super::*;

pub struct GameRunner<T: 'static, G: PvpGame<T>> {
    game: G,
    game_name: &'static str,
    description: &'static str,
    mode: GameMode,
    players: Vec<Player<T, G>>,
    timeout: f64,
    turn: usize,
    board: Message,
    last_turn: Instant,
    guild_id: u64,
    moves: Vec<T>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum GameMode {
    Casual,
    Competitive,
}

impl GameMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Casual => "casual",
            Self::Competitive => "competitive",
        }
    }
}

enum Player<T, G: PvpGame<T>> {
    Person(UserId),
    Ai(Box<dyn AiPlayer<T, G> + Send + Sync>),
}

impl<T, G: PvpGame<T>> Player<T, G> {
    async fn id(&self, ctx: &Context) -> UserId {
        match self {
            Self::Person(id) => *id,
            Self::Ai(_) => ctx.cache.current_user().await.id,
        }
    }
}

impl<T, G: PvpGame<T>> PartialEq for Player<T, G> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Person(me), Self::Person(other)) => me == other,
            _ => false,
        }
    }
}

impl<T, G: PvpGame<T>> Player<T, G> {
    fn is_ai(&self) -> bool {
        match self {
            Self::Ai(_) => true,
            _ => false,
        }
    }
}

fn get_description(game: &'static str) -> &'static str {
    let cmds = commands();
    let cmd = cmds
        .iter()
        .filter(|c| c.options.names.contains(&game))
        .next()
        .expect("this function was called with the name of a command that doesn't exist.");
    cmd.options.desc.unwrap_or("\u{200b}")
}

impl<Input: 'static + Clone, G: PvpGame<Input> + Send + Sync> GameRunner<Input, G> {
    pub async fn new<'a>(
        ctx: &'a Context,
        prompt: &'a Message,
        game: G,
        game_name: &'static str,
        timeout: f64,
    ) -> CommandResult<Self> {
        create_tables(game_name)?;

        let guild_id = *prompt.guild_id.ok_or("no server id")?.as_u64();

        let challenger = prompt.author.id;
        let challenged = match prompt.mentions.iter().next() {
            Some(c) => c,
            None => {
                prompt
                    .err_reply(ctx, "You need to tag another person to play against!")
                    .await?;
                unreachable!();
            }
        };

        let mut mode = GameMode::Casual;

        let me = ctx.cache.current_user().await.id;
        let challenged = if challenged.id == me {
            // this is a bot game

            if let Some(ai) = G::ai() {
                Player::Ai(ai)
            } else {
                prompt
                    .err_reply(ctx, "This game doesn't support AI players.")
                    .await?;
                unreachable!();
            }
        } else {
            // against a person

            // can only play competitively against other people
            if challenger != challenged.id {
                mode = GameMode::Competitive;
            }

            let dialog_txt = format!(
                "{}, you have been invited by {} to play a {} game of {}.
                To start the game, confirm this with a reaction within ten seconds.",
                challenged.mention(),
                challenger.mention(),
                mode.as_str(),
                G::title()
            );

            if confirm_dialog(ctx, prompt, "Game Invite", &dialog_txt, &challenged).await? {
                Player::Person(challenged.id)
            } else {
                Err("no confirmation")?
            }
        };

        let board = prompt
            .ereply(ctx, |e| e.title(G::title()).description("Loading game..."))
            .await?;

        G::input().prepare(ctx, &board).await?;

        let players = vec![Player::Person(challenger), challenged];

        Ok(Self {
            game,
            game_name,
            description: get_description(game_name),
            mode,
            players,
            timeout,
            turn: 0,
            board,
            last_turn: Instant::now(),
            moves: Vec::new(),
            guild_id,
        })
    }

    /// when a move takes to long
    fn forfeit(&self) -> bool {
        self.time_left() == 0.0
    }

    /// how much time is left for the current move
    fn time_left(&self) -> f64 {
        (self.timeout - self.last_turn.elapsed().as_secs_f64()).max(0.0)
    }

    /// returns a Mention of the player with the index
    async fn mention_player(&self, ctx: &Context, idx: usize) -> Mention {
        self.players[idx].id(ctx).await.mention()
    }

    /// updates the game field
    async fn draw(&mut self, ctx: &Context) -> CommandResult {
        let mentions = vec![
            self.mention_player(ctx, 0).await,
            self.mention_player(ctx, 1).await,
        ];

        let countdown = "<a:_:808040888235589772> <a:_:808040929515667526>";

        let status = match self.game.status() {
            _ if self.forfeit() => format!(
                "{} won by inactivity of {}.",
                mentions[1 - self.turn],
                mentions[self.turn],
            ),
            GameState::Win(p) => format!("{} won!", mentions[p]),
            GameState::Tie => String::from("It's a tie!"),
            _ => format!(
                "{}({}) plays next.\nTime left: {} seconds. (updated every once in a while)",
                mentions[self.turn].mention(),
                G::figures()[self.turn],
                self.time_left() as u64
            ),
        };

        let board = self.game.draw();
        let desc = self.description;

        self.board
            .eedit(ctx, |e| {
                e.title(G::title());
                e.description(desc);
                e.field("Board", board, false);
                e.field("Status", status, false)
            })
            .await?;
        Ok(())
    }

    /// runs the game
    pub async fn run(&mut self, ctx: &Context) -> CommandResult {
        'game: loop {
            self.last_turn = Instant::now();
            let play = loop {
                self.draw(ctx).await?;

                if self.forfeit() {
                    break 'game;
                }

                let timeout = Duration::from_secs_f64(self.time_left().min(10.0));
                let play = match &mut self.players[self.turn] {
                    Player::Person(id) => tryc!(G::input()
                        .receive_input(ctx, &self.board, id, timeout)
                        .await
                        .ok()),
                    Player::Ai(ai) => ai.make_move(&self.game, self.turn),
                };

                let state = self.game.make_move(play.clone(), self.turn);
                if state != GameState::Invalid {
                    self.moves.push(play);
                    break state;
                }

                if self.players[self.turn].is_ai() {
                    // AI is the one that made the invalid move
                    let err_msg = format!(
                        "The AI for this game sucks and tries to do invalid moves, {} pls fix.",
                        DISCORD_AUTHOR
                    );
                    self.board.err_reply(ctx, &err_msg).await?;
                }
            };

            if play.is_finished() {
                break;
            } else {
                self.turn = 1 - self.turn;
            }
        }

        self.draw(ctx).await?;

        if self.mode == GameMode::Competitive {
            let winner = match self.game.status() {
                GameState::Win(p) => Some(p),
                _ if self.forfeit() => Some(1 - self.turn),
                _ => None,
            };

            let mut players = Vec::new();
            for p in self.players.iter() {
                players.push(*p.id(ctx).await.as_u64());
            }

            // TODO
            // log_game(self.game_name, server, &players, &self.moves, winner)?;
            elo::process_game(self.game_name, self.guild_id, &players, winner)?;
        }

        Ok(())
    }
}
