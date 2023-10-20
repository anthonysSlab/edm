pub enum Level {
    Info,
    Warn,
    Error,
    Fatal,
}

#[macro_export]
macro_rules! log {
    (WARN, $($arg:tt)*) => {
        $crate::logger::logger($crate::logger::Level::Warn, format!($($arg)*))
    };
    (ERROR, $($arg:tt)*) => {
        $crate::logger::logger($crate::logger::Level::Error, format!($($arg)*))
    };
    (FATAL, $($arg:tt)*) => {
        $crate::logger::log_exit(format!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::logger::logger($crate::logger::Level::Info, format!($($arg)*))
    };
}

use std::fmt::Display;
pub fn logger<T: Display>(level: Level, message: T) {
    use Level::*;
    match level {
        Info  => println!("{}", message),
        Warn  => println!("\x1b[33;1mW: {}\x1b[0m", message),
        Error => println!("\x1b[31;1mE: {}\x1b[0m", message),
        Fatal => println!("\x1b[91;1mFATAL: {}\x1b[0m", message),
    }
}

pub fn log_exit<T: Display>(message: T) -> ! {
    logger(Level::Fatal, message);
    std::process::exit(1);
}
