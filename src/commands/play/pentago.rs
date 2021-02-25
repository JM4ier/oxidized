use super::*;

#[derive(Default, PartialEq, Eq)]
pub struct Pentago {
    field: [[Option<usize>; 6]; 6],
}

#[derive(Clone)]
pub struct PMove {
    x: usize,
    y: usize,
    sel: usize,
    dir: usize,
}

const ANTI_CLOCKWISE: usize = 1;
const CLOCKWISE: usize = 3;

impl PvpGame<PMove> for Pentago {
    fn title() -> &'static str {
        "Pentago"
    }
    fn input() -> Box<dyn InputMethod<PMove> + Send + Sync> {
        Box::new(TextInput(Box::new(|text| {
            let text = text.to_lowercase().chars().collect::<Vec<char>>();

            if text.len() < 4 {
                Err("text too short.")?;
            }

            let digit = |i: usize| -> CommandResult<usize> {
                Ok(text[i].to_digit(10).ok_or("no digit")? as usize - 1)
            };

            let x = digit(0)?;
            let y = digit(1)?;
            let s = digit(2)?;

            let dir = if text[3] == 'a' {
                ANTI_CLOCKWISE
            } else {
                CLOCKWISE
            };

            Ok(PMove { x, y, sel: s, dir })
        })))
    }

    fn draw(&self) -> String {
        let mut drawing = String::new();

        let black = square(util::Color::Black);
        let blue = square(util::Color::Blue);

        let mut hor_border = String::from(blue);
        for x in 1..=6 {
            hor_border += &NUMBERS[x];
            hor_border += blue;
        }
        hor_border += "\n";

        drawing += &hor_border;

        for y in 0..6 {
            drawing += &NUMBERS[y + 1];

            for x in 0..6 {
                match self.field[x][y] {
                    None => drawing += black,
                    Some(p) => drawing += &Self::figures()[p],
                }
                if x == 2 {
                    drawing += blue;
                } else if x != 5 {
                    drawing += black;
                }
            }

            drawing += &NUMBERS[y + 1];
            drawing += "\n";
            if y != 5 {
                drawing += blue;
                for x in 0..11 {
                    drawing += if y == 2 || x == 5 { blue } else { black };
                }
                drawing += blue;
                drawing += "\n";
            }
        }

        drawing += &hor_border;

        drawing += "\n\nSubfield layout:\n";
        drawing += &NUMBERS[1];
        drawing += &NUMBERS[2];
        drawing += "\n";
        drawing += &NUMBERS[3];
        drawing += &NUMBERS[4];

        drawing
    }

    fn make_move(&mut self, mov: PMove, person: usize) -> GameState {
        if mov.sel >= 4 || mov.x >= 6 || mov.y >= 6 || self.field[mov.x][mov.y].is_some() {
            return GameState::Invalid;
        }

        self.field[mov.x][mov.y] = Some(person);

        if let GameState::Win(_) = self.status() {
            return self.status();
        }

        // permutation table
        let mut perm = [
            [(1, 0), (0, 1), (1, 2), (2, 1)],
            [(0, 0), (0, 2), (2, 2), (2, 0)],
        ];

        // offsets of the sub-fields
        let offset = [(0, 0), (3, 0), (0, 3), (3, 3)];

        for perm in perm.iter_mut() {
            for perm in perm.iter_mut() {
                let off = offset[mov.sel];
                *perm = (perm.0 + off.0, perm.1 + off.1);
            }
        }

        for _ in 0..mov.dir {
            for perm in perm.iter() {
                let mut last = self.field[perm[3].0][perm[3].1];
                for (x, y) in perm.iter() {
                    let new_last = self.field[*x][*y];
                    self.field[*x][*y] = last;
                    last = new_last;
                }
            }
        }

        self.status()
    }

    fn status(&self) -> GameState {
        util::n_in_a_row(6, 6, &|x, y| self.field[x][y], 5)
    }

    fn figures() -> Vec<String> {
        vec![
            circle(util::Color::Yellow).into(),
            circle(util::Color::Red).into(),
        ]
    }

    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}
