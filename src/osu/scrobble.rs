use crate::config::get_config;
use crate::last_fm::LastfmScrobbler;
use crate::osu::{nerinyan::Beatmapset, window::OsuWindowDetails};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct OsuScrobble {
    pub window_details: OsuWindowDetails,
    pub beatmapset: Beatmapset,
    end_timestamps: Vec<u64>,
}

impl OsuScrobble {
    pub fn new(window_details: &OsuWindowDetails, beatmapset: &Beatmapset) -> Self {
        let timestamp = OsuScrobble::get_current_timestamp();

        Self {
            window_details: window_details.to_owned(),
            beatmapset: beatmapset.to_owned(),
            end_timestamps: vec![timestamp + (beatmapset.length / 2), timestamp + 240],
        }
    }

    pub fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn end(&self, scrobbler: &LastfmScrobbler) {
        let timestamp = OsuScrobble::get_current_timestamp();

        if self
            .to_owned()
            .end_timestamps
            .into_iter()
            .any(|end_timestamp| timestamp >= end_timestamp)
        {
            println!("Scrobbled!");

            let config = get_config();

            scrobbler.scrobble(
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
