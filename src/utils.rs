use crate::{logger::Logger, scores::Score, scrobbler::CONFIG};
use anyhow::{Result, bail};
use chrono::DateTime;
use colored::Colorize;
use std::{borrow::Cow, fmt::Display, io::stdin, process::exit as process_exit};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub fn exit<T: Display>(tag: &str, message: T) -> ! {
    Logger::error(tag, message);
    println!("\nPress enter to exit.");
    let _ = stdin().read_line(&mut String::new());
    process_exit(1);
}

pub fn get_osu_pid() -> Option<u32> {
    let system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    system
        .processes()
        .iter()
        .find(|(_, process)| process.name() == "osu!" || process.name() == "osu!.exe")
        .map(|process| process.0.as_u32())
}

pub fn handle_redirects(score: &Score, artist: &str, title: &str) -> (Option<String>, Option<String>) {
    let clean_extra_whitespaces = |string: Cow<'_, str>| string.trim().split(char::is_whitespace).collect::<Vec<&str>>().join(" ");

    let artists = [score.beatmapset.artist.to_lowercase(), score.beatmapset.artist_unicode.to_lowercase()];
    let mut new_artist = None;

    for (old, new) in &CONFIG.scrobbler.redirects.artists.equal_matches {
        if artists.contains(old) && new != artist {
            new_artist = Some(new.clone());
            break;
        }
    }

    if new_artist.is_none() {
        for (regex, replacer) in &CONFIG.scrobbler.redirects.artists.regex_matches {
            let haystack = new_artist.as_deref().unwrap_or(artist);

            if regex.is_match(haystack) {
                Logger::success(
                    "Scrobbler",
                    format!("Regex {} matched artist {}. Applying...", regex.to_string().bright_blue(), haystack.bright_blue()),
                );

                new_artist = Some(clean_extra_whitespaces(regex.replace(haystack, replacer)));
            }
        }
    }

    let titles = [score.beatmapset.title.to_lowercase(), score.beatmapset.title_unicode.to_lowercase()];
    let mut new_title = None;

    for (old, new) in &CONFIG.scrobbler.redirects.titles.equal_matches {
        if titles.contains(old) && new != title {
            new_title = Some(new.clone());
            break;
        }
    }

    if new_title.is_none() {
        for (regex, replacer) in &CONFIG.scrobbler.redirects.titles.regex_matches {
            let haystack = new_title.as_deref().unwrap_or(title);

            if regex.is_match(haystack) {
                Logger::success(
                    "Scrobbler",
                    format!("Regex {} matched title {}. Applying...", regex.to_string().bright_blue(), haystack.bright_blue()),
                );

                new_title = Some(clean_extra_whitespaces(regex.replace(haystack, replacer)));
            }
        }
    }

    (new_artist, new_title)
}

pub fn validate_scrobble(score: &Score) -> Result<()> {
    let (artist_romanized, artist_original) = (score.beatmapset.artist.to_lowercase(), score.beatmapset.artist_unicode.to_lowercase());

    if CONFIG.scrobbler.blacklist.artists.equal_matches.contains(&artist_romanized)
        || CONFIG.scrobbler.blacklist.artists.equal_matches.contains(&artist_original)
    {
        bail!("Beatmapset artist is blacklisted.");
    }

    if let Some(regex) = CONFIG
        .scrobbler
        .blacklist
        .artists
        .regex_matches
        .iter()
        .find(|regex| regex.is_match(&score.beatmapset.artist) || regex.is_match(&score.beatmapset.artist_unicode))
    {
        bail!("Beatmapset artist matches a blacklisted regex ({}).", regex.as_str().bright_red());
    }

    let (title_romanized, title_original) = (score.beatmapset.title.to_lowercase(), score.beatmapset.title_unicode.to_lowercase());

    if CONFIG.scrobbler.blacklist.titles.equal_matches.contains(&title_romanized)
        || CONFIG.scrobbler.blacklist.titles.equal_matches.contains(&title_original)
    {
        bail!("Beatmapset title is blacklisted.");
    }

    if let Some(regex) = CONFIG
        .scrobbler
        .blacklist
        .titles
        .regex_matches
        .iter()
        .find(|regex| regex.is_match(&score.beatmapset.title) || regex.is_match(&score.beatmapset.title_unicode))
    {
        bail!("Beatmapset title matches a blacklisted regex ({}).", regex.as_str().bright_red());
    }

    let difficulty = score.beatmap.version.to_lowercase();

    if CONFIG.scrobbler.blacklist.difficulties.equal_matches.contains(&difficulty) {
        bail!("Beatmap difficulty is blacklisted.");
    }

    if let Some(regex) = CONFIG.scrobbler.blacklist.difficulties.regex_matches.iter().find(|regex| regex.is_match(&score.beatmap.version)) {
        bail!("Beatmap difficulty matches a blacklisted regex ({}).", regex.as_str().bright_red());
    }

    if score.beatmap.total_length < CONFIG.scrobbler.min_beatmap_length_secs {
        bail!(
            "Beatmap's total length ({}) is less than the configured minimum length ({}).",
            format!("{}s", score.beatmap.total_length).bright_blue(),
            format!("{}s", CONFIG.scrobbler.min_beatmap_length_secs).bright_blue(),
        );
    }

    if !score.passed {
        let started_at = score.started_at.as_ref().and_then(|started_at| DateTime::parse_from_rfc3339(started_at).ok());
        let ended_at = DateTime::parse_from_rfc3339(&score.ended_at).ok();
        let delta = ended_at.and_then(|ended_at| started_at.map(|started_at| (ended_at - started_at).as_seconds_f64())).unwrap_or_default();

        if delta > 0. {
            let rate = score
                .mods
                .iter()
                .find(|score_mod| score_mod.acronym == "DT" || score_mod.acronym == "NC")
                .and_then(|score_mod| score_mod.settings.as_ref().map(|settings| settings.speed_change.unwrap_or(1.5)))
                .unwrap_or(1.);
            let hit_length = score.beatmap.hit_length as f64 / rate;

            // A valid scrobble should be half of the beatmap's hit length or 4 minutes, whichever occurs earlier
            // This might go through if the user paused, took a long break, and continued (just to fail some time after)
            let is_valid_scrobble = delta >= hit_length / 2. || delta >= 60. * 4.;

            if !is_valid_scrobble {
                bail!("Play's progress is less than half of the drain time.");
            }
        }
    }

    Ok(())
}
