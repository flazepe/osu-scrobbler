use window_titles::{Connection, ConnectionTrait};

pub fn get_window_title() -> Option<String> {
    for title in Connection::new().unwrap().window_titles().unwrap() {
        if title.starts_with("osu!") && title.contains(['-', ']']) {
            return Some(title.chars().skip(8).collect());
        }
    }

    None
}
