use std::io::{stdin, stdout, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";
const HELP: &str = 
"edm - (ED iMproved) a line-oriented text editor, inspired by ed.

Usage: edm [FILE]

Options:
    -h, --help      Print this help message and exit.
    -V, --version   Print version information and exit.

If it begins with a '!', raed output of a shell command.

Report bugs to <contact@anthonyslab.org>";

fn main() {
    let mut filename = None;
    let mut main_buffer = match std::env::args().nth(1) {
        None => {
            println!("{}", HELP);
            exit(0);
        },
        Some(arg) => match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                exit(0);
            },
            "-V" | "--version" => {
                println!("{}", VERSION);
                exit(0);
            },

            cmd if cmd.starts_with('!') => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd[1..])
                    .output()
                    .expect("failed to execute process");
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(String::from)
                    .collect::<Vec<String>>()

            },

            file => {
                filename = Some(file.to_string());
                std::fs::read_to_string(file).unwrap_or_else(|why|{
                    match why.kind() {
                        std::io::ErrorKind::NotFound => String::new(),
                        _ => {
                            eprintln!("{}: {}", file, why);
                            exit(1);
                        },
                    }
                })
                .lines()
                .map(String::from)
                .collect::<Vec<String>>()
            },
        },
    };
    
    let mut current_line = main_buffer.len();
    let mut is_saved = true;
    let mut command_stack = Vec::new(); // TODO: implement command history

    loop {
        print!(":");
        stdout().flush().expect("Failed to flush stdout");

        let input = stdin_get_line();
        command_stack.push(input.clone());
        let cmd = match parse_command(&input, main_buffer.len()) {
            Ok(cmd) => cmd,
            Err(why) => {
                eprintln!("{}", why);
                continue;
            },
        };
        
        use crate::CommandKind::*;
        match cmd.command {
            Quit => {
                if !is_saved {
                    eprintln!("Changes Not Written");
                    continue;
                } exit(0)
            },
            QuitForce => exit(0),
            Write => {
                let file = if let Some(ref file) = cmd.args { file }
                else if let Some(ref file) = filename { file }
                else {
                    eprintln!("No filename given");
                    continue;
                };

                std::fs::write(file, main_buffer.join("\n")).unwrap_or_else(|why|{
                    eprintln!("{}: {}", file, why);
                    exit(1);
                });

                is_saved = true;
            },
            WriteQuit => {
                let file = if let Some(ref file) = cmd.args { file }
                else if let Some(ref file) = filename { file }
                else {
                    eprintln!("No filename given");
                    continue;
                };

                std::fs::write(file, main_buffer.join("\n")).unwrap_or_else(|why|{
                    eprintln!("{}: {}", file, why);
                    exit(1);
                }); exit(0);
            },

            Append => {
                let mut buffer = Vec::new();
                loop {
                    let line = stdin_get_line();

                    if line == "." { break; }

                    buffer.push(line);
                }

                main_buffer.splice(current_line..current_line, buffer.clone());
                current_line += buffer.len();
                is_saved = false;
            },

            Insert => {
                let mut buffer = Vec::new();
                loop {
                    let line = stdin_get_line();

                    if line == "." { break; }

                    buffer.push(line);
                }

                main_buffer.splice(current_line-1..current_line-1, buffer.clone());
                current_line += buffer.len();
                is_saved = false;
            },


            Command => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd.args.unwrap())
                    .output()
                    .expect("failed to execute process");
                print!("{}", String::from_utf8_lossy(&output.stdout));
            },

            Print => println!("{}", main_buffer.join("\n")),
            NumberPrint => {
                for (i, line) in main_buffer.iter().enumerate() {
                    println!("{}\t{}", i + 1, line);
                }
            },
            _ => todo!(),
        }

    }
}

fn stdin_get_line() -> String {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("Failed to read line");
    buffer[..buffer.len() - 1].to_string()
}

fn write_file(filename: &str, buffer: &[String]) {
    std::fs::write(filename, buffer.join("\n")).unwrap_or_else(|why|{
        eprintln!("{}: {}", filename, why);
        exit(1);
    });
    println!("{}ln; {}b", buffer.len(), buffer.iter().map(|s| s.len()).sum::<usize>());
}

struct Command {
    range: Option<(usize, usize)>,
    command: CommandKind,
    args: Option<String>,
}

enum CommandKind {
    Quit,
    QuitForce,
    Write,
    WriteQuit,

    Append,
    Insert,
    Read,

    Change,
    Delete,

    Move,

    Yank,
    Put,
    
    Undo,
    Redo,

    Substitute,
    Global,

    Command,

    Print,
    NumberPrint,
}

fn parse_command(mut input: &str, len: usize) -> Result<Command, &'static str> {
    let mut range = None;

    if input.starts_with(',') {
        range = Some((1, len));
        input = &input[1..];
    }

    else if input.starts_with(|c: char| c.is_ascii_digit()) {
        let mut span = input.splitn(2, ',');

        let Ok(start) = span.next().unwrap().parse::<usize>() else {
            return Err("Invalid address");
        };

        let end = match span.next() {
            None => len,
            Some(end) => end.parse::<usize>().unwrap(),
        };

        range = Some((start, end));
        input = &input[1..];
    }

    if input.is_empty() {
        return Err("No command given");
    }

    let mut input = input.chars().peekable();

    let command = match input.next().unwrap() {
        'q' if input.peek() == Some(&'!') => {
            input.next();
            CommandKind::QuitForce
        },
        'q' => CommandKind::Quit,

        'w' if input.peek() == Some(&'q') => {
            input.next();
            CommandKind::WriteQuit
        },
        'w' => CommandKind::Write,

        'a' => CommandKind::Append,
        'i' => CommandKind::Insert,
        'r' => CommandKind::Read,

        'c' => CommandKind::Change,
        'd' => CommandKind::Delete,

        'm' => CommandKind::Move,

        'y' => CommandKind::Yank,
        'P' => CommandKind::Put,

        's' => CommandKind::Substitute,
        'g' => CommandKind::Global,

        '!' => CommandKind::Command,

        'p' => CommandKind::Print,
        'n' => CommandKind::NumberPrint,
        _ => return Err("Invalid command"),
    };

    let args = {
        let args = input.collect::<String>();

        if !args.is_empty() {
            Some(args.trim().to_string())
        } else { None }
    };

    Ok(Command { range, command, args })
}
