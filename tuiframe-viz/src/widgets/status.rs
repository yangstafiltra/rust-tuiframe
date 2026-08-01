//! Status / navigation widgets: status_bar, breadcrumb_bar, hotkey_footer,
//! key_binding.

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

// ---- Status Bar ----

pub struct StatusBar;

impl Widget for StatusBar {
    fn name(&self) -> &'static str {
        "status_bar"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["normal", "insert", "visual", "command", "busy", "error"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (mode, mode_fg, mode_bg): (&str, Color, Color) = match variant {
            0 => (" NORMAL ", Color::Black, prims::palette(0)),
            1 => (" INSERT ", Color::Black, prims::palette(1)),
            2 => (" VISUAL ", Color::Black, prims::palette(2)),
            3 => (" COMMAND ", Color::Black, prims::palette(3)),
            4 => (" BUSY ", Color::Black, prims::palette(4)),
            _ => (" ERROR ", Color::Black, prims::palette(5)),
        };
        let bar_y = if area.height >= 4 { area.height.saturating_sub(3) } else { 0 };
        // Left: mode segment with a powerline slash.
        row_text_bg(buf, area, 0, bar_y, mode, mode_fg, mode_bg);
        let mut x = mode.chars().count() as u16;
        row_text_bg(buf, area, x, bar_y, "▌", mode_bg, Color::DarkGray);
        x += 1;
        // Left-middle: file path.
        let file = "src/widgets/status_bar.rs";
        row_text_bg(buf, area, x, bar_y, &format!(" {file} "), Color::White, Color::DarkGray);
        x += file.chars().count() as u16 + 3;
        // Right-middle: git branch.
        let git = " git:main±2 ";
        let git_len = git.chars().count() as u16;
        let git_x = area.width.saturating_sub(git_len).saturating_sub(2);
        if git_x > x {
            row_text_bg(buf, area, git_x, bar_y, git, Color::Black, prims::palette(1));
        }
        // Far right: encoding + a powerline tail.
        let enc = " UTF-8 ";
        let enc_len = enc.chars().count() as u16;
        let enc_x = area.width.saturating_sub(enc_len + git_len + 3);
        if enc_x > x {
            row_text_bg(buf, area, enc_x, bar_y, enc, Color::DarkGray, Color::Rgb(40, 40, 40));
        }
        row_text_bg(buf, area, area.width.saturating_sub(1), bar_y, "▐", Color::DarkGray, Color::Black);
    }
}

// ---- Breadcrumb Bar ----

pub struct BreadcrumbBar;

impl Widget for BreadcrumbBar {
    fn name(&self) -> &'static str {
        "breadcrumb_bar"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["project", "settings", "navigation", "filesystem", "git", "deep"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let crumbs: &[&str] = match variant {
            0 => &["my-app", "src", "engine", "widgets"],
            1 => &["Settings", "Appearance", "Theme"],
            2 => &["Home", "Docs", "Components", "Utility", "Breadcrumb"],
            3 => &["/", "home", "user", "code", "rust"],
            4 => &["repo", "feature/tui", "src"],
            5 => &["alpha", "beta", "gamma", "delta", "epsilon"],
            _ => &["fallback"],
        };
        let color = prims::palette(variant);
        let bar_y = if area.height >= 3 { area.height.saturating_sub(3) } else { 0 };
        row_text_bg(buf, area, 0, bar_y, " ▚ ", Color::Black, color);
        let mut x = 4u16;
        for (i, c) in crumbs.iter().enumerate() {
            if i > 0 {
                let sep = " / ";
                row_text(buf, area, x, bar_y, sep, prims::DIM);
                x += 3;
            }
            let style = if i == crumbs.len() - 1 {
                Style::new().fg(color).add_modifier(Modifier::BOLD)
            } else {
                WHITE
            };
            row_text(buf, area, x, bar_y, c, style);
            x += c.chars().count() as u16;
        }
    }
}

// ---- Hotkey Footer ----

pub struct HotkeyFooter;

impl Widget for HotkeyFooter {
    fn name(&self) -> &'static str {
        "hotkey_footer"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["default", "editing", "preview", "search", "debug", "full"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let hints: Vec<&str> = match variant {
            0 => vec!["q quit", "1-9 presets", "b bezier", "p palette"],
            1 => vec!["Esc cancel", "Tab complete", "Enter apply"],
            2 => vec!["←/→ switch", "b editor", "i data", "q quit"],
            3 => vec!["↑/↓ navigate", "Enter open", "Esc close"],
            4 => vec!["F5 refresh", "F9 breakpoint", "F10 step"],
            5 => vec!["q quit", "1-9 switch", "b bezier", "p palette", "i input", "r reset", "←/→ presets"],
            _ => vec!["?"],
        };
        let color = prims::palette(variant);
        let bar_y = if area.height >= 3 { area.height.saturating_sub(3) } else { 0 };
        row_text_bg(buf, area, 0, bar_y, "▚ ", Color::Black, color);
        let mut x = 3u16;
        for (i, h) in hints.iter().enumerate() {
            let (fg, bg) = if i % 2 == 0 {
                (Color::Black, color)
            } else {
                (Color::DarkGray, Color::Rgb(40, 40, 40))
            };
            row_text_bg(buf, area, x, bar_y, &format!(" {h} "), fg, bg);
            x += h.chars().count() as u16 + 2;
        }
    }
}

// ---- Key Binding ----

pub struct KeyBinding;

impl Widget for KeyBinding {
    fn name(&self) -> &'static str {
        "key_binding"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["modifier", "f-key", "chord", "two-key", "mouse", "plain"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (key, label, hint): (&str, &str, &str) = match variant {
            0 => ("Ctrl", "modifier", "Hold Ctrl"),
            1 => ("F12", "function key", "Runs debugger"),
            2 => ("Ctrl+Shift+P", "chord", "Command palette"),
            3 => ("g g", "two-key", "Go to top"),
            4 => ("Click", "mouse", "Open editor"),
            5 => ("q", "plain", "Quit"),
            _ => ("?", "unknown", ""),
        };
        let color = prims::palette(variant);
        let y = area.height / 2;
        // A keycap-style pill.
        let pill = format!(" ⟨ {key} ⟩ ");
        let x0 = area.width.saturating_sub(pill.chars().count() as u16) / 2;
        let _st = Style::new().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD);
        row_text_bg(buf, area, x0, y.saturating_sub(1), &pill, Color::Black, color);
        row_text(buf, area, area.width.saturating_sub(30) / 2, y, &format!("  {label}  "), WHITE);
        row_text(buf, area, area.width.saturating_sub(40) / 2, y + 1, hint, prims::DIM);
    }
}
