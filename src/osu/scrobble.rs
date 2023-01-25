use crate::config::get_config;
use crate::last_fm::LastfmScrobbler;
use crate::osu::{nerinyan::Beatmapset, window::OsuWindowDetails};
use crate::scrobble_loop::get_current_timestamp;

pub struct OsuScrobble {
    scrobbler: LastfmScrobbler,
    pub window_details: OsuWindowDetails,
    pub beatmapset: Beatmapset,
    end_timestamps: Vec<u64>,
}

impl OsuScrobble {
    pub fn new(
        scrobbler: LastfmScrobbler,
        window_details: &OsuWindowDetails,
        beatmapset: &Beatmapset,
    ) -> Self {
        let timestamp = get_current_timestamp();

        // Hide this for now. Need to figure out how to make the last now playing message disappear after scrobbling a track.
        /*
        let config = get_config();

        scrobbler.set_now_playing(
            if config.options.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
            if config.options.use_original_metadata {
                &beatmapset.artist_unicode
            } else {
                &beatmapset.artist
            },
            if config.options.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
        );
        */

        Self {
            scrobbler: scrobbler,
            window_details: window_details.to_owned(),
            beatmapset: beatmapset.to_owned(),
            end_timestamps: vec![timestamp + (beatmapset.length / 2), timestamp + 240],
        }
    }

    pub fn end(&self) {
        let timestamp = get_current_timestamp();

        if timestamp >= self.end_timestamps[0] || timestamp >= self.end_timestamps[1] {
            println!("Scrobbled ^");

            let config = get_config();

            self.scrobbler.scrobble(
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
            println!("Not scrobbled ^");
        }
    }
}
