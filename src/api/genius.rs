use super::MetadataResult;
use serde::Deserialize;
use reqwest::header::AUTHORIZATION;

#[derive(Debug, Deserialize)]
struct GeniusSearchResponse {
    response: GeniusResponseData,
}

#[derive(Debug, Deserialize)]
struct GeniusResponseData {
    hits: Vec<GeniusHit>,
}

#[derive(Debug, Deserialize)]
struct GeniusHit {
    result: GeniusSong,
}

#[derive(Debug, Deserialize)]
struct GeniusSong {
    title: String,
    artist_names: String,
    song_art_image_url: Option<String>,
}

pub struct GeniusClient {
    access_token: String,
}

impl GeniusClient {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    pub async fn search(&self, term: &str) -> Result<Vec<MetadataResult>, String> {
        if self.access_token.is_empty() {
            return Err("Genius Access Token is missing".to_string());
        }

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.genius.com/search?q={}",
            urlencoding::encode(term)
        );

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| format!("Genius request failed: {}", e))?;

        if !response.status().is_success() {
             return Err(format!("Genius request failed with status: {}", response.status()));
        }

        let genius_res: GeniusSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Genius parse failed: {}", e))?;

        let results = genius_res.response.hits.into_iter().map(|hit| {
            MetadataResult {
                title: hit.result.title,
                artist: hit.result.artist_names,
                album: "Unknown (Genius)".to_string(),
                cover_url: hit.result.song_art_image_url,
                source: "Genius".to_string(),
            }
        }).collect();

        Ok(results)
    }
}
