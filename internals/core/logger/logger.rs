use colored::{ColoredString, Colorize};

pub struct Logger {}

impl Logger {
    pub fn info(message: &str) {
        println!("{}{}", Logger::info_prefix(), message);
    }

    pub fn error(message: &str) {
        eprintln!("{}{}", Logger::error_prefix(), message);
    }

    pub fn exit_with_info(message: &str) {
        Logger::info(message);
        panic!("{}{}", Logger::info_prefix(), message);
    }

    pub fn exit_with_error(message: &str) {
        Logger::error(message);
        panic!("{}{}", Logger::error_prefix(), message);
    }

    pub fn green(message: &str) -> ColoredString {
        message.bright_green()
    }

    fn info_prefix() -> ColoredString {
        "Compressor: ".bright_green().bold()
    }

    fn error_prefix() -> ColoredString {
        "Compressor: ".red().bold()
    }
}
