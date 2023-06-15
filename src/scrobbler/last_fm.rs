use colored::Colorize;
use md5::compute;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

const API_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct LastfmScrobbler {
    client: Client,
    api_key: String,
    api_secret: String,
    session_key: String,
}

#[derive(Deserialize)]
struct LastfmSession {
    session: LastfmSessionData,
}

#[derive(Deserialize)]
struct LastfmSessionData {
    name: String,
    key: String,
}

impl LastfmScrobbler {
    pub fn new(username: String, password: String, api_key: String, api_secret: String) -> Self {
        let client = Client::new();

        let session_key = match client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(&Self::sign_query(
                BTreeMap::from([
                    ("format", "json".to_string()),
                    ("api_key", api_key.to_string()),
                    ("method", "auth.getMobileSession".to_string()),
                    ("password", password),
                    ("username", username),
                ]),
                &api_secret,
            ))
            .send()
            .unwrap()
            .json::<LastfmSession>()
        {
            Ok(session) => {
                println!("{} Successfully authenticated with username {}.", "[Last.fm]".bright_green(), session.session.name.bright_blue());
                session.session.key
            },
            Err(_) => panic!("{} Invalid credentials provided.", "[Last.fm]".bright_red()),
        };

        Self { client, api_key, api_secret, session_key }
    }

    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<(), String> {
        match self
            .client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(&Self::sign_query(
                BTreeMap::from([
                    ("api_key", self.api_key.to_string()),
                    ("artist[0]", artist.to_string()),
                    ("duration[0]", total_length.to_string()),
                    ("method", "track.scrobble".to_string()),
                    ("sk", self.session_key.to_string()),
                    ("timestamp[0]", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()),
                    ("track[0]", title.to_string()),
                ]),
                &self.api_secret,
            ))
            .send()
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                status_code => Err(format!("Received status code {status_code}.")),
            },
            Err(error) => Err(error.to_string()),
        }
    }

    fn sign_query<'a>(mut query: BTreeMap<&'a str, String>, api_secret: &str) -> BTreeMap<&'a str, String> {
        let mut string = String::new();

        for (key, value) in query.iter() {
            if *key == "format" {
                continue;
            }

            string += format!("{key}{value}").as_str();
        }

        string += api_secret;
        query.insert("api_sig", format!("{:x}", compute(string)));
        query
    }
}
