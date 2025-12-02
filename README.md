# osu-scrobbler

An osu! Last.fm and ListenBrainz scrobbler. This scrobbler only works for gameplay!

## Setup

1.  Download the latest release binary for your operating system [here](https://github.com/flazepe/osu-scrobbler/releases/latest).

2.  Create a folder called `osu-scrobbler` (or anything really) in your system and move the downloaded binary file there.

3.  In the same folder, create a file called `config.toml` with [this](https://github.com/flazepe/osu-scrobbler/blob/master/config.example.toml) as the content.

> [!TIP]
> The `config.toml` file can also be placed anywhere and its path can be supplied through the `OSU_SCROBBLER_CONFIG_PATH` environment variable while executing the binary.

4.  Get your Last.fm API credentials [here](https://www.last.fm/api/account/create) (or [here](https://www.last.fm/api/accounts) if you already have one).

    -   For ListenBrainz users, head [here](https://listenbrainz.org/profile/) to get your user token.

5.  Edit the configuration values accordingly.

    -   Most notably, your osu! user ID and scrobbler credentials.

6.  Execute the binary in the terminal and the scrobbler should start.

> [!TIP]
> If you're on Windows or macOS, you can also double click the binary file to launch the scrobbler.

## Configuration

Refer [here](https://github.com/flazepe/osu-scrobbler/wiki/Configuration) for more details about the configuration properties.

## Autostart

You can configure the scrobbler to automatically start on startup. Refer [here](https://github.com/flazepe/osu-scrobbler/wiki/Autostart) for the necessary steps.
