use cosmic::app::{Core, Task};
use cosmic::iced::widget::svg;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
use cosmic::iced_runtime::core::window;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{self, text};
use cosmic::Element;

use crate::config::Config;
use crate::hotspot;

const APP_ID: &str = "io.github.reality2_roycdavies.cosmic-hotspot";

enum HotspotCommand {
    Toggle,
}

#[derive(Debug)]
enum HotspotEvent {
    StatusUpdate {
        active: bool,
        clients: Vec<String>,
    },
    ToggleStarted,
    ToggleComplete(Result<String, String>),
}

#[derive(Debug, Clone)]
pub enum Message {
    PollStatus,
    AnimationTick,
    ToggleHotspot,
    OpenSettings,
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
}

/// Number of animation frames in one ripple cycle.
const ANIM_FRAMES: u8 = 12;

pub struct HotspotApplet {
    core: Core,
    popup: Option<Id>,
    hotspot_active: bool,
    is_toggling: bool,
    status_message: String,
    /// Counts down from N to 0; while > 0, status_message is preserved (not overwritten by polls)
    status_hold_ticks: u8,
    connected_clients: Vec<String>,
    config: Config,
    cmd_tx: std::sync::mpsc::Sender<HotspotCommand>,
    event_rx: std::sync::mpsc::Receiver<HotspotEvent>,
    anim_frame: u8,
}

impl cosmic::Application for HotspotApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        let config = Config::load();
        // Save default config if it doesn't exist yet
        let _ = config.save();

        let initial_active = hotspot::is_hotspot_active(&config);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(run_background(cmd_rx, event_tx));
        });

        let applet = Self {
            core,
            popup: None,
            hotspot_active: initial_active,
            is_toggling: false,
            status_hold_ticks: 0,
            status_message: if initial_active {
                "Active".to_string()
            } else {
                "Inactive".to_string()
            },
            connected_clients: Vec::new(),
            config,
            cmd_tx,
            event_rx,
            anim_frame: 0,
        };

        (applet, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::AnimationTick => {
                if self.hotspot_active {
                    self.anim_frame = (self.anim_frame + 1) % ANIM_FRAMES;
                }
            }

            Message::PollStatus => {
                while let Ok(event) = self.event_rx.try_recv() {
                    match event {
                        HotspotEvent::StatusUpdate { active, clients } => {
                            self.hotspot_active = active;
                            self.connected_clients = clients;
                            // Reload config so popup reflects settings changes
                            self.config = Config::load();
                            if self.status_hold_ticks > 0 {
                                self.status_hold_ticks -= 1;
                            } else if !self.is_toggling {
                                self.status_message = if active {
                                    "Active".to_string()
                                } else {
                                    "Inactive".to_string()
                                };
                            }
                        }
                        HotspotEvent::ToggleStarted => {
                            self.is_toggling = true;
                            self.status_message = if self.hotspot_active {
                                "Stopping...".to_string()
                            } else {
                                "Starting...".to_string()
                            };
                        }
                        HotspotEvent::ToggleComplete(result) => {
                            self.is_toggling = false;
                            // Hold the result message for ~10 seconds (5 poll cycles at 2s)
                            self.status_hold_ticks = 5;
                            match result {
                                Ok(msg) => self.status_message = msg,
                                Err(e) => self.status_message = format!("Error: {e}"),
                            }
                        }
                    }
                }
            }

            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }

            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }

            Message::ToggleHotspot => {
                let _ = self.cmd_tx.send(HotspotCommand::Toggle);
                self.is_toggling = true;
                self.status_message = if self.hotspot_active {
                    "Stopping...".to_string()
                } else {
                    "Starting...".to_string()
                };
            }

            Message::OpenSettings => {
                std::thread::spawn(|| {
                    // Try unified settings hub first, fall back to standalone
                    let unified = std::process::Command::new("cosmic-applet-settings")
                        .arg(APP_ID)
                        .spawn();
                    if unified.is_err() {
                        let exe = std::env::current_exe()
                            .unwrap_or_else(|_| "cosmic-hotspot".into());
                        if let Err(e) = std::process::Command::new(exe)
                            .arg("--settings-standalone")
                            .spawn()
                        {
                            eprintln!("Failed to launch settings: {e}");
                        }
                    }
                });
            }
        }

        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let poll = cosmic::iced::time::every(std::time::Duration::from_secs(2))
            .map(|_| Message::PollStatus);

        if self.hotspot_active {
            // ~8 FPS ripple animation when active
            let anim = cosmic::iced::time::every(std::time::Duration::from_millis(125))
                .map(|_| Message::AnimationTick);
            cosmic::iced::Subscription::batch(vec![poll, anim])
        } else {
            poll
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let suggested = self.core.applet.suggested_size(true);
        let icon_size = suggested.0 as f32;

        let icon: Element<Message> = if self.hotspot_active {
            // Animated pulsating rings
            let svg_data = ripple_svg(self.anim_frame);
            let handle = svg::Handle::from_memory(svg_data.into_bytes());
            cosmic::iced::widget::svg(handle)
                .width(Length::Fixed(icon_size))
                .height(Length::Fixed(icon_size))
                .into()
        } else {
            widget::icon::from_name(
                "io.github.reality2_roycdavies.cosmic-hotspot-inactive-symbolic",
            )
            .symbolic(true)
            .into()
        };

        let have_popup = self.popup;
        let btn = self
            .core
            .applet
            .button_from_element(icon, true)
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<HotspotApplet>(
                        move |state: &mut HotspotApplet| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);

                            let popup_width = 280u32;
                            let popup_height = 300u32;

                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                Some((popup_width, popup_height)),
                                None,
                                None,
                            );
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            popup_settings
                        },
                        Some(Box::new(|state: &HotspotApplet| {
                            Element::from(state.core.applet.popup_container(
                                state.popup_content(),
                            ))
                            .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        let tooltip = if self.hotspot_active {
            "Hotspot (ON)"
        } else {
            "Hotspot (OFF)"
        };

        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            tooltip,
            self.popup.is_some(),
            |a| Message::Surface(a),
            None,
        ))
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl HotspotApplet {
    fn popup_content(&self) -> widget::Column<'_, Message> {
        use cosmic::iced::widget::{column, container, horizontal_space, row, Space};
        use cosmic::iced::{Alignment, Color};

        let title_row = row![
            text::body("WiFi Hotspot"),
            horizontal_space(),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let status_text = format!("Status: {}", self.status_message);
        let ssid_text = format!("SSID: {}", self.config.ssid);

        let info_section = column![
            text::body(status_text),
            text::caption(ssid_text),
        ]
        .spacing(2);

        // Connected clients section
        let mut clients_col = column![text::caption("Connected clients:")].spacing(2);
        if self.connected_clients.is_empty() {
            clients_col = clients_col.push(text::caption("  (none)"));
        } else {
            for client in &self.connected_clients {
                clients_col = clients_col.push(text::caption(format!("  {client}")));
            }
        }

        // Toggle row
        let toggle_label = if self.hotspot_active { "Hotspot" } else { "Hotspot" };
        let toggle_btn: Element<Message> = if self.is_toggling {
            widget::button::standard(if self.hotspot_active {
                "Stopping..."
            } else {
                "Starting..."
            })
            .into()
        } else if self.hotspot_active {
            widget::button::destructive("Turn Off")
                .on_press(Message::ToggleHotspot)
                .into()
        } else {
            widget::button::suggested("Turn On")
                .on_press(Message::ToggleHotspot)
                .into()
        };

        let toggle_row = row![
            text::body(toggle_label),
            horizontal_space(),
            toggle_btn,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let settings_row = row![
            horizontal_space(),
            widget::button::standard("Settings...").on_press(Message::OpenSettings),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let divider = || {
            container(Space::new(Length::Fill, Length::Fixed(1.0))).style(
                |theme: &cosmic::Theme| {
                    let cosmic = theme.cosmic();
                    container::Style {
                        background: Some(cosmic::iced::Background::Color(
                            Color::from(cosmic.palette.neutral_5),
                        )),
                        ..Default::default()
                    }
                },
            )
        };

        column![
            title_row,
            divider(),
            info_section,
            divider(),
            clients_col,
            divider(),
            toggle_row,
            divider(),
            settings_row,
        ]
        .spacing(8)
        .padding(12)
    }
}

async fn run_background(
    cmd_rx: std::sync::mpsc::Receiver<HotspotCommand>,
    event_tx: std::sync::mpsc::Sender<HotspotEvent>,
) {
    loop {
        // Check for commands from the UI
        if let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                HotspotCommand::Toggle => {
                    let _ = event_tx.send(HotspotEvent::ToggleStarted);

                    let config = Config::load();
                    let active = hotspot::is_hotspot_active(&config);

                    let result = if active {
                        hotspot::stop_hotspot(&config)
                    } else {
                        hotspot::start_hotspot(&config)
                    };

                    let _ = event_tx.send(HotspotEvent::ToggleComplete(result));
                }
            }
        }

        // Poll current status
        let config = Config::load();
        let active = hotspot::is_hotspot_active(&config);
        let clients = if active {
            hotspot::get_connected_clients(&config)
        } else {
            Vec::new()
        };

        let _ = event_tx.send(HotspotEvent::StatusUpdate { active, clients });

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

/// Generate an SVG string showing pulsating concentric rings.
/// Three rings ripple outward, each offset by 1/3 of the cycle.
fn ripple_svg(frame: u8) -> String {
    let mut rings = String::new();
    for phase in 0..3u8 {
        // Each ring is offset by 4 frames (1/3 of 12-frame cycle)
        let t = ((frame + phase * (ANIM_FRAMES / 3)) % ANIM_FRAMES) as f32
            / ANIM_FRAMES as f32;
        let r = 1.5 + t * 6.0; // radius: 1.5 → 7.5
        let opacity = 1.0 - t; // fades as it expands
        rings.push_str(&format!(
            r#"<circle cx="8" cy="8" r="{r:.1}" fill="none" stroke="currentColor" stroke-width="1.2" opacity="{opacity:.2}"/>"#,
        ));
    }
    // Center dot
    rings.push_str(r#"<circle cx="8" cy="8" r="1.2" fill="currentColor"/>"#);
    format!(
        r#"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg">{rings}</svg>"#
    )
}

pub fn run_applet() -> cosmic::iced::Result {
    cosmic::applet::run::<HotspotApplet>(())
}
