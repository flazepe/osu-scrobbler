use crate::config::get_config;
use crate::nerinyan::get_beatmapset;
use crate::osu::get_osu_window_details;
use crate::osu_scrobble::OsuScrobble;
use async_recursion::async_recursion;
use std::time::Duration;
use tokio::time;

#[async_recursion]
pub async fn main(scrobbler: &rustfm_scrobble::Scrobbler, mut osu_scrobble: Option<OsuScrobble>) {
    let config = get_config();

    match get_osu_window_details() {
        Some(window_details) => {
            if osu_scrobble.is_none()
                || &osu_scrobble.as_ref().unwrap().window_details.title != &window_details.title
            {
                if let Some(beatmapset) = get_beatmapset(&window_details).await {
                    if beatmapset.length >= config.options.min_beatmap_length_seconds {
                        osu_scrobble = Some(OsuScrobble::new(&window_details, &beatmapset));

                        // Hide this for now. Need to figure out how to make the last now playing message disappear after scrobbling a track.
                        /*
                        last_fm::set_now_playing(
                            &scrobbler,
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

                        println!(
                            "Now playing: {}!",
                            osu_scrobble.as_ref().unwrap().window_details.raw_title
                        );
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

            osu_scrobble = None;
        }
    }

    let mut interval = time::interval(Duration::from_millis(5000));

    interval.tick().await;
    interval.tick().await;

    main(scrobbler, osu_scrobble).await
}
