//! Progress / loading widgets: loading_screen, auto_updater.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::prims::{self, WHITE};
use super::Widget;

fn row_text(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, style: Style) {
    let avail = area.width.saturating_sub(x);
    for (i, ch) in s.chars().take(avail as usize).enumerate() {
        let cx = area.x + x + i as u16;
        if cx < area.x + area.width && y < area.height {
            buf[(cx, area.y + y)].set_symbol(&ch.to_string()).set_style(style);
        }
    }
}

fn row_text_bg(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
    let st = Style::new().fg(fg).bg(bg);
    for (i, ch) in s.chars().take(area.width.saturating_sub(x) as usize).enumerate() {
        let cx = area.x + x + i as u16;
        if cx < area.x + area.width && y < area.height {
            buf[(cx, area.y + y)].set_symbol(&ch.to_string()).set_style(st);
        }
    }
}

const SPINNERS: [&str; 4] = ["◐", "◓", "◑", "◒"];

// ---- Loading Screen ----

pub struct LoadingScreen;

impl Widget for LoadingScreen {
    fn name(&self) -> &'static str {
        "loading_screen"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["boot", "compile", "download", "sync", "install", "shutdown"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let (title, msg): (&str, &str) = match variant {
            0 => ("Starting tuiframe…", "Loading component catalog"),
            1 => ("Compiling", "Optimizing release build"),
            2 => ("Downloading", "fetching tuiframe-viz"),
            3 => ("Syncing", "Pulling latest components"),
            4 => ("Installing", "copying files"),
            _ => ("Shutting down", "Cleaning up"),
        };
        // Fill the whole screen with a faint scrim.
        let color = prims::palette(variant);
        let cy = area.height.saturating_sub(10) / 2;
        let cx = area.width.saturating_sub(40) / 2;
        let boxr = Rect::new(cx, cy, 40, 9);
        prims::frame(buf, boxr, title, color);
        // Spinner.
        let spinner = SPINNERS[(tick as usize) % SPINNERS.len()];
        row_text(buf, boxr, 2, 2, spinner, Style::new().fg(color).add_modifier(Modifier::BOLD));
        row_text(buf, boxr, 6, 2, msg, WHITE);
        // Progress bar.
        let w = boxr.width.saturating_sub(6);
        let progress = (tick % 120) as f64 / 120.0;
        let filled = (w as f64 * progress) as u16;
        row_text(buf, boxr, 2, 4, &format!("{}", "█".repeat(filled as usize)), Style::new().fg(color));
        row_text(buf, boxr, 2 + filled, 4, &"░".repeat((w - filled) as usize), prims::DIM);
        row_text(buf, boxr, 2, 5, &format!("{:>3.0}%", progress * 100.0), WHITE);
    }
}

// ---- Auto Updater ----

pub struct AutoUpdater;

impl Widget for AutoUpdater {
    fn name(&self) -> &'static str {
        "auto_updater"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["checking", "available", "downloading", "installing", "done", "failed"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let color = prims::palette(variant);
        let cy = area.height.saturating_sub(14) / 2;
        let cx = area.width.saturating_sub(52) / 2;
        let boxr = Rect::new(cx, cy, 52, 13);
        prims::frame(buf, boxr, "Auto Updater", color);
        row_text(buf, boxr, 2, 2, "Current version:  1.2.0", WHITE);
        row_text(buf, boxr, 2, 3, "Latest version:   1.3.0", Style::new().fg(Color::LightGreen));
        match variant {
            0 => {
                row_text(buf, boxr, 2, 5, &format!("{} Checking for updates…", SPINNERS[(tick as usize) % SPINNERS.len()]), prims::DIM);
            }
            1 => {
                row_text(buf, boxr, 2, 5, "▲ Update available", Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD));
                row_text(buf, boxr, 2, 6, "  Changelog: fixed presets, faster renderer", prims::DIM);
                row_text_bg(buf, boxr, 2, 8, " [Update now] ", Color::Black, color);
                row_text(buf, boxr, 15, 8, " [Later] ", WHITE);
            }
            2 | 3 => {
                let w = boxr.width.saturating_sub(8);
                let progress = if variant == 2 {
                    (tick % 90) as f64 / 90.0
                } else {
                    (tick % 40) as f64 / 40.0
                };
                let filled = (w as f64 * progress) as u16;
                let verb = if variant == 2 { "Downloading" } else { "Installing" };
                row_text(buf, boxr, 2, 5, &format!("{verb}… {:.0}%", progress * 100.0), WHITE);
                row_text(buf, boxr, 2, 6, &"█".repeat(filled as usize), Style::new().fg(color));
                row_text(buf, boxr, 2 + filled, 6, &"░".repeat((w - filled) as usize), prims::DIM);
                if variant == 2 {
                    row_text(buf, boxr, 2, 8, &format!("  {} MB / 24 MB", ((progress * 24.0) as u16)), prims::DIM);
                }
            }
            4 => {
                row_text(buf, boxr, 2, 5, "✓ Update complete — restart to apply.", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD));
                row_text_bg(buf, boxr, 2, 8, " [Restart now] ", Color::Black, Color::LightGreen);
            }
            _ => {
                row_text(buf, boxr, 2, 5, "✕ Update failed: network unavailable.", Style::new().fg(Color::LightRed));
                row_text_bg(buf, boxr, 2, 8, " [Retry] ", Color::Black, color);
                row_text(buf, boxr, 10, 8, " [Cancel] ", WHITE);
            }
        }
    }
}
