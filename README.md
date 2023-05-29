# osu-scrobbler

An osu! Last.fm scrobbler. This scrobbler only works for gameplay!

## Setup

-   Download the latest release.
-   Extract the archive you downloaded to your desired folder.
-   Navigate to that folder and rename the `config.example.toml` file to `config.toml`.
-   Get your Last.fm API credentials from [here](https://www.last.fm/api/account/create) (or [here](https://www.last.fm/api/accounts) if you already have one) and edit the configuration values accordingly.

## Caveats

-   Since this scrobbler heavily relies on osu!'s window title, a scrobble will still count if you pause or stay on the fail screen for long enough.
-   This scrobbler relies on the beatmap you're playing to be on the [Nerinyan](https://nerinyan.moe) mirror. If the beatmap isn't on the mirror, your play won't be scrobbled. This shouldn't be a problem in most cases.
