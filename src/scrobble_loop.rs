use crate::{
    config::get_config,
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

fn check(osu_scrobble: &mut Option<OsuScrobble>) {
    let config = get_config();

    match get_osu_window_details() {
        Some(window_details) => {
            if osu_scrobble.is_none()
                || osu_scrobble.as_ref().unwrap().window_details.title != window_details.title
            {
                if let Some(beatmapset) = get_beatmapset(&window_details) {
                    if beatmapset.length >= config.options.min_beatmap_length_seconds {
                        *osu_scrobble = Some(OsuScrobble::new(
                            LastfmScrobbler::new(),
                            &window_details,
                            &beatmapset,
                        ));

                        println!(
                            "Playing: {} - {}",
                            if config.options.use_original_metadata {
                                &beatmapset.artist_unicode
                            } else {
                                &beatmapset.artist
                            },
                            if config.options.use_original_metadata {
                                &beatmapset.title_unicode
                            } else {
                                &beatmapset.title
                            }
                        );
                    } else {
                        *osu_scrobble = None;
                    }
                } else {
                    *osu_scrobble = None;
                    println!("Could not find {} in mirror.", window_details.raw_title);
                }
            }
        }
        None => {
            if let Some(osu_scrobble) = osu_scrobble {
                osu_scrobble.end();
            }

            *osu_scrobble = None;
        }
    }
}

pub fn main() {
    let mut osu_scrobble = None;

    loop {
        check(&mut osu_scrobble);
        let timestamp = get_current_timestamp() + 5;
        while get_current_timestamp() < timestamp {}
    }
}
