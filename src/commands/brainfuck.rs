use crate::prelude::*;
use rusqlite::{params, Result};
use serenity::{
    framework::standard::{macros::*, *},
    model::prelude::*,
    prelude::*,
};
use std::collections::*;
use std::time::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ExitCode {
    Success,
    Timeout,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Instr {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Output,
    Input,
    JumpRight(usize),
    JumpLeft(usize),
    Terminate,
}

#[group]
#[commands(brainfuck, store, load)]
pub struct Brainfuck;

#[command]
async fn brainfuck(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let program = args.single::<String>()?;
    let program = make_program(&program)?;
    let input = args.rest().as_bytes();

    let (output, exit_code) = execute(&program, input, 1.0, 1000);
    let output = String::from_utf8_lossy(&output);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Brainfuck Program Execution");
                e.field("Output", output, false);
                e.field("Exit Code", format!("{:?}", exit_code), false)
            })
        })
        .await?;

    Ok(())
}

fn make_program(string: &str) -> Result<Vec<Instr>, &'static str> {
    let mut program = Vec::new();
    let mut stack = VecDeque::new();
    for ch in string.chars() {
        let idx = program.len();
        let instr = match ch {
            '>' => Instr::MoveRight,
            '<' => Instr::MoveLeft,
            '+' => Instr::Increment,
            '-' => Instr::Decrement,
            '.' => Instr::Output,
            ',' => Instr::Input,
            '[' => {
                stack.push_back(idx);
                Instr::JumpRight(0)
            }
            ']' => {
                let target = match stack.pop_back() {
                    Some(t) => t,
                    None => return Err("mismatched brackets"),
                };
                match program[target] {
                    Instr::JumpRight(ref mut target) => *target = idx + 1,
                    _ => unreachable!(),
                };
                Instr::JumpLeft(target + 1)
            }
            _ => continue,
        };
        program.push(instr);
    }
    if !stack.is_empty() {
        Err("mismatched brackets")
    } else {
        program.push(Instr::Terminate);
        Ok(program)
    }
}

fn execute(
    code: &[Instr],
    mut input: &[u8],
    time_limit: f64,
    char_limit: usize,
) -> (Vec<u8>, ExitCode) {
    let mut output = "\u{200b}".as_bytes().to_owned();
    let mut ptr = 0usize;
    let mut data = vec![0u8; 30_000];
    let mut instr_ptr = 0;

    let begin = Instant::now();

    while begin.elapsed().as_secs_f64() < time_limit {
        let instr = code[instr_ptr];
        match instr {
            Instr::MoveRight => ptr = (ptr + 1) % data.len(),
            Instr::MoveLeft => ptr = (ptr + data.len() - 1) % data.len(),
            Instr::Increment => data[ptr] = data[ptr].wrapping_add(1),
            Instr::Decrement => data[ptr] = data[ptr].wrapping_sub(1),
            Instr::Input => {
                if input.len() > 0 {
                    data[ptr] = input[0];
                    input = &input[1..];
                } else {
                    data[ptr] = 0;
                }
            }
            Instr::Output => {
                if output.len() < char_limit {
                    output.push(data[ptr])
                }
            }
            Instr::JumpRight(target) => {
                if data[ptr] == 0 {
                    instr_ptr = target;
                    continue;
                }
            }
            Instr::JumpLeft(target) => {
                if data[ptr] > 0 {
                    instr_ptr = target;
                    continue;
                }
            }
            Instr::Terminate => return (output, ExitCode::Success),
        }
        instr_ptr += 1;
    }
    (output, ExitCode::Timeout)
}

fn create_table() -> Result<()> {
    db()?.execute(
        "CREATE TABLE IF NOT EXISTS brainfuck (author TEXT, name TEXT, program TEXT);",
        params!(),
    )?;
    Ok(())
}

#[command]
#[min_args(2)]
#[max_args(2)]
pub async fn store(_: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let name = args.single::<String>()?;
    let program = args.single::<String>()?;

    if name.as_str().chars().any(|ch| !ch.is_ascii_alphabetic()) {
        Err("Invalid program name")?;
    }
    if program.as_str().chars().any(|ch| !",.<>+-[]".contains(ch)) {
        Err("Invalid program")?;
    }

    let author = format!("{}", msg.author.id);

    create_table()?;
    let db = db()?;

    let affected = db.execute(
        "UPDATE brainfuck SET program = ?3 WHERE author=?1 AND name = ?2;",
        params!(author, name, program),
    )?;

    if affected == 0 {
        db.execute(
            "INSERT INTO brainfuck (author, name, program) VALUES (?1, ?2, ?3)",
            params!(author, name, program),
        )?;
    }

    Ok(())
}

#[command]
#[min_args(1)]
#[max_args(1)]
pub async fn load(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let name = args.single::<String>()?;
    let author = format!("{}", msg.author.id);
    if name.as_str().chars().any(|ch| !ch.is_ascii_alphabetic()) {
        Err("Invalid program name")?;
    }

    create_table()?;
    let db = db()?;
    let program: String = db.query_row(
        "SELECT program FROM brainfuck WHERE author = ?1 AND name = ?2",
        params!(author, name),
        |row| row.get(0),
    )?;

    msg.reply(ctx, format!("`{}`", program)).await?;

    Ok(())
}
