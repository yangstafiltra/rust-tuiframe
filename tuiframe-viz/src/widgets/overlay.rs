//! Overlay / popup widgets: popup, dialog, context_menu, floating_palette,
//! spotlight, onboarding_tip, tutorial_step, theme_picker.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::prims::{self, WHITE};
use super::Widget;

/// Clear a box (transparent fill) then draw a bordered frame.
pub(super) fn centered_box(buf: &mut Buffer, area: Rect, w: u16, h: u16, title: &str, color: Color) -> Rect {
    if w == 0 || h == 0 {
        return Rect::new(area.x, area.y, 0, 0);
    }
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let boxr = Rect::new(x, y, w.min(area.width), h.min(area.height));
    // Clear under the box so it reads as a modal overlay.
    for yy in boxr.y..boxr.y.saturating_add(boxr.height) {
        for xx in boxr.x..boxr.x.saturating_add(boxr.width) {
            if xx < area.width && yy < area.height {
                buf[(xx, yy)].set_symbol(" ").reset();
            }
        }
    }
    prims::frame(buf, boxr, title, color);
    boxr
}

/// Dim the area outside a centered box (a scrim), leaving `boxr` bright.
pub(super) fn scrim(buf: &mut Buffer, area: Rect, boxr: Rect) {
    let st = Style::new().fg(Color::Black);
    for y in 0..area.height {
        for x in 0..area.width {
            let inside = x >= boxr.x && x < boxr.x.saturating_add(boxr.width) && y >= boxr.y && y < boxr.y.saturating_add(boxr.height);
            if !inside {
                let c = &mut buf[(area.x + x, area.y + y)];
                let sym = c.symbol().to_string();
                if sym != " " {
                    c.set_style(st);
                }
            }
        }
    }
}

fn row_text(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, style: Style) {
    let avail = area.width.saturating_sub(x);
    for (i, ch) in s.chars().take(avail as usize).enumerate() {
        let cx = area.x + x + i as u16;
        if cx < area.x + area.width && y < area.height {
            buf[(cx, area.y + y)].set_symbol(&ch.to_string()).set_style(style);
        }
    }
}

fn btn(buf: &mut Buffer, area: Rect, x: u16, y: u16, label: &str, active: bool) -> u16 {
    let st = if active {
        Style::new().fg(Color::Black).bg(prims::palette(0)).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(prims::palette(0))
    };
    row_text(buf, area, x, y, &format!("[{label}]"), st);
    label.chars().count() as u16 + 2
}

// ---- Popup ----

pub struct Popup;

impl Widget for Popup {
    fn name(&self) -> &'static str {
        "popup"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["message", "input", "progress", "confirm", "info", "custom"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let color = prims::palette(variant);
        let (title, body): (&str, &[&str]) = match variant {
            0 => ("Message", &["Changes saved successfully.", "", "Your work is up to date."]),
            1 => ("Input", &["Project name:", ""]),
            2 => ("Progress", &["Installing components…", ""]),
            3 => ("Confirm", &["Delete this item permanently?"]),
            4 => ("Info", &["Version 1.2.0 is available."]),
            _ => ("Custom", &["Anything you like, in a modal box."]),
        };
        let h = 9u16;
        let boxr = centered_box(buf, area, 46, h, title, color);
        scrim(buf, area, boxr);
        let inner = Rect::new(boxr.x + 2, boxr.y + 2, boxr.width.saturating_sub(4), boxr.height.saturating_sub(4));
        for (i, line) in body.iter().enumerate() {
            row_text(buf, inner, 0, i as u16, line, WHITE);
        }
        let buttons = ["OK", "Cancel"];
        let mut bx = inner.x;
        let by = inner.y + inner.height - 2;
        for (i, b) in buttons.iter().enumerate() {
            bx += btn(buf, inner, bx - inner.x, by - inner.y, b, i == 0);
            bx += 2;
        }
        if variant == 1 {
            row_text(buf, inner, 0, 2, "▍my_app        ", Style::new().fg(Color::Black).bg(prims::palette(1)));
        }
        if variant == 2 {
            let w = inner.width.saturating_sub(4);
            let filled = (w as f64 * 0.58) as u16;
            row_text(buf, inner, 0, 2, &format!("{}█", "█".repeat(filled as usize)), Style::new().fg(prims::palette(1)));
            row_text(buf, inner, filled + 1, 2, "░░░░ 58%", WHITE);
        }
    }
}

// ---- Dialog ----

pub struct Dialog;

impl Widget for Dialog {
    fn name(&self) -> &'static str {
        "dialog"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["info", "success", "warning", "error", "confirm", "critical"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let (color, icon, title, body, btn_a, btn_b): (Color, &str, &str, &str, &str, &str) = match variant {
            0 => (Color::Blue, "ⓘ", "Information", "Operation completed.", "OK", ""),
            1 => (Color::LightGreen, "✓", "Success", "All systems nominal.", "Done", ""),
            2 => (Color::Yellow, "⚠", "Warning", "Disk space is low.", "OK", "Ignore"),
            3 => (Color::Red, "✕", "Error", "Could not connect to server.", "Retry", "Cancel"),
            4 => (Color::Cyan, "?", "Confirm", "Publish these changes?", "Publish", "Cancel"),
            _ => (Color::LightMagenta, "!", "Critical", "Restart required.", "Restart", "Later"),
        };
        let h = 9u16;
        let boxr = centered_box(buf, area, 50, h, title, color);
        scrim(buf, area, boxr);
        let inner = Rect::new(boxr.x + 2, boxr.y + 2, boxr.width.saturating_sub(4), boxr.height.saturating_sub(4));
        row_text(buf, inner, 0, 1, icon, Style::new().fg(color).add_modifier(Modifier::BOLD));
        row_text(buf, inner, 4, 1, body, WHITE);
        let mut bx = inner.x + inner.width.saturating_sub(20);
        let by = inner.y + inner.height - 2;
        bx += btn(buf, inner, bx - inner.x, by - inner.y, btn_a, true);
        if !btn_b.is_empty() {
            bx += 2;
            btn(buf, inner, bx - inner.x, by - inner.y, btn_b, false);
        }
    }
}

// ---- Context Menu ----

pub struct ContextMenu;

impl Widget for ContextMenu {
    fn name(&self) -> &'static str {
        "context_menu"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["edit", "tabs", "view", "selection", "send", "sort"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let items: &[(&str, &str)] = match variant {
            0 => &[("Cut", "Ctrl+X"), ("Copy", "Ctrl+C"), ("Paste", "Ctrl+V"), ("Delete", "Del"), ("", ""), ("Select All", "Ctrl+A")],
            1 => &[("Close Tab", "Ctrl+W"), ("New Tab", "Ctrl+T"), ("Reopen Closed", "Ctrl+Shift+T"), ("", ""), ("Pin Tab", "")],
            2 => &[("Zoom In", "Ctrl+="), ("Zoom Out", "Ctrl+-"), ("Fullscreen", "F11"), ("", ""), ("Toggle Sidebar", "Ctrl+B")],
            3 => &[("Copy", "Ctrl+C"), ("Copy Path", ""), ("", ""), ("Rename", "F2"), ("Delete", "Del")],
            4 => &[("Reply", "R"), ("Forward", "F"), ("", ""), ("Mark Read", ""), ("Archive", "E")],
            _ => &[("Sort by Name", ""), ("Sort by Size", ""), ("Sort by Date", ""), ("", ""), ("Reverse Order", "")],
        };
        let w = 28u16;
        let h = (items.len() as u16).min(10) + 2;
        let bx = area.x + area.width.saturating_sub(w).saturating_sub(4);
        let by = area.y + 3;
        let boxr = Rect::new(bx, by, w, h);
        prims::frame(buf, boxr, "Context Menu", prims::palette(variant));
        let mut ly = boxr.y + 1;
        for (i, (label, key)) in items.iter().enumerate() {
            if ly >= boxr.y + boxr.height - 1 {
                break;
            }
            if label.is_empty() {
                row_text(buf, boxr, 1, ly - boxr.y, "─────────────", prims::DIM);
            } else {
                let active = i == 1;
                let style = if active {
                    Style::new().fg(Color::Black).bg(prims::palette(variant)).add_modifier(Modifier::BOLD)
                } else {
                    WHITE
                };
                row_text(buf, boxr, 1, ly - boxr.y, label, style);
                row_text(buf, boxr, 1 + 14, ly - boxr.y, key, prims::DIM);
            }
            ly += 1;
        }
    }
}

// ---- Floating Palette ----

pub struct FloatingPalette;

impl Widget for FloatingPalette {
    fn name(&self) -> &'static str {
        "floating_palette"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["default", "dark", "accent", "compact", "wide", "bottom"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let actions: &[&str] = match variant {
            0 => &["New File", "Open", "Save", "Run", "Search", "Settings"],
            1 => &["Terminal", "Explorer", "Git", "Debug", "Extensions"],
            2 => &["Build", "Test", "Deploy", "Lint", "Format"],
            3 => &["Copy", "Paste", "Cut", "Undo"],
            4 => &["Aligner", "Formatter", "Linter", "Refactor", "Docs"],
            _ => &["Home", "About", "Help", "Quit"],
        };
        let color = prims::palette(variant + 2);
        let bottom = variant == 5;
        let w = (actions.len() as u16 * 14).min(area.width.saturating_sub(6));
        let h = 5u16;
        let bx = area.x + area.width.saturating_sub(w) / 2;
        let by = if bottom {
            area.y + area.height.saturating_sub(h).saturating_sub(2)
        } else {
            area.y + 2
        };
        let boxr = Rect::new(bx, by, w, h);
        prims::frame(buf, boxr, "Actions", color);
        let mut cx = boxr.x + 1;
        for (i, a) in actions.iter().enumerate() {
            let active = i == 0;
            let style = if active {
                Style::new().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
            } else {
                WHITE
            };
            row_text(buf, boxr, cx - boxr.x, 2, &format!(" {a} "), style);
            cx += a.chars().count() as u16 + 2;
            if cx - boxr.x > w - 2 {
                break;
            }
        }
    }
}

// ---- Spotlight ----

pub struct Spotlight;

impl Widget for Spotlight {
    fn name(&self) -> &'static str {
        "spotlight"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["commands", "files", "symbols", "settings", "actions", "recent"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (prompt, results): (&str, &[&str]) = match variant {
            0 => ("Run a command…", &["b  Toggle bezier editor", "i  Input custom data", "p  Cycle palette", "q  Quit"]),
            1 => (&"Open file… ", &["src/engine.rs", "src/widgets/overlay.rs", "Cargo.toml", "components/viz/heatmap.toml"]),
            2 => (&"Go to symbol… ", &["fn run_widget", "struct Spotlight", "trait Widget", "fn make"]),
            3 => (&"Search settings… ", &["Theme: nord", "Font size: 12", "Tab width: 4", "Line numbers: on"]),
            4 => (&"What can I do? ", &["Preview a component", "List categories", "Validate deps", "Scaffold a project"]),
            _ => (&"Recent… ", &["tuiframe preview heatmap", "tuiframe list --json", "cargo test --workspace", "tuiframe preview status_bar"]),
        };
        let color = prims::palette(variant);
        let w = 56u16.min(area.width.saturating_sub(8));
        let h = (results.len() as u16 + 5).min(area.height.saturating_sub(4));
        let boxr = centered_box(buf, area, w, h, "Spotlight", color);
        scrim(buf, area, boxr);
        let inner = Rect::new(boxr.x + 2, boxr.y + 2, boxr.width.saturating_sub(4), boxr.height.saturating_sub(4));
        row_text(buf, inner, 0, 0, &format!("❯ {prompt}"), WHITE);
        for (i, r) in results.iter().enumerate() {
            let active = i == 0;
            let style = if active {
                Style::new().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
            } else {
                WHITE
            };
            row_text(buf, inner, 1, 2 + i as u16, r, style);
        }
    }
}

// ---- Onboarding Tip ----

pub struct OnboardingTip;

impl Widget for OnboardingTip {
    fn name(&self) -> &'static str {
        "onboarding_tip"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["first-run", "feature", "shortcut", "tip", "welcome", "arrow"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (title, body, target): (&str, &str, &str) = match variant {
            0 => ("Welcome!", "Press b to open the bezier editor.", "key: b"),
            1 => ("New feature", "Use 1-9 to switch chart presets.", "key: 1-9"),
            2 => ("Shortcut", "q quits, p cycles the palette.", "key: q/p"),
            3 => ("Tip", "Click to open the editor too.", "mouse"),
            4 => ("Getting started", "Pick a component from the catalog.", "catalog"),
            _ => ("Arrow", "Look over there →", "→ target"),
        };
        let color = prims::palette(variant + 1);
        let w = 36u16;
        let h = 7u16;
        let bx = area.x + area.width.saturating_sub(w) / 2;
        let by = area.y + 2;
        let boxr = Rect::new(bx, by, w, h);
        prims::frame(buf, boxr, title, color);
        row_text(buf, boxr, 2, 2, body, WHITE);
        row_text(buf, boxr, 2, 4, &format!("▸ {target}"), prims::DIM);
        // Little arrow pointing at the box from the top.
        if area.height > h + 5 {
            let ax = boxr.x + boxr.width / 2;
            let ay = boxr.y.saturating_sub(1);
            if ax < area.width && ay < area.height {
                let cell = &mut buf[(ax, ay)];
                cell.set_symbol("▼").set_fg(color);
            }
        }
    }
}

// ---- Tutorial Step ----

pub struct TutorialStep;

impl Widget for TutorialStep {
    fn name(&self) -> &'static str {
        "tutorial_step"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["step1", "step2", "step3", "step4", "step5", "done"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (step, desc, total): (usize, &str, usize) = match variant {
            0 => (1, "Open the bezier editor", 4),
            1 => (2, "Cycle through presets", 4),
            2 => (3, "Drag a control point", 4),
            3 => (4, "Save to a slot (1-3)", 4),
            4 => (5, "Make your own curve", 5),
            _ => (5, "You did it! 🎉", 5),
        };
        let color = prims::palette(step % 6);
        let h = 8u16;
        let boxr = centered_box(buf, area, 44, h, "Tutorial", color);
        scrim(buf, area, boxr);
        let inner = Rect::new(boxr.x + 2, boxr.y + 2, boxr.width.saturating_sub(4), boxr.height.saturating_sub(4));
        // Step indicator dots.
        for i in 0..total {
            let dot = if i < step { "●" } else { "○" };
            let style = if i < step {
                Style::new().fg(prims::palette(i))
            } else {
                prims::DIM
            };
            row_text(buf, inner, (i * 2) as u16, 0, dot, style);
        }
        row_text(buf, inner, 0, 2, &format!("Step {step} of {total}"), prims::DIM);
        row_text(buf, inner, 0, 3, desc, WHITE);
        row_text(buf, inner, 0, 5, "Next: press →", prims::CYAN_BOLD);
    }
}

// ---- Theme Picker ----

pub struct ThemePicker;

impl Widget for ThemePicker {
    fn name(&self) -> &'static str {
        "theme_picker"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["nord", "gruvbox", "dracula", "solarized", "tokyo", "material"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let themes = ["nord", "gruvbox", "dracula", "solarized", "tokyo-night", "material"];
        let color = prims::palette(variant);
        let w = 60u16.min(area.width.saturating_sub(6));
        let h = 14u16.min(area.height.saturating_sub(4));
        let boxr = centered_box(buf, area, w, h, "Theme Picker", color);
        scrim(buf, area, boxr);
        let lw = 22u16;
        let list = Rect::new(boxr.x + 1, boxr.y + 1, lw, boxr.height.saturating_sub(2));
        prims::frame(buf, list, "Themes", Color::DarkGray);
        for (i, t) in themes.iter().enumerate() {
            let active = i == variant;
            let style = if active {
                Style::new().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
            } else {
                WHITE
            };
            row_text(buf, list, 1, 1 + i as u16, t, style);
        }
        // Live preview pane.
        let px = list.x + list.width + 1;
        let pw = boxr.width.saturating_sub(list.width + 3);
        let preview = Rect::new(px, boxr.y + 1, pw, boxr.height.saturating_sub(2));
        prims::frame(buf, preview, "Preview", Color::DarkGray);
        let bg = match variant {
            0 => Color::Rgb(46, 52, 64),
            1 => Color::Rgb(40, 40, 40),
            2 => Color::Rgb(40, 42, 54),
            3 => Color::Rgb(0, 43, 54),
            4 => Color::Rgb(26, 27, 38),
            _ => Color::Rgb(38, 50, 56),
        };
        for y in 1..preview.height.saturating_sub(1) {
            for x in 1..preview.width.saturating_sub(1) {
                let c = &mut buf[(preview.x + x, preview.y + y)];
                c.set_symbol(" ").set_bg(bg);
            }
        }
        row_text(buf, preview, 2, 2, "The quick brown fox", WHITE);
        row_text(buf, preview, 2, 3, "jumps over the dog.", Style::new().fg(color));
    }
}
