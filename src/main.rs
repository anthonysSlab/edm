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

        use parser::Command::*;
        match cmd {
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
                let mut buffer = Vec::new();
                loop {
                    let line = get_line_from(&mut tty);
                    if line == ".\n" { break; }
                    buffer.push(line);
                }

                let len = buffer.len();
                main_buffer.splice(current_line-1..current_line, buffer);
                current_line += len;
                is_saved = false;
            },

            Print => print::lines_enumerated(&main_buffer),

            Delete(Some((start, end))) => {
                is_saved = false;
                current_line = start-1;
                if let Some(end) = end {
                    if start == end {
                        main_buffer.remove(start-1);
                        continue;
                    }
                    main_buffer.splice(start-1..end, Vec::new());
                    continue;
                } 

                main_buffer.splice(start-1.., Vec::new());
            },

            Delete(None) => {
                main_buffer.remove(current_line-1);
                current_line -= 1;
                is_saved = false;
            },

            Line => {
                println!("{}", current_line);
            },
        }
    }
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

