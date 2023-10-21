use crate::log;

type RegexMatch = String;

pub enum Range {
    Single(usize),         // 6
    Bounded(usize, usize), // 6..9
    Start(usize),          // 6..
    End(usize),            // ..9
    None,
}

pub enum Command {
    Write(Option<String>),
    WriteQuit(Option<String>),
    Quit,
    ForceQuit,

    Insert,
    Delete(Range),
    Change(Range, /* Option<RegexMatch>, */ Option<String>),

    Print,
    Line,

}

pub fn parse_command(input: &str) -> Option<Command> {
    let Ok((range, rest)) = parse_range(input) else {
        return None;
    };

    let args = rest.split_whitespace().collect::<Vec<_>>();

    match args[0] {
        "w" => match args.len() {
            1 => return Some(Command::Write(None)),
            2 => return Some(Command::Write(Some(args[1].to_string()))),
            _ => (),
        },

        "wq" => match args.len() {
            1 => return Some(Command::WriteQuit(None)),
            2 => return Some(Command::WriteQuit(Some(args[1].to_string()))),
            _ => (),
        },

        "c" => {
            if args.len() == 1 {
                return Some(Command::Change(range, None));
            }

            return Some(Command::Change(range, Some(String::from(&rest[1..]))));
        },

        _ => (),
    }

    if args.len() > 1 {
        log!(ERROR, "Too many arguments");
        return None;
    }

    match args[0] {
        "q" => Some(Command::Quit),

        "q!" => Some(Command::ForceQuit),

        "i" => Some(Command::Insert),

        "p" => Some(Command::Print),

        "d" => Some(Command::Delete(range)),

        "l" => Some(Command::Line),

        _ => {
            log!(ERROR, "Unknown command"); None
        },
    }
}

fn parse_range(input: &str) -> Result<(Range, &str), ()> {
    let (range, rest) = split_string(input);

    let Some(range) = range else {
        return Ok((Range::None, rest));
    };

    let Some((start, end)) = range.split_once(',') else {
        let Ok(range) = range.parse::<usize>() else {
            log!(ERROR, "Invalid range");
            return Err(());
        };
        return Ok((Range::Single(range), rest));
    };

    match start {
        "" if end.is_empty() => {
            log!(ERROR, "Invalid range {}", range); Err(())
        },

        "" => {
            let Ok(end) = end.parse::<usize>() else {
                log!(ERROR, "Invalid range");
                return Err(());
            }; Ok((Range::End(end), rest))
        },
        
        _ if end.is_empty() => {
            let Ok(start) = start.parse::<usize>() else {
                log!(ERROR, "Invalid range");
                return Err(());
            };

            Ok((Range::Start(start), rest))
        },

        _ => {
            let Ok(start) = start.parse::<usize>() else {
                log!(ERROR, "Invalid range");
                return Err(());
            };

            let Ok(end) = end.parse::<usize>() else {
                log!(ERROR, "Invalid range");
                return Err(());
            }; Ok((Range::Bounded(start, end), rest))
        },
    }
}

fn split_string(input_string: &str) -> (Option<&str>, &str) {
    if input_string.is_empty() {
        return (None, "");
    } 

    if input_string.chars().next().unwrap().is_alphabetic() {
        return (None, input_string);
    } 

    let index = input_string.chars()
        .position(|c| c.is_alphabetic())
        .unwrap_or(input_string.len());

    let (part1, part2) = input_string.split_at(index);
    (Some(part1), part2)
}
