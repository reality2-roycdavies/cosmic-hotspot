//! Embeddable settings page for cosmic-hotspot
//!
//! Provides the settings UI as standalone State/Message/init/update/view
//! functions that can be embedded in cosmic-applet-settings or wrapped
//! in a standalone Application window.

use cosmic::iced::Length;
use cosmic::widget::{self, button, settings, text, text_input};
use cosmic::Element;

use crate::config::Config;
use crate::hotspot;

const BAND_OPTIONS: &[&str] = &["bg", "a"];
const BAND_LABELS: &[&str] = &["2.4 GHz (bg)", "5 GHz (a)"];

pub struct State {
    pub config: Config,
    pub status_message: String,
    pub selected_band_idx: usize,
    pub wifi_interfaces: Vec<String>,
    pub network_interfaces: Vec<String>,
    pub selected_hotspot_idx: Option<usize>,
    pub selected_internet_idx: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SsidChanged(String),
    PasswordChanged(String),
    HotspotInterfaceSelected(usize),
    InternetInterfaceSelected(usize),
    ConnectionNameChanged(String),
    GatewayIpChanged(String),
    BandSelected(usize),
    Save,
    ResetDefaults,
    RefreshInterfaces,
}

pub fn init() -> State {
    let config = Config::load();
    let selected_band_idx = BAND_OPTIONS
        .iter()
        .position(|&b| b == config.band)
        .unwrap_or(0);

    let wifi_interfaces = hotspot::list_wifi_interfaces();
    let network_interfaces = hotspot::list_network_interfaces();

    let selected_hotspot_idx = wifi_interfaces
        .iter()
        .position(|i| *i == config.hotspot_interface);
    let selected_internet_idx = network_interfaces
        .iter()
        .position(|i| *i == config.internet_interface);

    State {
        config,
        status_message: String::new(),
        selected_band_idx,
        wifi_interfaces,
        network_interfaces,
        selected_hotspot_idx,
        selected_internet_idx,
    }
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::SsidChanged(val) => {
            state.config.ssid = val;
            state.status_message = "Unsaved changes".to_string();
        }
        Message::PasswordChanged(val) => {
            state.config.password = val;
            state.status_message = "Unsaved changes".to_string();
        }
        Message::HotspotInterfaceSelected(idx) => {
            if idx < state.wifi_interfaces.len() {
                state.selected_hotspot_idx = Some(idx);
                state.config.hotspot_interface = state.wifi_interfaces[idx].clone();
                state.status_message = "Unsaved changes".to_string();
            }
        }
        Message::InternetInterfaceSelected(idx) => {
            if idx < state.network_interfaces.len() {
                state.selected_internet_idx = Some(idx);
                state.config.internet_interface = state.network_interfaces[idx].clone();
                state.status_message = "Unsaved changes".to_string();
            }
        }
        Message::ConnectionNameChanged(val) => {
            state.config.connection_name = val;
            state.status_message = "Unsaved changes".to_string();
        }
        Message::GatewayIpChanged(val) => {
            state.config.gateway_ip = val;
            state.status_message = "Unsaved changes".to_string();
        }
        Message::BandSelected(idx) => {
            if idx < BAND_OPTIONS.len() {
                state.selected_band_idx = idx;
                state.config.band = BAND_OPTIONS[idx].to_string();
                state.status_message = "Unsaved changes".to_string();
            }
        }
        Message::Save => {
            match state.config.save() {
                Ok(()) => state.status_message = "Settings saved".to_string(),
                Err(e) => state.status_message = format!("Error: {e}"),
            }
        }
        Message::ResetDefaults => {
            state.config = Config::default();
            state.selected_band_idx = 0;
            state.selected_hotspot_idx = state.wifi_interfaces
                .iter()
                .position(|i| *i == state.config.hotspot_interface);
            state.selected_internet_idx = state.network_interfaces
                .iter()
                .position(|i| *i == state.config.internet_interface);
            match state.config.save() {
                Ok(()) => state.status_message = "Reset to defaults and saved".to_string(),
                Err(e) => state.status_message = format!("Error: {e}"),
            }
        }
        Message::RefreshInterfaces => {
            state.wifi_interfaces = hotspot::list_wifi_interfaces();
            state.network_interfaces = hotspot::list_network_interfaces();
            state.selected_hotspot_idx = state.wifi_interfaces
                .iter()
                .position(|i| *i == state.config.hotspot_interface);
            state.selected_internet_idx = state.network_interfaces
                .iter()
                .position(|i| *i == state.config.internet_interface);
            state.status_message = format!(
                "Found {} WiFi, {} network interfaces",
                state.wifi_interfaces.len(),
                state.network_interfaces.len()
            );
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let page_title = text::title1("WiFi Hotspot Settings");

    let network_section = settings::section()
        .title("Network")
        .add(settings::item(
            "SSID",
            text_input("Network name", &state.config.ssid)
                .on_input(Message::SsidChanged)
                .width(Length::Fixed(250.0)),
        ))
        .add(settings::item(
            "Password",
            text_input("WPA2 password", &state.config.password)
                .on_input(Message::PasswordChanged)
                .width(Length::Fixed(250.0)),
        ))
        .add(settings::item(
            "Band",
            widget::dropdown(
                BAND_LABELS,
                Some(state.selected_band_idx),
                Message::BandSelected,
            )
            .width(Length::Fixed(250.0)),
        ));

    let hotspot_dropdown: Element<'_, Message> = if state.wifi_interfaces.is_empty() {
        text::caption("No WiFi interfaces found").into()
    } else {
        widget::dropdown(
            &state.wifi_interfaces,
            state.selected_hotspot_idx,
            Message::HotspotInterfaceSelected,
        )
        .width(Length::Fixed(250.0))
        .into()
    };

    let internet_dropdown: Element<'_, Message> = if state.network_interfaces.is_empty() {
        text::caption("No network interfaces found").into()
    } else {
        widget::dropdown(
            &state.network_interfaces,
            state.selected_internet_idx,
            Message::InternetInterfaceSelected,
        )
        .width(Length::Fixed(250.0))
        .into()
    };

    let interfaces_section = settings::section()
        .title("Interfaces")
        .add(settings::item("Hotspot interface", hotspot_dropdown))
        .add(settings::item("Internet interface", internet_dropdown))
        .add(settings::item_row(vec![
            button::standard("Refresh Devices")
                .on_press(Message::RefreshInterfaces)
                .into(),
        ]));

    let advanced_section = settings::section()
        .title("Advanced")
        .add(settings::item(
            "Connection name",
            text_input("NM connection name", &state.config.connection_name)
                .on_input(Message::ConnectionNameChanged)
                .width(Length::Fixed(250.0)),
        ))
        .add(settings::item(
            "Gateway IP",
            text_input("e.g. 192.168.44.1/24", &state.config.gateway_ip)
                .on_input(Message::GatewayIpChanged)
                .width(Length::Fixed(250.0)),
        ));

    let save_btn = button::suggested("Save")
        .on_press(Message::Save);

    let reset_btn = button::destructive("Reset to Defaults")
        .on_press(Message::ResetDefaults);

    let actions_section = settings::section()
        .title("Actions")
        .add(settings::item_row(vec![
            save_btn.into(),
            reset_btn.into(),
        ]));

    let mut content_items: Vec<Element<'_, Message>> = vec![
        page_title.into(),
        network_section.into(),
        interfaces_section.into(),
        advanced_section.into(),
        actions_section.into(),
    ];

    if !state.status_message.is_empty() {
        content_items.push(text::body(&state.status_message).into());
    }

    settings::view_column(content_items).into()
}
