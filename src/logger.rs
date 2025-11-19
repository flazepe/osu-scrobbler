use colored::{Color, Colorize};
use std::{fmt::Display, fs::OpenOptions, io::Write};

pub fn log<T: Display, U: Into<Color>>(tag: &str, tag_color: U, message: T) {
    let tag = if tag.starts_with('\t') { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") };
    let colored_tag = tag.color(tag_color);
    println!("{colored_tag} {message}");
}

pub fn log_error<T: Display>(tag: &str, message: T) {
    log(tag, Color::BrightRed, message);
}

pub fn log_success<T: Display>(tag: &str, message: T) {
    log(tag, Color::BrightGreen, message);
}

pub fn log_warn<T: Display>(tag: &str, message: T) {
    log(tag, Color::BrightYellow, message);
}

pub fn log_file<T: Display>(message: T) {
    let mut options = OpenOptions::new();
    options.create(true).write(true).append(true);

    let Ok(mut file) = options.open("scrobble.log") else { return };
    file.write_all(format!("{message}\n").as_bytes()).ok();
}
