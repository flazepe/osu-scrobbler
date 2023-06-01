use crate::{
    config::{get_config, ScrobbleConfig},
    last_fm::Scrobbler as LastfmScrobbler,
    osu::{
        nerinyan::{get_beatmapset, CompactBeatmapset},
        window::get_window_title,
    },
};
use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct Scrobbler {
    config: ScrobbleConfig,
    last_fm: LastfmScrobbler,
    beatmapset: Option<CompactBeatmapset>,
    timestamp: u64,
}

impl Scrobbler {
    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn new() -> Self {
        let config = get_config();

        Self {
            config: config.scrobble,
            last_fm: LastfmScrobbler::new(
                &config.last_fm.username,
                &config.last_fm.password,
                &config.last_fm.api_key,
                &config.last_fm.api_secret,
            ),
            beatmapset: None,
            timestamp: Scrobbler::get_current_timestamp(),
        }
    }

    pub fn start(&mut self) {
        println!("Started osu-scrobbler!");

        loop {
            self.poll();
            sleep(Duration::from_secs(1));
        }
    }

    fn poll(&mut self) {
        match get_window_title() {
            Some(window_title) => {
                if self.beatmapset.is_none() {
                    let Some(beatmapset) = get_beatmapset(&window_title) else { return; };
                    self.start_scrobble(&beatmapset);
                }
            }
            None => self.end_scrobble(),
        }
    }

    fn start_scrobble(&mut self, beatmapset: &CompactBeatmapset) {
        if beatmapset.total_length < self.config.min_beatmap_length_secs {
            return;
        }

        self.beatmapset = Some(beatmapset.clone());
        self.timestamp = Scrobbler::get_current_timestamp();

        println!(
            "Playing: {} - {}",
            match self.config.use_original_metadata {
                true => &beatmapset.artist_unicode,
                false => &beatmapset.artist,
            },
            match self.config.use_original_metadata {
                true => &beatmapset.title_unicode,
                false => &beatmapset.title,
            },
        );
    }

    fn end_scrobble(&mut self) {
        let Some(beatmapset) = &self.beatmapset else { return; };
        let timestamp = Scrobbler::get_current_timestamp();

        match timestamp >= self.timestamp + (beatmapset.total_length as u64 / 2)
            || timestamp >= self.timestamp + 240
        {
            true => match self.last_fm.scrobble(
                match self.config.use_original_metadata {
                    true => &beatmapset.title_unicode,
                    false => &beatmapset.title,
                },
                match self.config.use_original_metadata {
                    true => &beatmapset.artist_unicode,
                    false => &beatmapset.artist,
                },
                match self.config.use_original_metadata {
                    true => &beatmapset.title_unicode,
                    false => &beatmapset.title,
                },
            ) {
                Ok(_) => println!("Scrobbled ^"),
                Err(error) => {
                    println!("An error occurred while scrobbling to Last.fm: {error}")
                }
            },
            false => println!("Not scrobbled ^"),
        }

        self.beatmapset = None;
    }
}
