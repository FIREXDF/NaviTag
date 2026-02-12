use image::GenericImageView;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::file::AudioFile as LoftyAudioFile;
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType, MimeType};

#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<u32>,
    pub picture_data: Option<Vec<u8>>,
    pub thumbnail_data: Option<Vec<u8>>,
}

impl AudioFile {
    pub fn load(path: PathBuf) -> Option<Self> {
        let tagged_file_opt = Probe::open(&path).ok().and_then(|p| p.read().ok());
        let tag = tagged_file_opt.as_ref().and_then(|tf| tf.primary_tag().or_else(|| tf.first_tag()));

        if let Some(tag) = tag {
            let title = tag.title().as_deref()
                .or_else(|| path.file_stem().and_then(|s| s.to_str()))
                .unwrap_or("Unknown Title")
                .to_string();
            
            let picture_data = tag.pictures().first().map(|p| p.data().to_vec());

            let thumbnail_data = if let Some(data) = &picture_data {
                if let Ok(img) = image::load_from_memory(data) {
                     let thumbnail = img.resize_to_fill(40, 40, image::imageops::FilterType::Triangle);
                     let mut buf = Cursor::new(Vec::new());
                     if thumbnail.write_to(&mut buf, image::ImageOutputFormat::Png).is_ok() {
                         Some(buf.into_inner())
                     } else {
                         None
                     }
                } else {
                    None
                }
            } else {
                None
            };

            Some(Self {
                path,
                title,
                artist: tag.artist().as_deref().unwrap_or("Unknown Artist").to_string(),
                album: tag.album().as_deref().unwrap_or("Unknown Album").to_string(),
                year: tag.year(),
                picture_data,
                thumbnail_data,
            })
        } else {
            Some(Self {
                path: path.clone(),
                title: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("Unknown".to_string()),
                artist: "Unknown Artist".to_string(),
                album: "Unknown Album".to_string(),
                year: None,
                picture_data: None,
                thumbnail_data: None,
            })
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let mut tagged_file = Probe::open(&self.path)
            .map_err(|e| e.to_string())?
            .read()
            .map_err(|e| e.to_string())?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                tagged_file.first_tag_mut().ok_or("No writable tag found.")?
            }
        };

        tag.set_title(self.title.clone());
        tag.set_artist(self.artist.clone());
        tag.set_album(self.album.clone());
        
        if let Some(data) = &self.picture_data {
             let picture = Picture::new_unchecked(
                PictureType::CoverFront,
                Some(MimeType::Jpeg), 
                None,
                data.clone()
            );
            tag.push_picture(picture);
        }

        tagged_file.save_to_path(&self.path, WriteOptions::new()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn scan_folder(path: &Path) -> Vec<AudioFile> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext = ext.to_lowercase();
                    if ["mp3", "flac", "ogg", "m4a", "wav"].contains(&ext.as_str()) {
                        if let Some(audio_file) = AudioFile::load(path.clone()) {
                            files.push(audio_file);
                        }
                    }
                }
            }
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}
