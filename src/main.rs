use std::io::{stdout, Write};
use std::process::exit;
use std::fs::File;
use std::io::{BufReader, BufRead};

mod logger;
mod init;
mod print;
mod parser;

fn main() {
    let mut main_buffer = init::init();

    print::lines_enumerated(&main_buffer);

    let mut is_saved = false;
    let mut filename: Option<String> = None;
    let mut current_line = main_buffer.len();

    #[cfg(unix)]
    let mut tty = BufReader::new(File::open("/dev/tty").unwrap());

    #[cfg(windows)]
    let mut tty = BufReader::new(File::open("\\\\.\\CONIN$").unwrap());

    loop {
        print!(":");
        stdout().flush().unwrap();

        let input = get_line_from(&mut tty);
        let Some(cmd) = parser::parse_command(&input) else {
            continue;
        };

        use parser::{Range, Command::*};
        match cmd {
            Print => print::lines_enumerated(&main_buffer),
            Line => println!("{}", current_line),

            Quit => {
                if is_saved { exit(0); }
                log!(WARN, "No write since last change");
                is_saved = true;
            },

            ForceQuit => exit(0),

            Write(None) => if let Some(ref file) = filename {
                write_file(file, &main_buffer);
                is_saved = true;
            } else { 
                log!(ERROR, "No file name"); 
            },

            Write(Some(file)) => {
                write_file(&file, &main_buffer);
                filename = Some(file);
                is_saved = true;
            },

            WriteQuit(None) => if let Some(file) = filename {
                write_file(&file, &main_buffer);
                exit(0);
            } else { 
                log!(ERROR, "No file name"); 
            },

            WriteQuit(Some(file)) => {
                write_file(&file, &main_buffer);
                exit(0);
            },

            Insert => {
                let lines = get_from_prompt(&mut tty);
                let len = lines.len();
                main_buffer.splice(current_line-1..current_line, lines);
                current_line += len;
                is_saved = false;
            },

            Delete(range) => {
                let range = match range {
                    Range::Bounded(start, end) => start-1..end,
                    Range::Start(start) => start..main_buffer.len(),
                    Range::End(end) => 0..end,
                    Range::Single(line) => line-1..line,
                    Range::None => current_line-1..current_line,
                };

                current_line -= normalize_curr_line(&range, current_line);

                main_buffer.splice(range, Vec::new());
                is_saved = false;
            },

            Change(range, change) => {
                let range = match range {
                    Range::Bounded(start, end) => start-1..end,
                    Range::Start(start) => start..main_buffer.len(),
                    Range::End(end) => 0..end,
                    Range::Single(line) => line-1..line,
                    Range::None => current_line-1..current_line,
                };
                
                current_line -= normalize_curr_line(&range, current_line);

                let lines = match change {
                    Some(change) => vec![change],
                    None => get_from_prompt(&mut tty),
                };

                current_line += lines.len();
                main_buffer.splice(range, lines);
                is_saved = false;
            }
        }
    }
}

fn get_from_prompt(tty: &mut BufReader<File>) -> Vec<String> {
    let mut buffer = Vec::new();
    loop {
        let line = get_line_from(tty);
        if line == ".\n" { break; }
        buffer.push(line);
    } buffer
}

fn normalize_curr_line(range: &std::ops::Range<usize>, current: usize) -> usize {
    if range.start < current {
        return range.end - range.start;
    } 0
}

fn write_file(filename: &str, buffer: &[String]) {
    let mut file = std::fs::OpenOptions::new()
        .write(true).create(true).open(filename).unwrap();

    log!("{:?} {}l {}b", filename, buffer.len(), buffer.join("").len());

    file.write_all(buffer.join("").as_bytes()).unwrap();
}

fn get_line_from(input: &mut BufReader<File>) -> String {
    let mut buffer = String::new();
    input.read_line(&mut buffer).expect("Failed to read line");
    buffer
}

