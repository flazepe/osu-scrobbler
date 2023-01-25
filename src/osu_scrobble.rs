use crate::config::get_config;
use crate::last_fm::scrobble;
use crate::nerinyan::Beatmapset;
use crate::osu::OsuWindowDetails;
use rustfm_scrobble::Scrobbler;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct OsuScrobble {
    pub window_details: OsuWindowDetails,
    pub beatmapset: Beatmapset,
    end_timestamp: u64,
}

impl OsuScrobble {
    pub fn new(window_details: &OsuWindowDetails, beatmapset: &Beatmapset) -> Self {
        Self {
            window_details: window_details.to_owned(),
            beatmapset: beatmapset.to_owned(),
            end_timestamp: OsuScrobble::get_current_timestamp() + (beatmapset.length / 2),
        }
    }

    pub fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn end(&self, scrobbler: &Scrobbler) {
        if OsuScrobble::get_current_timestamp() >= self.end_timestamp {
            println!("Scrobbled!");

            let config = get_config();

            scrobble(
                &scrobbler,
                if config.options.use_original_metadata {
                    &self.beatmapset.title_unicode
                } else {
                    &self.beatmapset.title
                },
                if config.options.use_original_metadata {
                    &self.beatmapset.artist_unicode
                } else {
                    &self.beatmapset.artist
                },
                if config.options.use_original_metadata {
                    &self.beatmapset.title_unicode
                } else {
                    &self.beatmapset.title
                },
            );
        } else {
            println!("Not scrobbled.");
        }
    }
}
