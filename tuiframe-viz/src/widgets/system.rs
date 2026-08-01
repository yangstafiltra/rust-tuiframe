//! System / exception widgets: accessibility_helper, error_boundary,
//! exception_handler, screenshot_mode, welcome_screen, migration_wizard.

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

// ---- Accessibility Helper ----

pub struct AccessibilityHelper;

impl Widget for AccessibilityHelper {
    fn name(&self) -> &'static str {
        "accessibility_helper"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["focus", "contrast", "large-text", "high-contrast", "reduce-motion", "screen-reader"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        let title = match variant {
            0 => "Focus Navigation",
            1 => "Color Contrast",
            2 => "Large Text",
            3 => "High Contrast",
            4 => "Reduce Motion",
            _ => "Screen Reader",
        };
        let w = 58u16.min(area.width.saturating_sub(4));
        let boxr = Rect::new(area.x + (area.width - w) / 2, area.y + 2, w, 12);
        prims::frame(buf, boxr, title, color);
        match variant {
            0 => {
                row_text(buf, boxr, 2, 1, "Tab to move focus between widgets.", WHITE);
                row_text_bg(buf, boxr, 2, 3, " [1] Sidebar ", Color::Black, color);
                row_text(buf, boxr, 16, 3, " [2] Main  [3] StatusBar ", prims::DIM);
                row_text(buf, boxr, 2, 5, "Focused: List (sidebar)", Style::new().fg(color).add_modifier(Modifier::BOLD));
                row_text(buf, boxr, 2, 7, "Announce: \"List, 5 items\"", prims::DIM);
            }
            1 => {
                row_text(buf, boxr, 2, 1, "All text meets WCAG AA contrast.", prims::GREEN);
                let swatches = [("White on Black", 21.0), ("Cyan on Black", 9.2), ("Gray on Black", 5.6), ("Blue on White", 7.2)];
                for (i, (label, ratio)) in swatches.iter().enumerate() {
                    row_text(buf, boxr, 2, 3 + i as u16, &format!("■ {label:<18} {ratio:>4} : 1"), if i == 1 { Style::new().fg(prims::palette(i)) } else { WHITE });
                }
            }
            2 => {
                row_text(buf, boxr, 2, 1, "Font size preview", WHITE);
                row_text(buf, boxr, 2, 2, "Small:  This is sample text.", prims::DIM);
                row_text(buf, boxr, 2, 3, "Large:  This is sample text.", Style::new().fg(color).add_modifier(Modifier::BOLD));
                row_text(buf, boxr, 2, 5, "Current: 16px  [−] 12px  [+] 20px", prims::DIM);
            }
            3 => {
                row_text(buf, boxr, 2, 1, "High contrast palette enabled", Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD));
                row_text_bg(buf, boxr, 2, 3, " ████ BLACK ", Color::Black, Color::White);
                row_text_bg(buf, boxr, 18, 3, " ████ WHITE ", Color::White, Color::Black);
                row_text_bg(buf, boxr, 2, 4, " ████ CYAN ", Color::Black, Color::LightCyan);
                row_text_bg(buf, boxr, 18, 4, " ████ YELLOW ", Color::Black, Color::LightYellow);
            }
            4 => {
                row_text(buf, boxr, 2, 1, "Motion effects reduced.", WHITE);
                row_text(buf, boxr, 2, 3, "Transitions: fade only", prims::DIM);
                row_text(buf, boxr, 2, 5, "▏▏▏   (no slide / scale)", prims::DIM);
            }
            _ => {
                row_text(buf, boxr, 2, 1, "Screen reader active", Style::new().fg(Color::LightGreen));
                row_text(buf, boxr, 2, 3, "\"Popup dialog appeared. 3 options.\"", prims::DIM);
                row_text(buf, boxr, 2, 5, "\"Button OK focused. Press Enter.\"", prims::DIM);
                row_text(buf, boxr, 2, 7, "[a] announce all  [v] verbosity", prims::DIM);
            }
        }
    }
}

// ---- Error Boundary ----

pub struct ErrorBoundary;

impl Widget for ErrorBoundary {
    fn name(&self) -> &'static str {
        "error_boundary"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["panic", "fallback", "retry", "partial", "boundary-tree", "recovered"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant + 3);
        let w = 60u16.min(area.width.saturating_sub(4));
        let boxr = Rect::new(area.x + (area.width - w) / 2, area.y + 3, w, 11);
        prims::frame(buf, boxr, "Error Boundary", color);
        match variant {
            0 => {
                row_text_bg(buf, boxr, 1, 1, " !  A widget crashed  ", Color::Black, Color::LightRed);
                row_text(buf, boxr, 2, 3, "render() panicked: index out of bounds", Style::new().fg(Color::LightRed));
                row_text_bg(buf, boxr, 2, 5, " [Try again] ", Color::Black, color);
                row_text_bg(buf, boxr, 15, 5, " [Report] ", Color::Black, Color::DarkGray);
            }
            1 => {
                row_text(buf, boxr, 2, 2, "This panel failed to load.", WHITE);
                row_text(buf, boxr, 2, 3, "  ┌─ sparkline.rs:14", prims::DIM);
                row_text(buf, boxr, 2, 4, "  └─ series_color: len 0", prims::DIM);
                row_text_bg(buf, boxr, 2, 6, " [Reload] ", Color::Black, color);
            }
            2 => {
                row_text(buf, boxr, 2, 2, "Retrying render…  (attempt 2)", WHITE);
                row_text_bg(buf, boxr, 2, 4, " Loading ", Color::Black, Color::LightYellow);
                row_text(buf, boxr, 2, 6, "Last error: empty dataset", prims::DIM);
            }
            3 => {
                row_text(buf, boxr, 2, 2, "Partial failure: 1 of 4 panels failed", prims::YELLOW);
                row_text(buf, boxr, 2, 4, "  ✓ Header   ✕ Chart   ✓ Footer   ✓ Nav", WHITE);
                row_text_bg(buf, boxr, 2, 6, " [Dismiss] ", Color::Black, color);
            }
            4 => {
                row_text(buf, boxr, 2, 1, "Boundary tree:", WHITE);
                row_text(buf, boxr, 2, 2, "  App", Style::new().fg(color));
                row_text(buf, boxr, 2, 3, "  └─ ErrorBoundary", Style::new().fg(color));
                row_text(buf, boxr, 2, 4, "      └─ ▸ ChartPanel (crashed)", prims::RED);
                row_text_bg(buf, boxr, 2, 6, " [Reset subtree] ", Color::Black, color);
            }
            _ => {
                row_text(buf, boxr, 2, 2, "✓ Recovered after fallback render.", Style::new().fg(Color::LightGreen));
                row_text(buf, boxr, 2, 4, "The chart loaded from its saved state.", prims::DIM);
                row_text_bg(buf, boxr, 2, 6, " [Continue] ", Color::Black, color);
            }
        }
    }
}

// ---- Exception Handler ----

pub struct ExceptionHandler;

impl Widget for ExceptionHandler {
    fn name(&self) -> &'static str {
        "exception_handler"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["panic", "trap", "crash-report", "watchdog", "recovered", "handler"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        // Full-width banner at the top.
        let (banner, msg): (&str, &str) = match variant {
            0 => ("PANIC", "thread 'main' panicked at src/main.rs:12"),
            1 => ("TRAP", "illegal instruction at 0x7ff..."),
            2 => ("CRASH REPORT", "collecting diagnostics…"),
            3 => ("WATCHDOG", "process not responding — restarting"),
            4 => ("RECOVERED", "caught exception, state restored"),
            _ => ("HANDLER", "global panic hook installed"),
        };
        row_text_bg(buf, area, 0, 0, &format!("  {banner}  "), Color::Black, Color::LightRed);
        row_text(buf, area, banner.len() as u16 + 5, 0, msg, WHITE);

        let color = prims::palette(variant + 2);
        let w = 62u16.min(area.width.saturating_sub(4));
        let boxr = Rect::new(area.x + (area.width - w) / 2, area.y + 3, w, 12);
        prims::frame(buf, boxr, "Exception Details", color);
        let rows: &[&str] = match variant {
            0 => &[
                "Message: index out of bounds: the len is 4 but the index is 7",
                "Location: tuiframe-viz/src/charts/parcoords.rs:207",
                "",
                "Stack:",
                "  0: interpolate_series",
                "  1: render_all_presets_no_panic",
            ],
            1 => &["Signal: SIGILL", "Address: 0x7ffd3a2b4000", "Register: rip=...", "", "core dumped: /tmp/core.1234"],
            2 => &["Collecting backtrace…", "  frame 0: render()", "  frame 1: engine.run()", "  frame 2: main()", "", "Writing crash.log"],
            3 => &["Watchdog timer elapsed (30s).", "Attempting graceful restart…", "", "Saved session state.", "PID 9876 stopped."],
            4 => &["Exception recovered in handler.", "Message: divide by zero", "", "Falling back to safe value 0.", "Continuing…"],
            _ => &["Handler installed at startup.", "Hook: custom_panic_hook", "", "Captures message + backtrace.", "Writes to ~/.tuiframe/crash.log"],
        };
        for (i, r) in rows.iter().enumerate() {
            let style = if r.starts_with("Message:") {
                Style::new().fg(Color::LightRed)
            } else if r.starts_with("Stack:") || r.starts_with("  frame") {
                prims::DIM
            } else {
                WHITE
            };
            row_text(buf, boxr, 2, 1 + i as u16, r, style);
        }
        let action = match variant {
            4 => " [Continue] ",
            _ => " [Quit] ",
        };
        row_text_bg(buf, boxr, 2, rows.len() as u16 + 1, action, Color::Black, color);
    }
}

// ---- Screenshot Mode ----

pub struct ScreenshotMode;

impl Widget for ScreenshotMode {
    fn name(&self) -> &'static str {
        "screenshot_mode"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["region", "window", "fullscreen", "selection", "annotate", "export"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        // A dashed selection rectangle.
        let (sel_w, sel_h) = match variant {
            0 => (area.width.saturating_sub(30), area.height.saturating_sub(12)),
            1 => (area.width.saturating_sub(12), area.height.saturating_sub(6)),
            2 => (area.width, area.height),
            3 => (area.width.saturating_sub(20), area.height.saturating_sub(9)),
            4 => (area.width.saturating_sub(26), area.height.saturating_sub(10)),
            _ => (area.width.saturating_sub(16), area.height.saturating_sub(8)),
        };
        let sx = (area.width - sel_w) / 2;
        let sy = (area.height - sel_h) / 2;
        for x in 0..sel_w {
            row_text(buf, area, sx + x, sy, if x % 2 == 0 { "┄" } else { " " }, prims::DIM);
            row_text(buf, area, sx + x, sy + sel_h - 1, if x % 2 == 0 { "┄" } else { " " }, prims::DIM);
        }
        for y in 1..sel_h.saturating_sub(1) {
            row_text(buf, area, sx, sy + y, if y % 2 == 0 { "┆" } else { " " }, prims::DIM);
            row_text(buf, area, sx + sel_w - 1, sy + y, if y % 2 == 0 { "┆" } else { " " }, prims::DIM);
        }
        for (cx, cy, ch) in [(0, 0, '┌'), (sel_w - 1, 0, '┐'), (0, sel_h - 1, '└'), (sel_w - 1, sel_h - 1, '┘')] {
            row_text(buf, area, sx + cx, sy + cy, &ch.to_string(), Style::new().fg(color).add_modifier(Modifier::BOLD));
        }
        // Coordinate readout at the top of the selection.
        let dim = format!("  {sel_w}×{sel_h}  @ ({sx},{sy})  ");
        row_text_bg(buf, area, sx, sy, &dim, Color::Black, color);
        // Controls.
        row_text_bg(buf, area, 2, area.height.saturating_sub(2), " [Enter] capture ", Color::Black, color);
        row_text_bg(buf, area, 18, area.height.saturating_sub(2), " [Esc] cancel ", Color::Black, Color::DarkGray);
        row_text(buf, area, 34, area.height.saturating_sub(2), "[←↑↓→] move  [+-] resize  [a] annotate", prims::DIM);
    }
}

// ---- Welcome Screen ----

pub struct WelcomeScreen;

impl Widget for WelcomeScreen {
    fn name(&self) -> &'static str {
        "welcome_screen"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["brand", "new-project", "recent", "quick-start", "about", "settings"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        let cy = area.height / 2;
        // Logo.
        let logo = "tuiframe";
        let lx = area.width.saturating_sub(logo.chars().count() as u16) / 2;
        for (i, ch) in logo.chars().enumerate() {
            let c = if variant == 3 { prims::palette(i % 4) } else { color };
            let cell = &mut buf[(lx + i as u16, cy.saturating_sub(4))];
            cell.set_symbol(&ch.to_string())
                .set_style(Style::new().fg(c).add_modifier(Modifier::BOLD));
        }
        row_text(buf, area, area.width.saturating_sub(50) / 2, cy.saturating_sub(2), "──────────────────────────────────────────────────", prims::DIM);
        let tagline = match variant {
            0 => "A TUI component framework for Rust",
            1 => "Create a new component project",
            2 => "Jump back into your recent work",
            3 => "Pick a component to preview",
            4 => "v0.1.0 — MIT license",
            _ => "Customize your workspace",
        };
        row_text(buf, area, area.width.saturating_sub(tagline.chars().count() as u16) / 2, cy.saturating_sub(1), tagline, prims::DIM);

        // Quick action grid.
        let actions: &[&str] = match variant {
            0 => &["New", "Open", "Docs", "Recent"],
            1 => &["Blank", "From template", "Import", "Scaffold"],
            2 => &["tuiframe-viz", "preview", "editor", "catalog"],
            3 => &["Popup", "StatusBar", "Loading", "Spotlight"],
            4 => &["License", "Credits", "Update", "Changelog"],
            _ => &["Theme", "Font", "Keymap", "Proxy"],
        };
        let grid_w = actions.len() as u16 * 16 + 4;
        let gx = area.width.saturating_sub(grid_w) / 2;
        for (i, a) in actions.iter().enumerate() {
            let x = gx + i as u16 * 16;
            row_text_bg(buf, area, x, cy + 1, &format!(" {a:<12} "), if i == 0 { Color::Black } else { Color::White }, if i == 0 { color } else { Color::Rgb(30, 30, 30) });
        }
        row_text(buf, area, area.width.saturating_sub(40) / 2, cy + 4, "[1-4] select  [Enter] open  [q] quit", prims::DIM);
    }
}

// ---- Migration Wizard ----

pub struct MigrationWizard;

impl Widget for MigrationWizard {
    fn name(&self) -> &'static str {
        "migration_wizard"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["intro", "source", "options", "progress", "summary", "done"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let color = prims::palette(variant);
        let steps = ["Intro", "Source", "Options", "Migrate", "Summary"];
        let current = variant.min(4);
        let w = 64u16.min(area.width.saturating_sub(4));
        let boxr = Rect::new(area.x + (area.width - w) / 2, area.y + 2, w, 15);
        prims::frame(buf, boxr, "Migration Wizard", color);
        // Step indicator.
        let mut sx = boxr.x + 2;
        for (i, s) in steps.iter().enumerate() {
            let done = i < current;
            let active = i == current;
            let style = if active {
                Style::new().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
            } else if done {
                Style::new().fg(Color::LightGreen)
            } else {
                prims::DIM
            };
            row_text(buf, boxr, sx - boxr.x, 1, &format!(" {s} "), style);
            sx += s.chars().count() as u16 + 3;
            if i < steps.len() - 1 {
                row_text(buf, boxr, sx - boxr.x - 1, 1, "→", prims::DIM);
            }
        }
        // Body.
        match variant {
            0 => {
                row_text(buf, boxr, 2, 3, "This wizard migrates your settings to the new format.", WHITE);
                row_text(buf, boxr, 2, 4, "It will copy 3 config files and validate them.", prims::DIM);
            }
            1 => {
                row_text(buf, boxr, 2, 3, "Source location:", WHITE);
                row_text_bg(buf, boxr, 2, 5, " ~/.tuiframe/old/config.toml ", Color::Black, color);
                row_text(buf, boxr, 2, 7, "  12 files found", prims::DIM);
            }
            2 => {
                row_text(buf, boxr, 2, 3, "Options:", WHITE);
                row_text(buf, boxr, 2, 5, "  ☑ Copy keybindings", prims::GREEN);
                row_text(buf, boxr, 2, 6, "  ☑ Preserve themes", prims::GREEN);
                row_text(buf, boxr, 2, 7, "  ☐ Migrate plugins", prims::DIM);
            }
            3 => {
                let wbar = boxr.width.saturating_sub(8);
                let progress = (tick % 60) as f64 / 60.0;
                let filled = (wbar as f64 * progress) as u16;
                row_text(buf, boxr, 2, 3, &format!("Migrating… {:>3.0}%", progress * 100.0), WHITE);
                row_text(buf, boxr, 2, 4, &"█".repeat(filled as usize), Style::new().fg(color));
                row_text(buf, boxr, 2 + filled, 4, &"░".repeat((wbar - filled) as usize), prims::DIM);
                row_text(buf, boxr, 2, 6, "  config.toml   ✓", prims::GREEN);
                row_text(buf, boxr, 2, 7, "  keymap.toml   ✓", prims::GREEN);
            }
            4 => {
                row_text(buf, boxr, 2, 3, "✓ Migration complete!", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD));
                row_text(buf, boxr, 2, 5, "  3 files migrated, 0 failed.", WHITE);
                row_text(buf, boxr, 2, 6, "  Backup saved to ~/.tuiframe/backup-20260801", prims::DIM);
                row_text_bg(buf, boxr, 2, 8, " [Finish] ", Color::Black, color);
            }
            _ => {}
        }
    }
}
