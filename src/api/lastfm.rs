use super::MetadataResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LastFmSearchResponse {
    results: LastFmResults,
}

#[derive(Debug, Deserialize)]
struct LastFmResults {
    trackmatches: LastFmTrackMatches,
}

#[derive(Debug, Deserialize)]
struct LastFmTrackMatches {
    track: Vec<LastFmTrack>,
}

#[derive(Debug, Deserialize)]
struct LastFmTrack {
    name: String,
    artist: String,
    image: Option<Vec<LastFmImage>>,
}

#[derive(Debug, Deserialize)]
struct LastFmImage {
    #[serde(rename = "#text")]
    url: String,
    size: String,
}

pub struct LastFmClient {
    api_key: String,
}

impl LastFmClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn search(&self, term: &str) -> Result<Vec<MetadataResult>, String> {
        if self.api_key.is_empty() {
            return Err("Last.fm API Key is missing".to_string());
        }

        let url = format!(
            "http://ws.audioscrobbler.com/2.0/?method=track.search&track={}&api_key={}&format=json",
            urlencoding::encode(term),
            self.api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| format!("Last.fm request failed: {}", e))?;

        if !response.status().is_success() {
             return Err(format!("Last.fm request failed with status: {}", response.status()));
        }

        let lastfm_res: LastFmSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Last.fm parse failed: {}", e))?;

        let results = lastfm_res.results.trackmatches.track.into_iter().map(|track| {
            let mut best_image = None;
            if let Some(images) = track.image {
                if let Some(img) = images.iter().find(|i| i.size == "extralarge") {
                    if !img.url.is_empty() { best_image = Some(img.url.clone()); }
                }
                if best_image.is_none() {
                     if let Some(img) = images.iter().find(|i| i.size == "large") {
                        if !img.url.is_empty() { best_image = Some(img.url.clone()); }
                    }
                }
            }

            MetadataResult {
                title: track.name,
                artist: track.artist,
                album: "Unknown (Last.fm)".to_string(),
                cover_url: best_image,
                source: "Last.fm".to_string(),
            }
        }).collect();

        Ok(results)
    }
}
