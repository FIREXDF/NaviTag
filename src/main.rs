mod api;
mod audio;
mod toast;
mod settings;

use iced::widget::{button, checkbox, column, container, image as image_widget, row, scrollable, stack, text, text_input, vertical_space};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;
use std::time::{Duration, Instant};


pub fn main() -> iced::Result {
    iced::application("NaviTag - Music Tagger", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    TitleScreen,
    Editor,
}

struct App {
    current_page: Page,
    last_edit_time: Option<Instant>,
    has_unsaved_changes: bool,
    current_dir: Option<PathBuf>,
    files: Vec<audio::AudioFile>,
    selected_file_index: Option<usize>,
    search_query: String,
    search_results: Vec<api::MetadataResult>,
    search_images: Vec<Option<Vec<u8>>>,
    is_searching: bool,
    toast_manager: toast::Manager,
    settings: settings::UserSettings,
    show_settings: bool,
    
    show_exit_confirmation: bool,
    should_exit: bool,
    
    is_loading: bool,
    loading_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFolder,
    FolderPicked(Option<PathBuf>),
    FilesLoaded(Vec<audio::AudioFile>),
    FileSelected(usize),
    TitleChanged(String),
    ArtistChanged(String),
    AlbumChanged(String),
    SavePressed,
    SearchQueryChanged(String),
    SearchPressed,
    SearchResults(Result<Vec<api::MetadataResult>, String>),
    SearchCoverLoaded(usize, Result<Vec<u8>, String>),
    ApplyMetadata(api::MetadataResult),
    CoverDownloaded(Result<Vec<u8>, String>),
    SaveAll,
    
    CloseRequested,
    ConfirmExit(bool),
    CancelExit,
    
    Tick(Instant),
    SpotifyIdChanged(String),
    SpotifySecretChanged(String),
    ToggleSpotify(bool),
    BatchTag,
    BatchResults(Result<Vec<api::MetadataResult>, String>),
    ToggleSettings,
    SettingsChanged(settings::UserSettings),
    SaveSettings,
    SwitchToEditor,
    SwitchToTitle,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_page: Page::TitleScreen,
            last_edit_time: None,
            has_unsaved_changes: false,
            current_dir: None,
            files: Vec::new(),
            selected_file_index: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_images: Vec::new(),
            is_searching: false,
            toast_manager: toast::Manager::new(),
            settings: settings::UserSettings::load(),
            show_settings: false,

            show_exit_confirmation: false,
            should_exit: false,
            is_loading: false,
            loading_message: String::new(),
        }
    }
}

impl App {
    fn subscription(&self) -> iced::Subscription<Message> {
        let tick = if self.has_unsaved_changes {
             iced::time::every(Duration::from_millis(100)).map(Message::Tick)
        } else {
             iced::Subscription::none()
        };
        
        let events = iced::window::close_events().map(|_| Message::CloseRequested);

        iced::Subscription::batch(vec![tick, events])
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {

            Message::OpenFolder => {
                self.is_loading = true;
                self.loading_message = "Selecting folder...".to_string();
                Task::perform(pick_folder(), Message::FolderPicked)
            }
            Message::FolderPicked(Some(path)) => {
                self.current_dir = Some(path.clone());
                self.current_page = Page::Editor;
                self.loading_message = "Scanning files...".to_string();
                Task::perform(load_files(path), Message::FilesLoaded)
            }
            Message::FolderPicked(None) => {
                self.is_loading = false;
                Task::none()
            }
            Message::FilesLoaded(files) => {
                self.files = files;
                self.is_loading = false;
                self.selected_file_index = None;
                Task::none()
            }
            Message::SwitchToEditor => {
                self.current_page = Page::Editor;
                Task::none()
            }
            Message::SwitchToTitle => {
                self.current_page = Page::TitleScreen;
                Task::none()
            }
            Message::FileSelected(index) => {
                
                if self.has_unsaved_changes {
                    let _ = self.update(Message::SavePressed);
                }

                self.selected_file_index = Some(index);
                if let Some(file) = self.files.get(index) {
                     self.search_query = format!("{} {}", file.artist, file.title).trim().to_string();
                }
                Task::none()
            }
            Message::TitleChanged(val) => {
                if let Some(idx) = self.selected_file_index {
                    self.files[idx].title = val;
                    self.has_unsaved_changes = true;
                    self.last_edit_time = Some(Instant::now());
                }
                Task::none()
            }
            Message::ArtistChanged(val) => {
                if let Some(idx) = self.selected_file_index {
                    self.files[idx].artist = val;
                    self.has_unsaved_changes = true;
                    self.last_edit_time = Some(Instant::now());
                }
                Task::none()
            }
            Message::AlbumChanged(val) => {
                if let Some(idx) = self.selected_file_index {
                    self.files[idx].album = val;
                    self.has_unsaved_changes = true;
                    self.last_edit_time = Some(Instant::now());
                }
                Task::none()
            }
            Message::SavePressed => {
                if let Some(idx) = self.selected_file_index {
                    let file = &mut self.files[idx];
                    match file.save() {
                        Ok(_) => {
                             self.toast_manager.add(toast::Toast::new(
                                toast::Status::Success,
                                "Saved",
                                "File metadata updated successfully"
                            ));
                            self.has_unsaved_changes = false;
                            self.last_edit_time = None;
                        }
                        Err(e) => {
                             self.toast_manager.add(toast::Toast::new(
                                toast::Status::Error,
                                "Save Failed",
                                e
                            ));
                        }
                    }
                }
                Task::none()
            }
            Message::BatchTag => {
                if let Some(path) = &self.current_dir {
                    if let Some(folder_name) = path.file_name().and_then(|s| s.to_str()) {
                         self.is_searching = true;
                         self.is_loading = true;
                         self.loading_message = "Batch searching metadata...".to_string();
                         let query = folder_name.to_string();
                         let settings = self.settings.clone();
                         
                         Task::perform(async move {
                              Ok(api::search_all(query, settings).await)
                         }, Message::BatchResults)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::BatchResults(Ok(results)) => {
                self.is_searching = false;
                self.is_loading = false;
                if results.is_empty() {
                     self.toast_manager.add(toast::Toast::new(toast::Status::Info, "Batch Info", "No results found for batch tagging"));
                } else {
                     let count = std::cmp::min(self.files.len(), results.len());
                     for i in 0..count {
                         self.files[i].title = results[i].title.clone();
                         self.files[i].artist = results[i].artist.clone();
                         self.files[i].album = results[i].album.clone();
                     }
                      self.toast_manager.add(toast::Toast::new(
                          toast::Status::Success, 
                          "Batch Applied", 
                          format!("Applied metadata to {} files", count)
                      ));
                }
                Task::none()
            }
            Message::BatchResults(Err(e)) => {
                self.is_searching = false;
                self.is_loading = false;
                self.toast_manager.add(toast::Toast::new(toast::Status::Error, "Batch Error", e));
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            Message::SearchPressed => {
                if !self.search_query.is_empty() {
                    self.is_searching = true;
                    self.search_results.clear();
                    self.search_images.clear();
                    let query = self.search_query.clone();
                    let settings = self.settings.clone();
                    Task::perform(async move {
                         api::search_all(query, settings).await.into_iter().map(|r| r).collect::<Vec<_>>()
                    }, |res| Message::SearchResults(Ok(res)))
                } else {
                    Task::none()
                }
            }
            Message::SearchResults(Ok(results)) => {
                self.is_searching = false;
                self.search_results = results;
                self.search_images = vec![None; self.search_results.len()];

                if self.search_results.is_empty() {
                    self.toast_manager.add(toast::Toast::new(
                        toast::Status::Info,
                        "No Results",
                        "Try a different search term"
                    ));
                    Task::none()
                } else {
                    let tasks: Vec<Task<Message>> = self.search_results.iter().enumerate().filter_map(|(i, res)| {
                        res.cover_url.clone().map(|url| {
                             Task::perform(download_thumbnail(Some(url)), move |res| Message::SearchCoverLoaded(i, res))
                        })
                    }).collect();
                    
                    Task::batch(tasks)
                }
            }
            Message::SearchResults(Err(e)) => {
                self.is_searching = false;
                self.toast_manager.add(toast::Toast::new(
                     toast::Status::Error,
                     "Search Error",
                     e
                ));
                Task::none()
            }
            Message::SearchCoverLoaded(index, Ok(bytes)) => {
                if index < self.search_images.len() {
                    self.search_images[index] = Some(bytes);
                }
                Task::none()
            }
            Message::SearchCoverLoaded(_, Err(_)) => {  
                Task::none()
            }
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Task::none()
            }
            Message::SettingsChanged(settings) => {
                self.settings = settings;
                Task::none()
            }
            Message::SaveSettings => {
                self.settings.save();
                self.show_settings = false;
                self.toast_manager.add(toast::Toast::new(
                    toast::Status::Success,
                    "Settings Saved",
                    "Configuration updated"
                ));
                Task::none()
            }
            Message::SpotifyIdChanged(val) => {
                self.settings.spotify_id = val;
                Task::none()
            }
            Message::SpotifySecretChanged(val) => {
                self.settings.spotify_secret = val;
                Task::none()
            }
            Message::ToggleSpotify(val) => {
                self.settings.enable_spotify = val;
                Task::none()
            }
            Message::ApplyMetadata(meta) => {
                if let Some(idx) = self.selected_file_index {
                    self.files[idx].title = meta.title;
                    self.files[idx].artist = meta.artist;
                    self.files[idx].album = meta.album;
                    
                    return Task::perform(download_image(meta.cover_url), Message::CoverDownloaded);
                }
                Task::none()
            }
            Message::CoverDownloaded(Ok(bytes)) => {
                if let Some(idx) = self.selected_file_index {
                     self.files[idx].picture_data = Some(bytes);
                     self.toast_manager.add(toast::Toast::new(
                        toast::Status::Success,
                        "Cover Updated",
                        "New cover art downloaded and applied."
                    ));
                }
                Task::none()
            }
            Message::CoverDownloaded(Err(e)) => {
                  self.toast_manager.add(toast::Toast::new(
                     toast::Status::Error,
                     "Cover Error",
                     format!("Failed to download cover: {}", e)
                 ));
                  Task::none()
            }
            Message::SaveAll => self.perform_save_all(),

            Message::CloseRequested => {
                if self.has_unsaved_changes {
                    self.show_exit_confirmation = true;
                    Task::none()
                } else {
                    iced::window::get_latest().and_then(iced::window::close)
                }
            }
            Message::ConfirmExit(save) => {
                self.show_exit_confirmation = false;
                if save {
                    let _ = self.perform_save_all(); 
                     iced::window::get_latest().and_then(iced::window::close)
                } else {
                     iced::window::get_latest().and_then(iced::window::close)
                }
            }
            Message::CancelExit => {
                self.show_exit_confirmation = false;
                Task::none()
            }
            
            Message::Tick(_) => {
                 if self.has_unsaved_changes {
                     match self.last_edit_time {
                         Some(time) if time.elapsed() > Duration::from_secs(1) => {
                             return Task::done(Message::SavePressed);
                         }
                         _ => {}
                     }
                }
                Task::none()
            }

        }
    }


    fn perform_save_all(&mut self) -> Task<Message> {
        let mut success_count = 0;
        let mut error_count = 0;
        
        for file in &mut self.files {
            match file.save() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }

        if error_count == 0 && success_count > 0 {
             self.toast_manager.add(toast::Toast::new(
                toast::Status::Success,
                "All Saved",
                format!("Successfully saved {} files.", success_count)
            ));
        } else if error_count > 0 {
             self.toast_manager.add(toast::Toast::new(
                toast::Status::Error,
                "Save Errors",
                format!("Saved: {}, Failed: {}. Check file permissions.", success_count, error_count)
            ));
        }

        self.has_unsaved_changes = false;
        Task::none()
    }


    fn view(&self) -> Element<'_, Message> {
        let content = match self.current_page {
            Page::TitleScreen => {
                container(
                    column![
                         image_widget(image_widget::Handle::from_bytes(include_bytes!("logo.png").to_vec())).width(Length::Fixed(150.0)),
                         text("NaviTag").size(40).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                         vertical_space().height(20),
                         button("Open Folder").on_press(Message::OpenFolder).padding(15).width(Length::Fixed(200.0)),
                         button("Settings").on_press(Message::ToggleSettings).padding(15).width(Length::Fixed(200.0)),
                    ]
                    .align_x(iced::Alignment::Center)
                    .spacing(20)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                     background: Some(iced::Color::from_rgb(0.1, 0.1, 0.1).into()),
                     ..Default::default()
                })
                .into()
            }
            Page::Editor => {    
                let file_list_header = text(if let Some(path) = &self.current_dir {
                    format!("Files in: {}", path.file_name().unwrap_or_default().to_string_lossy())
                } else {
                    "No folder open".to_string()
                }).size(18).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

                        let file_list_content = column(
                    self.files.iter().enumerate().map(|(i, f)| {
                        let is_selected = Some(i) == self.selected_file_index;
                        
                        let thumb: Element<Message> = if let Some(data) = &f.thumbnail_data {
                             image_widget(image_widget::Handle::from_bytes(data.clone())).width(Length::Fixed(40.0)).height(Length::Fixed(40.0)).into()
                        } else {
                             container(text("?").size(20))
                                .width(Length::Fixed(40.0))
                                .height(Length::Fixed(40.0))
                                .align_x(iced::alignment::Horizontal::Center)
                                .align_y(iced::alignment::Vertical::Center)
                                .style(|_theme: &Theme| container::Style {
                                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                                    ..Default::default()
                                })
                                .into()
                        };

                        let content = row![
                            thumb,
                            column![
                                text(&f.title).size(14).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                                text(&f.artist).size(12).color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                            ].spacing(2)
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center);

                        button(content)
                            .on_press(Message::FileSelected(i))
                            .width(Length::Fill)
                            .padding(10)
                            .style(move |theme: &Theme, status| {
                                let palette = theme.palette();
                                if is_selected {
                                     button::Style {
                                        background: Some(palette.primary.into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::border::Border { radius: 8.0.into(), ..Default::default() },
                                        ..Default::default()
                                     }
                                } else {
                                     button::Style {
                                        background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
                                        text_color: palette.text,
                                        border: iced::border::Border { radius: 8.0.into(), ..Default::default() },
                                        ..Default::default()
                                     }
                                }
                            })
                            .into()
                    })
                    .collect::<Vec<_>>()
                )
                .spacing(8)
                .height(Length::Shrink);

                let file_list = scrollable(file_list_content).height(Length::Fill);

                let left_panel = container(
                    column![
                        file_list_header,
                        button("Open Folder").on_press(Message::OpenFolder).width(Length::Fill),
                        button("Back to Title").on_press(Message::SwitchToTitle).width(Length::Fill),
                        button("Save All").on_press(Message::SaveAll).width(Length::Fill).style(|_theme, status| {
                              button::Style {
                                 background: Some(iced::Color::from_rgb(0.2, 0.6, 0.2).into()),
                                 text_color: iced::Color::WHITE,
                                 border: iced::border::Border { radius: 5.0.into(), ..Default::default() },
                                 ..Default::default()
                              }
                        }),
                        file_list
                    ]
                    .spacing(10)
                    .height(Length::Fill)
                )
                .padding(15)
                .width(Length::FillPortion(1))
                .style(|theme: &Theme| container::Style {
                    background: Some(theme.palette().background.into()),
                    border: iced::border::Border { color: theme.palette().text, width: 1.0, radius: 5.0.into() },
                    ..Default::default()
                });

                let editor_content = if let Some(idx) = self.selected_file_index {
                    let file = &self.files[idx];
                    
                    let image_preview: Element<Message> = if let Some(data) = &file.picture_data {
                         image_widget(image_widget::Handle::from_bytes(data.clone())).width(Length::Fixed(200.0)).height(Length::Fixed(200.0)).into()
                    } else {
                         container(text("No Cover Art").size(20))
                            .width(Length::Fixed(200.0))
                            .height(Length::Fixed(200.0))
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .style(|_theme: &Theme| container::Style {
                                background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                                ..Default::default()
                            })
                            .into()
                    };

                    column![
                        text(format!("Editing: {}", file.path.file_name().unwrap().to_string_lossy())).size(20).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                        
                        row![
                            image_preview,
                            column![
                                 text("Title").size(12),
                                 text_input("Title", &file.title).on_input(Message::TitleChanged).padding(10),
                                 
                                 text("Artist").size(12),
                                 text_input("Artist", &file.artist).on_input(Message::ArtistChanged).padding(10),
                                 
                                 text("Album").size(12),
                                 text_input("Album", &file.album).on_input(Message::AlbumChanged).padding(10),
                            ].spacing(10).width(Length::Fill)
                        ].spacing(20),

                        button(if self.has_unsaved_changes { "Saving..." } else { "Saved" })
                            .on_press(Message::SavePressed)
                            .padding(10)
                            .width(Length::Fill)
                            .style(move |theme: &Theme, status| {
                                if self.has_unsaved_changes {
                                     button::primary(theme, status)
                                } else {
                                     button::success(theme, status)
                                }
                             })
                    ].spacing(20)
                } else {
                    column![
                        text("Select a file to start editing").size(24),
                        text("Open a folder to see your music files.").size(16)
                    ].spacing(20).align_x(iced::Alignment::Center)
                };

                let editor_panel = container(editor_content)
                .width(Length::FillPortion(2))
                .padding(20)
                .style(|_theme: &Theme| container::Style {
                    background: Some(_theme.palette().background.into()),
                    border: iced::border::Border { color: _theme.palette().text, width: 1.0, radius: 5.0.into() },
                    ..Default::default()
                });

                let search_input = text_input("Search Artist/Album...", &self.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .on_submit(Message::SearchPressed)
                    .padding(10);
                
                let search_results_list = scrollable(
                    column(
                        self.search_results.iter().enumerate().map(|(i, res)| {
                            let info = format!("{} - {}\n{}", res.artist, res.title, res.album);
                            let source = format!("Source: {}", res.source);
                            
                            let image_preview: Element<Message> = if let Some(Some(data)) = self.search_images.get(i) {
                                 image_widget(image_widget::Handle::from_bytes(data.clone())).width(Length::Fixed(50.0)).height(Length::Fixed(50.0)).into()
                            } else {
                                 container(text("?").size(20))
                                    .width(Length::Fixed(50.0))
                                    .height(Length::Fixed(50.0))
                                    .center_x(Length::Fill)
                                    .center_y(Length::Fill)
                                    .style(|_theme: &Theme| container::Style {
                                        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                                        ..Default::default()
                                    })
                                    .into()
                            };

                            container(
                                row![
                                    image_preview,
                                    column![
                                        text(info).size(12).width(Length::Fill),
                                        text(source).size(10).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ].width(Length::Fill).spacing(5),
                                    button("Apply").on_press(Message::ApplyMetadata(res.clone())).padding(5)
                                ]
                                .align_y(iced::Alignment::Center)
                                .spacing(10)
                            )
                            .padding(5)
                            .style(|_theme: &Theme| container::Style {
                                 background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
                                 border: iced::border::Border {
                                     color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                                     width: 1.0,
                                     radius: 3.0.into(),
                                 },
                                 ..Default::default()
                            })
                            .into()
                        }).collect::<Vec<_>>()
                    )
                    .spacing(10)
                    .height(Length::Shrink)
                ).height(Length::Fill);

                let right_panel = container(
                    column![
                        row![
                             text("Online Search").size(20).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }).width(Length::Fill),
                             button("Settings").on_press(Message::ToggleSettings).padding(5)
                        ].align_y(iced::Alignment::Center),

                        row![search_input, button("Go").on_press(Message::SearchPressed).padding(10)].spacing(10),
                        
                        if self.is_searching { text("Searching...") } else { text("") },
                        
                        button("Batch Tag (Folder)").on_press(Message::BatchTag).padding(10).width(Length::Fill),

                        search_results_list
                    ]
                    .spacing(20)
                )
                .padding(15)
                .width(Length::FillPortion(1))
                .style(|_theme: &Theme| container::Style {
                    background: Some(_theme.palette().background.into()),
                    border: iced::border::Border { color: _theme.palette().text, width: 1.0, radius: 5.0.into() },
                    ..Default::default()
                });

                row![left_panel, editor_panel, right_panel]
                    .spacing(10)
                    .padding(10)
                    .height(Length::Fill)
                    .into()
            }
        };
        
        let mut layers = vec![content];

        if self.show_settings {
             let settings_modal = Element::from(container(
                 column![
                     text("Settings").size(24).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                     
                     text("Apple Music").size(16).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                     checkbox("Enable Apple Music Search", self.settings.enable_apple_music)
                         .on_toggle(|v| Message::SettingsChanged(settings::UserSettings { enable_apple_music: v, ..self.settings.clone() })),
                     
                     text("Spotify").size(16).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                     checkbox("Enable Spotify Search", self.settings.enable_spotify)
                         .on_toggle(|v| Message::SettingsChanged(settings::UserSettings { enable_spotify: v, ..self.settings.clone() })),
                     
                     text("Client ID").size(12),
                     text_input("Client ID", &self.settings.spotify_id)
                         .on_input(|v| Message::SettingsChanged(settings::UserSettings { spotify_id: v, ..self.settings.clone() })),
                     text("Client Secret").size(12),
                     text_input("Client Secret", &self.settings.spotify_secret)
                         .on_input(|v| Message::SettingsChanged(settings::UserSettings { spotify_secret: v, ..self.settings.clone() })),
                    
                     text("Genius").size(16).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                     checkbox("Enable Genius Search", self.settings.enable_genius)
                         .on_toggle(|v| Message::SettingsChanged(settings::UserSettings { enable_genius: v, ..self.settings.clone() })),
                     text("Access Token").size(12),
                     text_input("Genius Access Token", &self.settings.genius_token)
                         .on_input(|v| Message::SettingsChanged(settings::UserSettings { genius_token: v, ..self.settings.clone() }))
                         .secure(true),

                     text("Last.fm").size(16).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                     checkbox("Enable Last.fm Search", self.settings.enable_lastfm)
                         .on_toggle(|v| Message::SettingsChanged(settings::UserSettings { enable_lastfm: v, ..self.settings.clone() })),
                     text("API Key").size(12),
                     text_input("Last.fm API Key", &self.settings.lastfm_api_key)
                         .on_input(|v| Message::SettingsChanged(settings::UserSettings { lastfm_api_key: v, ..self.settings.clone() }))
                         .secure(true),

                     row![
                         button("Save & Close").on_press(Message::SaveSettings).padding(10),
                         button("Cancel").on_press(Message::ToggleSettings).padding(10)
                     ].spacing(10)
                 ]
                 .spacing(10)
                 .padding(20)
             )
             .style(|_theme: &Theme| container::Style {
                 background: Some(_theme.palette().background.into()),
                 border: iced::border::Border { color: _theme.palette().text, width: 1.0, radius: 10.0.into() },
                 shadow: iced::Shadow { color: iced::Color::BLACK, offset: iced::Vector::new(0.0, 5.0), blur_radius: 20.0 },
                 ..Default::default()
             })
             .width(Length::Fill)
             .height(Length::Fill)
             .center_x(Length::Fill)
             .center_y(Length::Fill)
             .style(|_theme: &Theme| container::Style {
                 background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                 ..Default::default()
             }));
            
            layers.push(settings_modal);
        }

        if self.show_exit_confirmation {
            let overlay = Element::from(container(
                column![
                    text("Unsaved Changes").size(24).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                    text("You have unsaved changes. Do you want to save before quitting?").size(16),
                    row![
                        button("Save & Quit").on_press(Message::ConfirmExit(true)).padding(10).style(|_theme, _status| button::Style {
                            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.2).into()),
                            text_color: iced::Color::WHITE,
                            border: iced::border::Border { radius: 5.0.into(), ..Default::default() },
                            ..Default::default()
                        }),
                        button("Quit without Saving").on_press(Message::ConfirmExit(false)).padding(10).style(|_theme, _status| button::Style {
                            background: Some(iced::Color::from_rgb(0.8, 0.2, 0.2).into()),
                            text_color: iced::Color::WHITE,
                            border: iced::border::Border { radius: 5.0.into(), ..Default::default() },
                            ..Default::default()
                        }),
                        button("Cancel").on_press(Message::CancelExit).padding(10).style(|_theme, _status| button::Style {
                            background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                            text_color: iced::Color::WHITE,
                            border: iced::border::Border { radius: 5.0.into(), ..Default::default() },
                            ..Default::default()
                        }),
                    ].spacing(20)
                ]
                .spacing(20)
                .padding(30)
                .align_x(iced::Alignment::Center)
            )
            .style(|_theme: &Theme| container::Style {
                 background: Some(_theme.palette().background.into()),
                 border: iced::border::Border { color: _theme.palette().text, width: 1.0, radius: 10.0.into() },
                 shadow: iced::Shadow { color: iced::Color::BLACK, offset: iced::Vector::new(0.0, 5.0), blur_radius: 20.0 },
                 ..Default::default()
             })
             .width(Length::Fill)
             .height(Length::Fill)
             .center_x(Length::Fill)
             .center_y(Length::Fill)
             .style(|_theme: &Theme| container::Style {
                 background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.8).into()),
                 ..Default::default()
             }));
             layers.push(overlay);
        }

        if self.is_loading {
             let overlay = Element::from(container(
                 column![
                     text("Loading...").size(24).style(|_theme: &Theme| text::Style { color: Some(iced::Color::WHITE) }),
                     text(&self.loading_message).size(16).style(|_theme: &Theme| text::Style { color: Some(iced::Color::WHITE) })
                 ]
                 .spacing(10)
                 .align_x(iced::Alignment::Center)
             )
             .width(Length::Fill)
             .height(Length::Fill)
             .center_x(Length::Fill)
             .center_y(Length::Fill)
             .style(|_theme: &Theme| container::Style {
                 background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
                 ..Default::default()
             }));
             layers.push(overlay);
        }
        
        stack(vec![
            stack(layers).into(),
            self.toast_manager.view()
        ]).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new().pick_folder().await.map(|h| h.path().to_path_buf())
}

async fn load_files(path: PathBuf) -> Vec<audio::AudioFile> {
    tokio::task::spawn_blocking(move || audio::scan_folder(&path))
        .await
        .unwrap_or_default()
}

async fn perform_search(query: String) -> Result<Vec<api::MetadataResult>, String> {
    api::apple_music::search(&query).await
}

async fn download_image(url: Option<String>) -> Result<Vec<u8>, String> {
    if let Some(url) = url {
        let bytes = reqwest::get(&url).await.map_err(|e| e.to_string())?
            .bytes().await.map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    } else {
        Err("No URL provided".to_string())
    }
}

async fn download_thumbnail(url: Option<String>) -> Result<Vec<u8>, String> {
     if let Some(url) = url {
        let bytes = reqwest::get(&url).await.map_err(|e| e.to_string())?
            .bytes().await.map_err(|e| e.to_string())?
            .to_vec();

        tokio::task::spawn_blocking(move || {
            let img = image::load_from_memory(&bytes).map_err(|e: image::ImageError| e.to_string())?;
            let thumbnail = img.resize_to_fill(50, 50, image::imageops::FilterType::Triangle);
            
            let mut buf = std::io::Cursor::new(Vec::new());
            thumbnail.write_to(&mut buf, image::ImageOutputFormat::Png)
                .map_err(|e: image::ImageError| e.to_string())?;
            
            Ok::<Vec<u8>, String>(buf.into_inner())
        }).await.map_err(|e| format!("Task join error: {}", e))?
    } else {
        Err("No URL provided".to_string())
    }
}
