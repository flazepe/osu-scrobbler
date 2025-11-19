use colored::{Color, Colorize};
use std::{fmt::Display, fs::OpenOptions, io::Write};

pub struct Logger;

impl Logger {
    pub fn log<T: Display, U: Into<Color>>(tag: &str, tag_color: U, message: T) {
        let tag = if tag.starts_with('\t') { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") };
        let colored_tag = tag.color(tag_color);
        println!("{colored_tag} {message}");
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
