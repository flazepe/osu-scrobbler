# osu-scrobbler

An osu! Last.fm and ListenBrainz scrobbler. This scrobbler only works for gameplay!

# Setup

-   Download the latest release.
-   Extract the archive you downloaded to your desired folder.
-   Navigate to that folder and rename the `config.example.toml` file to `config.toml`.
    -   The `config.toml` file can also be placed anywhere and its path can be supplied through the `OSU_SCROBBLER_CONFIG_PATH` environment variable.
-   Get your Last.fm API credentials from [here](https://www.last.fm/api/account/create) (or [here](https://www.last.fm/api/accounts) if you already have one) and edit the configuration values accordingly.
-   For ListenBrainz users, head [here](https://listenbrainz.org/profile/) to get your user token.

# Configuration

An example configuration file can be viewed [here](https://github.com/flazepe/osu-scrobbler/blob/master/config.example.toml).

### `[scrobbler]`

The scrobbler configuration

-   `user_id` (required)

    -   Your osu! user ID
    -   e.g. `8834263`

-   `mode`

    -   The osu! mode to scrobble
    -   Possible values: `"default"`, `"osu"`, `"taiko"`, `"fruits"`, `"mania"`
    -   Default: `"default"`

-   `use_original_metadata`

    -   Whether to use original metadata instead of romanized metadata
    -   Default: `true`

-   `min_beatmap_length_secs`

    -   The minimum beatmap length to scrobble (in seconds)
    -   Default: `60`

-   `scrobble_fails`

    -   Whether to scrobble failed plays. On osu!lazer, the failed duration must at least be 50% of the entire beatmap or 4 minutes
    -   Default: `false`

-   `log_scrobbles`

    -   Whether to log scrobbles to a file
    -   Default: `false`

-   `artist_redirects`

    -   A list of tuples that contain the old and new artist name to replace with before scrobbling (case-insensitive)
    -   Example: `[["96猫", "Kuroneko"], ["ツユ", "TUYU"]]`

-   `artist_regex_redirects`

    -   A list of tuples that contain a regex pattern and replacer string to replace artists with before scrobbling (case-sensitive)
    -   The regex should account for the original metadata instead of the romanized data, unless `use_original_metadata` is set to `false`
    -   Put more ambiguous patterns last, since the program only finds the first match to replace
    -   Example: `[["(.+) feat\\. ななひら", "Nanahira"], ["(.+) feat\\. .+", "$1"]]`

-   `title_regex_redirects`

    -   A list of tuples that contain a regex pattern and replacer string to replace titles with before scrobbling (case-sensitive)
    -   The regex should account for the original metadata instead of the romanized data, unless `use_original_metadata` is set to `false`
    -   Put more ambiguous patterns last, since the program only finds the first match to replace
    -   Example: `[["(.+) (?i:\\(TV Size\\))", "$1"], ["(.+) (?i:\\(.+ Remix\\))", "$1"]]`

### `[scrobbler.blacklist.artists]`

Artists to blacklist from scrobbling

-   `equals`

    -   A list of artists to blacklist (case-insensitive)

-   `contains_words`

    -   A list of words inside an artist to blacklist (case-insensitive)

-   `matches_regex`

    -   A list of regex pattern strings to blacklist (case-sensitive)

### `[scrobbler.blacklist.titles]`

Titles to blacklist from scrobbling

-   `equals`

    -   A list of titles to blacklist (case-insensitive)

-   `contains_words`

    -   A list of words inside a title to blacklist (case-insensitive)

-   `matches_regex`

    -   A list of regex pattern strings to blacklist (case-sensitive)

### `[scrobbler.blacklist.difficulties]`

Difficulties to blacklist from scrobbling

-   `equals`

    -   A list difficulties to blacklist (case-insensitive)

-   `contains_words`

    -   A list of words inside a difficulty to blacklist (case-insensitive)

-   `matches_regex`

    -   A list of regex pattern strings to blacklist (case-sensitive)

### `[last_fm]` (required, but optional if you're already using at least one other scrobbler)

The Last.fm configuration

-   `username` (required)

    -   Your Last.fm username

-   `password` (required)

    -   Your Last.fm password

-   `api_key` (required)

    -   Your Last.fm API key

-   `api_secret` (required)

    -   Your Last.fm API secret

### `[listenbrainz]` (required, but optional if you're already using at least one other scrobbler)

The ListenBrainz configuration

-   `user_token` (required)

    -   Your ListenBrainz user token

# systemd Service Example

```ini
[Unit]
Description=osu-scrobbler
After=network.target
Wants=network-online.target

[Service]
Restart=always
RestartSec=10
ExecStart=/path/to/osu-scrobbler/executable
Environment=OSU_SCROBBLER_CONFIG_PATH=/path/to/osu-scrobbler/config.toml

[Install]
WantedBy=multi-user.target
```
