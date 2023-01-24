use async_recursion::async_recursion;
use std::time::Duration;
use tokio::{main, time};

mod config;
mod last_fm;
mod osu;
mod spotify;

#[async_recursion]
async fn check_loop(scrobbler: rustfm_scrobble::Scrobbler, old_title: String) {
    let mut _title = old_title.to_string();

    match osu::get_osu_window_title() {
        Some(new_title) => {
            let (title, artist) = osu::separate_title_and_artist(&new_title);
            last_fm::set_now_playing(&scrobbler, &title, &artist, &title);

            if new_title != old_title {
                _title = new_title.to_string();
                println!("osu! is playing: {}", new_title);

                /*
                if let Some(track) = spotify::get_track(&new_title).await {
                    last_fm::set_now_playing(&scrobbler, &track.title, &track.artist, &track.album);

                    println!(
                        "Set now playing to {} by {} on {} (length: {}s)!",
                        &track.title,
                        &track.artist,
                        &track.album,
                        &track.length / 1000
                    );
                }
                */
            }
        }
        None => {
            if _title != "" {
                _title = String::from("");
                println!("osu! is playing: -");
            }
        }
    }

    let mut interval = time::interval(Duration::from_millis(5000));

    interval.tick().await;
    interval.tick().await;

    check_loop(scrobbler, _title).await
}

#[main]
async fn main() {
    check_loop(last_fm::get_scrobbler(), String::from("")).await;
}
