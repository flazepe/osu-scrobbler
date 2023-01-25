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

pub fn main(scrobbler: &LastfmScrobbler, mut osu_scrobble: Option<OsuScrobble>) {
    let config = get_config();

    match get_osu_window_details() {
        Some(window_details) => {
            if osu_scrobble.is_none()
                || &osu_scrobble.as_ref().unwrap().window_details.title != &window_details.title
            {
                if let Some(beatmapset) = get_beatmapset(&window_details) {
                    if beatmapset.length >= config.options.min_beatmap_length_seconds {
                        osu_scrobble = Some(OsuScrobble::new(&window_details, &beatmapset));

                        // Hide this for now. Need to figure out how to make the last now playing message disappear after scrobbling a track.
                        /*
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

                        println!("Now playing: {}", window_details.raw_title);
                    } else {
                        osu_scrobble = None;
                    }
                } else {
                    osu_scrobble = None;
                    println!("Could not find {} in mirror.", window_details.raw_title);
                }
            }
        }
        None => {
            if let Some(osu_scrobble) = &osu_scrobble {
                osu_scrobble.end(&scrobbler);
            }

            if osu_scrobble.is_some() {
                osu_scrobble = None;
            }
        }
    }

    let timestamp = get_current_timestamp() + 5;
    while get_current_timestamp() < timestamp {}

    main(scrobbler, osu_scrobble)
}
