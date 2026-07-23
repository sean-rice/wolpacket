use ipnet::Ipv4Net;
use macaddr::MacAddr6;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long a success status renders in green before fading to cyan.
const SUCCESS_FLASH_DURATION: Duration = Duration::from_millis(600);

/// A named device in the inventory.
#[derive(Clone, Debug)]
pub struct Device {
    /// Hyphen-separated primary key, e.g. `"sonos-move2"`.
    pub id: String,
    /// Human-readable label.
    pub name: String,
    pub mac: MacAddr6,
}

/// Application state.
pub struct App {
    pub devices: Vec<Device>,
    pub selected: usize,
    /// The last confirmed valid CIDR.
    lan: String,
    /// The in-progress edit buffer (only meaningful when `editing_lan` is true).
    lan_edit: String,
    /// Whether `lan_edit` currently parses as a valid `Ipv4Net`.
    lan_valid: bool,
    pub editing_lan: bool,
    pub status: String,
    /// When the current status was set.
    status_time: Instant,
    /// Whether the current status represents a successful action.
    status_success: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let default_lan = "192.168.103.0/24".into();
        Self {
            devices: vec![
                Device {
                    id: "sonos-move2".into(),
                    name: "Sonos Move 2".into(),
                    mac: "74:CA:60:27:82:0A".parse().unwrap(),
                },
                Device {
                    id: "office-pc".into(),
                    name: "Office PC (example)".into(),
                    mac: "AA:BB:CC:DD:EE:FF".parse().unwrap(),
                },
            ],
            selected: 0,
            lan: default_lan,
            lan_edit: String::new(),
            lan_valid: true,
            editing_lan: false,
            status: timestamped("Ready"),
            status_time: Instant::now(),
            status_success: false,
            should_quit: false,
        }
    }

    pub fn selected_device(&self) -> Option<&Device> {
        self.devices.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected = (self.selected + 1) % self.devices.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.devices.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.devices.len() - 1);
        }
    }

    /// Set a status message with a leading HH:MM:SS timestamp.
    /// If `success` is true, the status renders in green for a brief flash.
    pub fn set_status(&mut self, msg: impl Into<String>, success: bool) {
        self.status = timestamped(msg);
        self.status_time = Instant::now();
        self.status_success = success;
    }

    /// The color to use for the status line based on flash state.
    pub fn status_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        if self.status_success && self.status_time.elapsed() < SUCCESS_FLASH_DURATION {
            Color::Green
        } else {
            Color::Cyan
        }
    }

    /// The LAN string to display (edit buffer while editing, otherwise committed).
    pub fn lan_display(&self) -> &str {
        if self.editing_lan {
            &self.lan_edit
        } else {
            &self.lan
        }
    }

    /// Whether the currently displayed LAN string is a valid CIDR.
    pub fn is_lan_valid(&self) -> bool {
        if self.editing_lan {
            self.lan_valid
        } else {
            true // committed value is always valid
        }
    }

    /// The broadcast address derived from the committed LAN CIDR.
    ///
    /// `lan` is always a valid CIDR, so this never fails.
    pub fn broadcast_addr(&self) -> Ipv4Addr {
        self.lan.parse::<Ipv4Net>().unwrap().broadcast()
    }

    /// Begin editing the LAN CIDR.
    pub fn start_edit_lan(&mut self) {
        self.lan_edit = self.lan.clone();
        self.editing_lan = true;
    }

    /// Commit the edit buffer if it is a valid CIDR. Returns `true` if committed.
    pub fn confirm_edit_lan(&mut self) -> bool {
        if self.lan_valid {
            self.lan = self.lan_edit.clone();
            self.editing_lan = false;
            self.set_status(format!("LAN set to {}", self.lan), true);
            true
        } else {
            false
        }
    }

    /// Discard the edit buffer and revert to the committed CIDR.
    pub fn cancel_edit_lan(&mut self) {
        self.lan_edit.clear();
        self.editing_lan = false;
    }

    /// Push a character into the edit buffer.
    pub fn push_lan_char(&mut self, c: char) {
        self.lan_edit.push(c);
        self.lan_valid = self.lan_edit.parse::<Ipv4Net>().is_ok();
    }

    /// Delete the last character from the edit buffer.
    pub fn pop_lan_char(&mut self) {
        self.lan_edit.pop();
        self.lan_valid = self.lan_edit.parse::<Ipv4Net>().is_ok();
    }
}

/// Format a message with a leading `[HH:MM:SS]` timestamp.
fn timestamped(msg: impl Into<String>) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("[{:02}:{:02}:{:02}] {}", h, m, s, msg.into())
}
