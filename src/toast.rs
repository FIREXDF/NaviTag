use iced::widget::{container, row, text};
use iced::{Element, Length, Theme};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub body: String,
    pub status: Status,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(status: Status, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            status,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
        }
    }

    pub fn expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

pub struct Manager {
    toasts: Vec<Toast>,
}

impl Default for Manager {
    fn default() -> Self {
        Self { toasts: Vec::new() }
    }
}

impl Manager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn update(&mut self) {
        self.toasts.retain(|t| !t.expired());
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let content = iced::widget::column(
            self.toasts
                .iter()
                .rev()
                .map(|toast| {
                    let icon = match toast.status {
                        Status::Success => "[OK]",
                        Status::Error => "[!]",
                        Status::Info => "(i)",
                    };

                    let color = match toast.status {
                        Status::Success => iced::Color::from_rgb(0.1, 0.8, 0.1),
                        Status::Error => iced::Color::from_rgb(0.8, 0.1, 0.1),
                        Status::Info => iced::Color::from_rgb(0.1, 0.1, 0.8),
                    };

                    container(
                        row![
                            text(icon).size(20).color(color),
                            iced::widget::column![
                                text(&toast.title).size(14).font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                                text(&toast.body).size(12)
                            ].spacing(2)
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                    )
                    .padding(10)
                    .style(move |theme: &Theme| container::Style {
                        background: Some(theme.palette().background.into()),
                        border: iced::border::Border {
                            color: color,
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        shadow: iced::Shadow {
                            color: iced::Color::BLACK,
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 10.0,
                        },
                        ..Default::default()
                    })
                    .width(300)
                    .into()
                })
                .collect::<Vec<_>>()
        )
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(20)
            .into()
    }
}
