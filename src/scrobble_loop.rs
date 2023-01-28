use crate::{
    config::{get_config, ScrobbleConfig},
    last_fm::LastfmScrobbler,
    osu::{nerinyan::get_beatmapset, scrobble::OsuScrobble, window::get_window_title},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn check(
    config: &ScrobbleConfig,
    scrobbler: &LastfmScrobbler,
    osu_scrobble: &mut Option<OsuScrobble>,
) {
    match get_window_title() {
        Some(window_title) => {
            if osu_scrobble.is_none() {
                if let Some(beatmapset) = get_beatmapset(&window_title) {
                    if beatmapset.length >= config.min_beatmap_length_seconds {
                        *osu_scrobble = Some(OsuScrobble::new(&beatmapset));

                        println!(
                            "Playing: {} - {}",
                            if config.use_original_metadata {
                                &beatmapset.artist_unicode
                            } else {
                                &beatmapset.artist
                            },
                            if config.use_original_metadata {
                                &beatmapset.title_unicode
                            } else {
                                &beatmapset.title
                            }
                        );
                    }
                }
            }
        }
        None => {
            if let Some(osu_scrobble) = osu_scrobble {
                osu_scrobble.end(config, scrobbler);
            }

            *osu_scrobble = None;
        }
    }
}

pub fn start() {
    let config = get_config();

    let scrobbler = LastfmScrobbler::new(
        &config.last_fm.username,
        &config.last_fm.password,
        &config.last_fm.api_key,
        &config.last_fm.api_secret,
    );

    let mut osu_scrobble = None;

    loop {
        check(&config.scrobble, &scrobbler, &mut osu_scrobble);
        let timestamp = get_current_timestamp() + 1;
        while get_current_timestamp() < timestamp {}
    }
}
