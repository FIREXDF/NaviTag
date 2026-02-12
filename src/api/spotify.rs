use super::MetadataResult;
use serde::Deserialize;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

#[derive(Debug, Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct SpotifySearchResponse {
    tracks: Tracks,
}

#[derive(Debug, Deserialize)]
struct Tracks {
    items: Vec<Track>,
}

#[derive(Debug, Deserialize)]
struct Track {
    name: String,
    album: Album,
    artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
struct Album {
    name: String,
    images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
struct Artist {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Image {
    url: String,
    height: Option<u32>,
    width: Option<u32>,
}

pub struct SpotifyClient {
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
}

impl SpotifyClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            access_token: None,
        }
    }

    pub async fn authenticate(&mut self) -> Result<(), String> {
        let client = reqwest::Client::new();
        let params = [("grant_type", "client_credentials")];
        
        let response = client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Auth request failed: {}", e))?;

        if !response.status().is_success() {
             return Err(format!("Auth failed with status: {}", response.status()));
        }

        let token_res: SpotifyTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Auth parse failed: {}", e))?;

        self.access_token = Some(token_res.access_token);
        Ok(())
    }

    pub async fn search(&mut self, term: &str) -> Result<Vec<MetadataResult>, String> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let token = self.access_token.as_ref().unwrap();
        let client = reqwest::Client::new();
        
        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
            urlencoding::encode(term)
        );

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Search request failed: {}", e))?;
        
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.authenticate().await?;
            let token = self.access_token.as_ref().unwrap();
             return self.search_retry(term, token).await;
        }

        if !response.status().is_success() {
            return Err(format!("Search failed with status: {}", response.status()));
        }

        let search_res: SpotifySearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Search parse failed: {}", e))?;

        let results = search_res.tracks.items.into_iter().map(|t| {
            let artist = t.artists.first().map(|a| a.name.clone()).unwrap_or_default();
            let cover_url = t.album.images.first().map(|i| i.url.clone());
            
            MetadataResult {
                title: t.name,
                artist,
                album: t.album.name,
                cover_url,
                source: "Spotify".to_string(),
            }
        }).collect();

        Ok(results)
    }

    async fn search_retry(&self, term: &str, token: &str) -> Result<Vec<MetadataResult>, String> {
          let client = reqwest::Client::new();
           let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
            urlencoding::encode(term)
        );

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Retry search request failed: {}", e))?;
        
         let search_res: SpotifySearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Retry search parse failed: {}", e))?;

        Ok(search_res.tracks.items.into_iter().map(|t| {
            let artist = t.artists.first().map(|a| a.name.clone()).unwrap_or_default();
            let cover_url = t.album.images.first().map(|i| i.url.clone());
            
            MetadataResult {
                title: t.name,
                artist,
                album: t.album.name,
                cover_url,
                source: "Spotify".to_string(),
            }
        }).collect())
    }
}
