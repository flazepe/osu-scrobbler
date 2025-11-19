use colored::Colorize;
use std::{fmt::Display, fs::OpenOptions, io::Write};

pub fn log_error<T: Display>(tag: &str, message: T) {
    println!("{} {message}", if tag.starts_with('\t') { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") }.bright_red());
}

pub fn log_success<T: Display>(tag: &str, message: T) {
    println!("{} {message}", if tag.starts_with('\t') { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") }.bright_green());
}

pub fn log_warn<T: Display>(tag: &str, message: T) {
    println!("{} {message}", if tag.starts_with('\t') { format!("\t[{}]", tag.trim()) } else { format!("[{tag}]") }.bright_yellow());
}

pub fn log_file<T: Display>(message: T) {
    let mut options = OpenOptions::new();
    options.create(true).write(true).append(true);

    let Ok(mut file) = options.open("scrobble.log") else { return };
    file.write_all(format!("{message}\n").as_bytes()).ok();
}
