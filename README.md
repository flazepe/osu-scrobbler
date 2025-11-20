# osu-scrobbler

An osu! Last.fm and ListenBrainz scrobbler. This scrobbler only works for gameplay!

## Setup

-   Download the latest release.
-   Extract the archive you downloaded to your desired folder.
-   Navigate to that folder and rename the `config.example.toml` file to `config.toml`.
    -   The `config.toml` file can also be placed anywhere and its path can be supplied through the `OSU_SCROBBLER_CONFIG_PATH` environment variable.
-   Get your Last.fm API credentials from [here](https://www.last.fm/api/account/create) (or [here](https://www.last.fm/api/accounts) if you already have one) and edit the configuration values accordingly.
-   For ListenBrainz users, head [here](https://listenbrainz.org/profile/) to get your user token.

## Configuration

Refer [here](https://github.com/flazepe/osu-scrobbler/wiki/Configuration) for more details about the configuration properties.

## Autostart

You can configure the scrobbler to automatically start on startup. Refer [here](https://github.com/flazepe/osu-scrobbler/wiki/Autostart) for the necessary steps.
