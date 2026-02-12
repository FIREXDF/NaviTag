use super::MetadataResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Debug, Deserialize)]
struct ItunesTrack {
    #[serde(rename = "trackName")]
    track_name: Option<String>,
    #[serde(rename = "artistName")]
    artist_name: Option<String>,
    #[serde(rename = "collectionName")]
    collection_name: Option<String>,
    #[serde(rename = "artworkUrl100")]
    artwork_url: Option<String>,
}

pub async fn search(term: &str) -> Result<Vec<MetadataResult>, String> {
    let url = format!(
        "https://itunes.apple.com/search?term={}&media=music&entity=song&limit=10",
        urlencoding::encode(term)
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json::<ItunesResponse>()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    let results = response.results.into_iter().map(|t| MetadataResult {
        title: t.track_name.unwrap_or_default(),
        artist: t.artist_name.unwrap_or_default(),
        album: t.collection_name.unwrap_or_default(),
        cover_url: t.artwork_url.map(|u| u.replace("100x100", "600x600")),
        source: "Apple Music".to_string(),
    }).collect();

    Ok(results)
}
