use crate::config::ScrobbleConfig;
use crate::last_fm::LastfmScrobbler;
use crate::osu::nerinyan::Beatmapset;
use crate::scrobble_loop::get_current_timestamp;

pub struct OsuScrobble {
    pub beatmapset: Beatmapset,
    timestamp: u64,
}

impl OsuScrobble {
    pub fn new(beatmapset: &Beatmapset) -> Self {
        // Hide this for now. Need to figure out how to make the last now playing message disappear after scrobbling a track.
        /*
        match scrobbler.set_now_playing(
            if config.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
            if config.use_original_metadata {
                &beatmapset.artist_unicode
            } else {
                &beatmapset.artist
            },
            if config.use_original_metadata {
                &beatmapset.title_unicode
            } else {
                &beatmapset.title
            },
        ) {
            Ok(_) => (),
            Err(err) => println!(
                "An error occurred while trying to set Last.fm now playing: {}",
                err
            ),
        }
        */

        Self {
            beatmapset: beatmapset.to_owned(),
            timestamp: get_current_timestamp(),
        }
    }

    pub fn end(&self, config: &ScrobbleConfig, scrobbler: &LastfmScrobbler) {
        let timestamp = get_current_timestamp();

        if timestamp >= self.timestamp + (u64::from(self.beatmapset.length) / 2)
            || timestamp >= self.timestamp + 240
        {
            match scrobbler.scrobble(
                if config.use_original_metadata {
                    &self.beatmapset.title_unicode
                } else {
                    &self.beatmapset.title
                },
                if config.use_original_metadata {
                    &self.beatmapset.artist_unicode
                } else {
                    &self.beatmapset.artist
                },
                if config.use_original_metadata {
                    &self.beatmapset.title_unicode
                } else {
                    &self.beatmapset.title
                },
            ) {
                Ok(_) => println!("Scrobbled ^"),
                Err(err) => println!(
                    "An error occurred while trying to scrobble in Last.fm: {}",
                    err
                ),
            };
        } else {
            println!("Not scrobbled ^");
        }
    }
}
