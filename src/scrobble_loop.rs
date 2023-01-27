use crate::{
    config::{get_config, ScrobbleConfig},
    last_fm::LastfmScrobbler,
    osu::{nerinyan::get_beatmapset, scrobble::OsuScrobble, window::get_osu_window_details},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn check(
    scrobble_config: &ScrobbleConfig,
    scrobbler: &LastfmScrobbler,
    osu_scrobble: &mut Option<OsuScrobble>,
) {
    match get_osu_window_details() {
        Some(window_details) => {
            if osu_scrobble.is_none() {
                if let Some(beatmapset) = get_beatmapset(&window_details) {
                    if beatmapset.length >= scrobble_config.min_beatmap_length_seconds {
                        *osu_scrobble = Some(OsuScrobble::new(&window_details, &beatmapset));

                        println!(
                            "Playing: {} - {}",
                            if scrobble_config.use_original_metadata {
                                &beatmapset.artist_unicode
                            } else {
                                &beatmapset.artist
                            },
                            if scrobble_config.use_original_metadata {
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
                osu_scrobble.end(scrobble_config, scrobbler);
            }

            *osu_scrobble = None;
        }
    }
}

pub fn main() {
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
        let timestamp = get_current_timestamp() + 5;
        while get_current_timestamp() < timestamp {}
    }
}
