use window_titles::{Connection, ConnectionTrait};

fn get_last_index(string: &String, target_char: &String) -> usize {
    let mut last_index = 0;

    for (index, char) in string.chars().enumerate() {
        if char.to_string() == target_char.to_string() {
            last_index = index;
        }
    }

    last_index
}

pub fn get_osu_window_title() -> Option<String> {
    for title in Connection::new().unwrap().window_titles().unwrap() {
        if title.starts_with("osu!") && title.contains(['-', ']']) {
            let title: String = title.chars().skip(8).collect();

            return Some(
                title
                    .chars()
                    .take(get_last_index(&title, &"[".to_string()) - 1)
                    .collect::<String>(),
            );
        }
    }

    None
}

pub fn separate_title_and_artist(title: &str) -> (String, String) {
    (
        title.chars().skip(title.find(" - ").unwrap() + 3).collect(),
        title.chars().take(title.find(" - ").unwrap()).collect(),
    )
}
