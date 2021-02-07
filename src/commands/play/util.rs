use super::GameState;
use crate::cart;
use crate::ser::*;

pub fn n_in_a_row(
    rows: usize,
    cols: usize,
    index: &dyn Fn(usize, usize) -> Option<usize>,
    n: usize,
) -> GameState {
    let n = n as isize;
    let mut filled = true;

    for (x, y) in cart!(0..cols, 0..rows) {
        filled &= index(x, y).is_some();

        let color = match index(x, y) {
            Some(color) => color,
            None => continue,
        };

        'dir: for (dx, dy) in cart!(-1..=1, -1..=1) {
            if dx == 0 && dy == 0 {
                continue;
            }
            for i in 1..n {
                let x = i * dx + x as isize;
                let y = i * dy + y as isize;
                if x < 0 || y < 0 || x >= cols as _ || y >= rows as _ {
                    continue 'dir;
                }
                if index(x as _, y as _) != Some(color) {
                    continue 'dir;
                }
            }
            return GameState::Win(color);
        }
    }
    if filled {
        GameState::Tie
    } else {
        GameState::Running
    }
}

// common unicode stuff to display game symbols

lazy_static! {
    pub static ref NUMBERS: Vec<String> = (0..10)
        .map(|num| format!("{}\u{fe0f}\u{20e3}", num))
        .collect();
}

pub fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(NUMBERS[num].clone())
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Brown,
}

pub fn square(color: Color) -> &'static str {
    match color {
        Color::White => "⬜",
        Color::Black => "⬛",
        Color::Red => "🟥",
        Color::Orange => "🟧",
        Color::Yellow => "🟨",
        Color::Green => "🟩",
        Color::Blue => "🟦",
        Color::Purple => "🟪",
        Color::Brown => "🟫",
    }
}

pub fn circle(color: Color) -> &'static str {
    match color {
        Color::White => "⚪️",
        Color::Black => "⚫️",
        Color::Red => "🔴",
        Color::Orange => "🟠",
        Color::Yellow => "🟡",
        Color::Green => "🟢",
        Color::Blue => "🔵",
        Color::Purple => "🟣",
        Color::Brown => "🟤",
    }
}
