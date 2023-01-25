use window_titles::{Connection, ConnectionTrait};

#[derive(Clone, Debug)]
pub struct OsuWindowDetails {
    pub raw_title: String,
    pub artist: String,
    pub title: String,
    pub beatmap: String,
}

fn get_last_index(string: &str, target_char: &char) -> usize {
    let mut last_index = 0;

    for (index, char) in string.chars().enumerate() {
        if &char == target_char {
            last_index = index;
        }
    }

    last_index
}

fn get_osu_window_title() -> Option<String> {
    for title in Connection::new().unwrap().window_titles().unwrap() {
        if title.starts_with("osu!") && title.contains(['-', ']']) {
            return Some(title.chars().skip(8).collect());
        }
    }

    None
}

pub fn get_osu_window_details() -> Option<OsuWindowDetails> {
    if let Some(title) = get_osu_window_title() {
        let beatmap_index = get_last_index(&title, &'[') - 1;
        let beatmap: String = title.chars().skip(beatmap_index + 2).collect();
        let artist_separator_index = title.find(" - ").unwrap();

        return Some(OsuWindowDetails {
            raw_title: title.to_string(),
            title: title
                .chars()
                .take(beatmap_index)
                .skip(artist_separator_index + 3)
                .collect(),
            artist: title.chars().take(artist_separator_index).collect(),
            beatmap: beatmap.chars().take(beatmap.len() - 1).collect(),
        });
    }

    None
}
