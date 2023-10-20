use crate::log;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = 
"edm - (ED iMproved) a line-oriented text editor, inspired by ed.

The input is read from STDIN.

Options:
    -h, --help      Print this help message and exit.
    -V, --version   Print version information and exit.

Report bugs to <contact@anthonyslab.org>";

// initial buffer
pub fn init() -> Vec<String> {
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                std::process::exit(0);
            },
            "-V" | "--version" => {
                println!("{}", VERSION);
                std::process::exit(0);
            },
            _ => (),
        }
    }

    let input = std::io::stdin();
    let mut out_buffer = Vec::new();

    loop {
        let mut buf = String::new();
        match input.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => out_buffer.push(buf),
            Err(e) => log!(FATAL, "{}", e),
        }
    }

    out_buffer
}
