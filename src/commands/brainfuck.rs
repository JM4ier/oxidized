use super::filter::*;
use crate::prelude::*;
use crate::ser::*;
use rusqlite::{params, Result};
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
#[checks(Spam)]
#[commands(brainfuck, store, load, run)]
pub struct Brainfuck;

#[command]
#[min_args(1)]
#[description = "Executes a brainfuck program. See [here](https://esolangs.org/wiki/Brainfuck) for an introduction to the language."]
#[usage = "<program> <input>"]
#[example = ",[.,]. echo"]
#[bucket("brainfuck")]
async fn brainfuck(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let program = args.single::<String>()?;
    let input = args.rest();
    make_exec(ctx, msg, &input, &[program]).await
}

async fn make_exec(
    ctx: &Context,
    msg: &Message,
    input: &str,
    programs: &[String],
) -> CommandResult {
    assert!(programs.len() > 0);

    let mut progs = Vec::new();
    for program in programs.iter() {
        let program = parse_instructions(&program)?;
        let program = ProgContext::new(program);
        progs.push(program);
    }

    let (iter, output, exit_code) =
        ProgContext::execute_piped(&mut progs, input.as_bytes(), 1.0, 1000);
    let output = String::from_utf8_lossy(&output);

    msg.ereply(ctx, |e| {
        e.title("Brainfuck Program Execution");
        e.field("Output", output, false);
        e.field("Exit Info", format!("{:?}({})", exit_code, iter), false)
    })
    .await?;

    Ok(())
}

fn parse_instructions(string: &str) -> Result<Vec<Instr>, &'static str> {
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

struct ProgContext {
    code: Vec<Instr>,
    data: Vec<u8>,
    ptr: usize,
    ip: usize,
}

#[derive(Copy, Clone)]
enum Output {
    Starved,
    Terminated,
    Timeout,
    Value(u8),
}

impl ProgContext {
    fn new(code: Vec<Instr>) -> Self {
        Self {
            code,
            data: vec![0u8; 30_000],
            ptr: 0,
            ip: 0,
        }
    }

    fn next_output(&mut self, input: &mut Option<u8>, iter: &mut usize, until: Instant) -> Output {
        while Instant::now() < until {
            *iter += 1;
            let instr = self.code[self.ip];
            match instr {
                Instr::MoveRight => self.ptr = (self.ptr + 1) % self.data.len(),
                Instr::MoveLeft => self.ptr = (self.ptr + self.data.len() - 1) % self.data.len(),
                Instr::Increment => self.data[self.ptr] = self.data[self.ptr].wrapping_add(1),
                Instr::Decrement => self.data[self.ptr] = self.data[self.ptr].wrapping_sub(1),
                Instr::Input => match input {
                    None => return Output::Starved,
                    Some(val) => {
                        self.data[self.ptr] = *val;
                        *input = None;
                    }
                },
                Instr::Output => {
                    self.ip += 1;
                    return Output::Value(self.data[self.ptr]);
                }
                Instr::JumpRight(target) => {
                    if self.data[self.ptr] == 0 {
                        self.ip = target;
                        continue;
                    }
                }
                Instr::JumpLeft(target) => {
                    if self.data[self.ptr] > 0 {
                        self.ip = target;
                        continue;
                    }
                }
                Instr::Terminate => return Output::Terminated,
            }
            self.ip += 1;
        }
        Output::Timeout
    }

    fn execute_piped(
        progs: &mut [Self],
        user_input: &[u8],
        time_limit: f64,
        char_limit: usize,
    ) -> (usize, Vec<u8>, ExitCode) {
        assert!(progs.len() > 0);

        let end = Instant::now() + Duration::from_secs_f64(time_limit);
        let mut pid = progs.len() - 1;
        let mut iter = 0;

        let mut user_input = user_input.iter().chain(Some(0).iter().cycle());
        let mut prog_input = vec![None; progs.len()];

        let mut output = Vec::new();

        loop {
            let out = progs[pid].next_output(&mut prog_input[pid], &mut iter, end);
            match out {
                Output::Starved => {
                    if pid == 0 {
                        prog_input[pid] = Some(*user_input.next().unwrap());
                    } else {
                        pid -= 1;
                    }
                }
                Output::Terminated => {
                    if pid == progs.len() - 1 {
                        return (iter, output, ExitCode::Success);
                    } else {
                        prog_input[pid + 1] = Some(0);
                        pid += 1;
                    }
                }
                Output::Timeout => {
                    return (iter, output, ExitCode::Timeout);
                }
                Output::Value(val) => {
                    if pid == progs.len() - 1 {
                        if output.len() < char_limit {
                            output.push(val);
                        }
                    } else {
                        prog_input[pid + 1] = Some(val);
                        pid += 1;
                    }
                }
            }
        }
    }
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
#[description = "Stores a brainfuck program for later or repeated usage."]
#[usage = "<name> <program>"]
#[example = "reverse >,[>,]<[.<]"]
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

fn load_program(name: &str, msg: &Message) -> CommandResult<String> {
    let author = format!("{}", msg.author.id);
    if name.chars().any(|ch| !ch.is_ascii_alphabetic()) {
        Err("Invalid program name")?;
    }

    create_table()?;
    let db = db()?;
    let program: String = db.query_row(
        "SELECT program FROM brainfuck WHERE author = ?1 AND name = ?2",
        params!(author, name),
        |row| row.get(0),
    )?;
    Ok(program)
}

#[command]
#[max_args(1)]
#[description = "Loads a stored brainfuck program and displays it to you, or displays the names of all stored programs."]
#[usage = "[<name>]"]
#[example = "reverse"]
pub async fn load(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    match args.single::<String>() {
        Ok(name) => {
            let program = load_program(&name, msg)?;
            msg.ereply(ctx, |e| {
                e.title(format!("{}.bf", name));
                e.field("\u{200b}", format!("```\n{}\n```", program), false)
            })
            .await?;
        }
        Err(_) => {
            let programs = {
                let db = db()?;
                let mut stmt = db.prepare("SELECT name FROM brainfuck WHERE author = ?1")?;
                let program_iter = stmt
                    .query_map(params!(format!("{}", msg.author.id)), |row| {
                        Ok(row.get::<_, String>(0)?)
                    })?;

                let mut programs = String::from("```\n");
                for program in program_iter {
                    programs += &program?;
                    programs += "\n";
                }
                programs + "```"
            };

            msg.ereply(ctx, |e| {
                e.title("Your stored brainfuck programs");
                e.field("\u{200b}", programs, false)
            })
            .await?;
        }
    }
    Ok(())
}

#[command]
#[min_args(1)]
#[description = "Loads a stored brainfuck program and runs it on an input given by you."]
#[usage = "<name> <input>"]
#[example = "reverse Hello, World!"]
#[bucket("brainfuck")]
pub async fn run(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let prog_names = args.single::<String>()?;
    let prog_names = prog_names.split('|');
    let mut progs = Vec::new();

    for prog in prog_names {
        progs.push(load_program(prog, msg)?);
    }

    make_exec(ctx, msg, args.rest(), &progs).await
}
