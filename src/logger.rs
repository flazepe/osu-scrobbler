use chrono::Local;
use colored::{Color, Colorize};
use std::{fmt::Display, fs::OpenOptions, io::Write};

pub struct Logger;

impl Logger {
    pub fn log<T: Display, U: Into<Color>>(tag: &str, tag_color: U, message: T) {
        let is_sub = tag.starts_with('\t');
        let tag = if is_sub { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") };
        println!("{} {} {message}", Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false).bright_black(), tag.color(tag_color));
    }

    pub fn error<T: Display>(tag: &str, message: T) {
        Self::log(tag, Color::BrightRed, message);
    }

    pub fn success<T: Display>(tag: &str, message: T) {
        Self::log(tag, Color::BrightGreen, message);
    }

    pub fn warn<T: Display>(tag: &str, message: T) {
        Self::log(tag, Color::BrightYellow, message);
    }

    pub fn file<T: Display>(message: T) {
        let mut options = OpenOptions::new();
        options.create(true).write(true).append(true);

        let Ok(mut file) = options.open("scrobble.log") else { return };
        file.write_all(format!("{message}\n").as_bytes()).ok();
    }
}
