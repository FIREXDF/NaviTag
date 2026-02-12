pub mod apple_music;
pub mod spotify;
pub mod genius;
pub mod lastfm;

#[derive(Debug, Clone)]
pub struct MetadataResult {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_url: Option<String>,
    pub source: String,
}

use crate::settings::UserSettings;

pub async fn search_all(term: String, settings: UserSettings) -> Vec<MetadataResult> {
    let mut results = Vec::new();

    let apple_future = async {
        if settings.enable_apple_music {
            apple_music::search(&term).await.unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    let spotify_future = async {
        if settings.enable_spotify && !settings.spotify_id.is_empty() {
             let mut client = spotify::SpotifyClient::new(settings.spotify_id.clone(), settings.spotify_secret.clone());
             client.search(&term).await.unwrap_or_default()
        } else {
             Vec::new()
        }
    };

    let genius_future = async {
        if settings.enable_genius && !settings.genius_token.is_empty() {
            let client = genius::GeniusClient::new(settings.genius_token.clone());
            client.search(&term).await.unwrap_or_default()
        } else {
             Vec::new()
        }
    };

    let lastfm_future = async {
        if settings.enable_lastfm && !settings.lastfm_api_key.is_empty() {
            let client = lastfm::LastFmClient::new(settings.lastfm_api_key.clone());
            client.search(&term).await.unwrap_or_default()
        } else {
             Vec::new()
        }
    };

    let (r1, r2, r3, r4) = tokio::join!(apple_future, spotify_future, genius_future, lastfm_future);

    results.extend(r1);
    results.extend(r2);
    results.extend(r3);
    results.extend(r4);
    
    results
}
