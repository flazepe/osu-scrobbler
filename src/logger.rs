use chrono::Local;
use colored::{Color, Colorize};
use std::{fmt::Display, fs::OpenOptions, io::Write};

pub struct Logger;

impl Logger {
    pub fn log<T: Into<Color>, U: Display>(tag: &str, tag_color: T, message: U, is_sub: bool) {
        let tag = if is_sub { format!("\t[{tag}]") } else { format!("[{tag}]") };
        println!("{} {} {message}", Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false).bright_black(), tag.color(tag_color));
    }

    pub fn error<T: Display>(tag: &str, message: T, is_sub: bool) {
        Self::log(tag, Color::BrightRed, message, is_sub);
    }

    pub fn success<T: Display>(tag: &str, message: T, is_sub: bool) {
        Self::log(tag, Color::BrightGreen, message, is_sub);
    }

    pub fn warn<T: Display>(tag: &str, message: T, is_sub: bool) {
        Self::log(tag, Color::BrightYellow, message, is_sub);
    }

    pub fn file<T: Display>(message: T) {
        let mut options = OpenOptions::new();
        options.create(true).write(true).append(true);

        let Ok(mut file) = options.open("scrobble.log") else { return };
        file.write_all(format!("{message}\n").as_bytes()).ok();
    }
}
