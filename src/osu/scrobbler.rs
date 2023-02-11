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
                    if let Some(beatmapset) = get_beatmapset(&window_title) {
                        if beatmapset.total_length >= self.config.min_beatmap_length_secs {
                            self.start_scrobble(&beatmapset);
                        }
                    }
                }
            }
            None => self.end_scrobble(),
        }
    }

    fn start_scrobble(&mut self, beatmapset: &CompactBeatmapset) {
        self.beatmapset = Some(beatmapset.to_owned());
        self.timestamp = Scrobbler::get_current_timestamp();

        println!(
            "Playing: {} - {}",
            if self.config.use_original_metadata {
                &beatmapset.artist_unicode
            } else {
                &beatmapset.artist
            },
            if self.config.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            }
        );

        // Hide this for now. Need to figure out how to make the last now playing message disappear after scrobbling a track.
        /*
        match self.last_fm.set_now_playing(
            if self.config.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
            if self.config.use_original_metadata {
                &beatmapset.artist_unicode
            } else {
                &beatmapset.artist
            },
            if self.config.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
        ) {
            Ok(_) => (),
            Err(err) => println!(
                "An error occurred while setting Last.fm now playing: {}",
                err
            ),
        }
        */
    }

    fn end_scrobble(&mut self) {
        if let Some(beatmapset) = &self.beatmapset {
            let timestamp = Scrobbler::get_current_timestamp();

            if timestamp >= self.timestamp + (beatmapset.total_length as u64 / 2)
                || timestamp >= self.timestamp + 240
            {
                match self.last_fm.scrobble(
                    if self.config.use_original_metadata {
                        &beatmapset.title_unicode
                    } else {
                        &beatmapset.title
                    },
                    if self.config.use_original_metadata {
                        &beatmapset.artist_unicode
                    } else {
                        &beatmapset.artist
                    },
                    if self.config.use_original_metadata {
                        &beatmapset.title_unicode
                    } else {
                        &beatmapset.title
                    },
                ) {
                    Ok(_) => println!("Scrobbled ^"),
                    Err(err) => println!("An error occurred while scrobbling to Last.fm: {}", err),
                };
            } else {
                println!("Not scrobbled ^");
            }

            self.beatmapset = None;
        }
    }
}
