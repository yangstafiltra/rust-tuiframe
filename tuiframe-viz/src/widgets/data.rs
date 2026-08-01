//! Data / text widgets: history, clipboard_view, command_output, inspector,
//! animated_text, empty_state.

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

// ---- History ----

pub struct History;

impl Widget for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["undo", "git", "prompt", "package", "build", "long"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (title, entries): (&str, Vec<&str>) = match variant {
            0 => ("Undo Stack", vec!["▸ paste text", "  insert 'foo'", "  delete word", "  type char", "  new file"]),
            1 => ("Git History", vec!["▸ fix: parcoords bounds", "  feat: bezier editor", "  refactor: grid lines", "  perf: rasterizer", "  init: tuiframe-viz"]),
            2 => ("Shell History", vec!["▸ cargo test", "  tuiframe preview heatmap", "  git push origin main", "  cargo build --release"]),
            3 => ("Package Versions", vec!["▸ v1.3.0  latest", "  v1.2.1  security", "  v1.2.0  stable", "  v1.1.5  legacy"]),
            4 => ("Build Log", vec!["▸ Compiling tuiframe-viz", "  Compiling tuiframe-cli", "  Finished release", "  Running tests"]),
            _ => ("Long", vec!["▸ item A", "  item B", "  item C", "  item D", "  item E", "  item F"]),
        };
        let entries: Vec<&str> = entries;
        let color = prims::palette(variant);
        let lx = area.width.saturating_sub(36) / 2;
        let ly = 3u16;
        let boxr = Rect::new(lx, ly, 36, entries.len() as u16 + 3);
        prims::frame(buf, boxr, title, color);
        for (i, e) in entries.iter().enumerate() {
            let active = e.starts_with("▸");
            let style = if active {
                Style::new().fg(color).add_modifier(Modifier::BOLD)
            } else {
                prims::DIM
            };
            row_text(buf, boxr, 2, 1 + i as u16, e, style);
        }
        row_text(buf, boxr, 2, entries.len() as u16 + 1, "[↑/↓] select  [Enter] restore", prims::DIM);
    }
}

// ---- Clipboard View ----

pub struct ClipboardView;

impl Widget for ClipboardView {
    fn name(&self) -> &'static str {
        "clipboard_view"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["text", "code", "paths", "urls", "json", "mixed"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (title, entries): (&str, Vec<&str>) = match variant {
            0 => ("Clipboard", vec!["https://opencode.ai/docs", "fn main() {", "let x = 42;", "git commit -m wip", "const N = 10"]),
            1 => ("Code Snippets", vec!["fn run() { ... }", "impl Chart for Area", "let w = area.width", "match key.code"]),
            2 => ("Paths", vec!["/home/user", "src/widgets/overlay.rs", "Cargo.toml", "target/release/"]),
            3 => ("URLs", vec!["https://docs.rs/ratatui", "https://crates.io", "https://github.com/opencode"]),
            4 => ("JSON", vec!["{\"name\":\"popup\"}", "{\"presets\":17}", "[1,2,3,4]", "{\"ok\":true}"]),
            _ => ("Mixed", vec!["plain text", "https://example.com", "src/main.rs", "{\"a\":1}", "let y = 2;"]),
        };
        let color = prims::palette(variant);
        let lx = area.width.saturating_sub(44) / 2;
        let boxr = Rect::new(lx, 2, 44, entries.len() as u16 + 3);
        prims::frame(buf, boxr, title, color);
        for (i, e) in entries.iter().enumerate() {
            let active = i == 0;
            let (fg, bg) = if active {
                (Color::Black, color)
            } else {
                (Color::Gray, Color::Black)
            };
            row_text_bg(buf, boxr, 1, 1 + i as u16, &format!(" {} ", e), fg, bg);
        }
        // Preview pane of the selected entry.
        let py = boxr.y + boxr.height + 1;
        let preview = Rect::new(lx, py, 44, 6);
        prims::frame(buf, preview, "Preview", Color::DarkGray);
        row_text(buf, preview, 1, 1, entries[0], WHITE);
        row_text(buf, preview, 1, 3, "Copied: [Ctrl+C]  Pin: [p]", prims::DIM);
    }
}

// ---- Command Output ----

pub struct CommandOutput;

impl Widget for CommandOutput {
    fn name(&self) -> &'static str {
        "command_output"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["test", "build", "git", "fmt", "lint", "deploy"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (title, lines): (&str, Vec<(&str, bool)>) = match variant {
            0 => ("cargo test", vec![
                ("   Compiling tuiframe-viz", false),
                ("   Running 71 tests", false),
                ("   test result: ok", true),
                ("   Finished", false),
            ]),
            1 => ("cargo build", vec![
                ("   Compiling tuiframe-cli", false),
                ("   Compiling tuiframe-viz", false),
                ("   error: unused variable", true),
                ("   Finished (with 1 error)", false),
            ]),
            2 => ("git status", vec![
                ("   On branch main", false),
                ("   modified: engine.rs", true),
                ("   untracked: widgets/", true),
                ("   nothing to commit", false),
            ]),
            3 => ("cargo fmt", vec![
                ("   Checking formatting", false),
                ("   found 3 unformatted files", true),
                ("   formatting finished", false),
            ]),
            4 => ("clippy", vec![
                ("   Checking tuiframe-viz", false),
                ("   warning: needless borrow", true),
                ("   Finished", false),
            ]),
            _ => ("deploy", vec![
                ("   Building image", false),
                ("   Pushing to registry", false),
                ("   Deployed to prod", true),
            ]),
        };
        let color = prims::palette(variant);
        let cx = area.width.saturating_sub(60) / 2;
        let boxr = Rect::new(cx, 2, 60, 9);
        prims::frame(buf, boxr, &format!("$ {title}"), color);
        for (i, (line, is_err)) in lines.iter().enumerate() {
            let style = if *is_err {
                Style::new().fg(Color::LightRed)
            } else {
                prims::DIM
            };
            row_text(buf, boxr, 2, 1 + i as u16, line, style);
        }
        row_text(buf, boxr, 2, lines.len() as u16 + 1, "exit code: 0", prims::GREEN);
    }
}

// ---- Inspector ----

pub struct Inspector;

impl Widget for Inspector {
    fn name(&self) -> &'static str {
        "inspector"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["widget-tree", "layout", "state", "events", "focus", "metrics"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        let w = area.width.saturating_sub(4);
        let boxr = Rect::new(area.x + 2, area.y + 1, w, area.height.saturating_sub(4));
        prims::frame(buf, boxr, "Widget Inspector", color);
        let rows: &[(&str, &str)] = match variant {
            0 => &[
                ("▸ App", "16×30"),
                ("  ├─ Sidebar", "4×30"),
                ("  │   └─ List", "4×28"),
                ("  └─ Main", "12×30"),
                ("      └─ Paragraph", "12×30"),
            ],
            1 => &[
                ("▸ Layout", "Horizontal"),
                ("  ├─ Constraint::Length(4)", ""),
                ("  ├─ Constraint::Min(0)", ""),
                ("  └─ Margin 0", ""),
            ],
            2 => &[
                ("▸ State", ""),
                ("  focused: List", ""),
                ("  selected: 2", ""),
                ("  scroll: (0, 0)", ""),
                ("  hover: None", ""),
            ],
            3 => &[
                ("▸ Events", ""),
                ("  key: q", ""),
                ("  resize: 96×30", ""),
                ("  mouse: click@12,3", ""),
            ],
            4 => &[
                ("▸ Focus chain", ""),
                ("  1 List (focused)", ""),
                ("  2 Paragraph", ""),
                ("  3 StatusBar", ""),
            ],
            _ => &[
                ("▸ Metrics", ""),
                ("  render: 1.2ms", ""),
                ("  cells: 2880", ""),
                ("  changes: 42", ""),
                ("  fps: 60", ""),
            ],
        };
        for (i, (label, val)) in rows.iter().enumerate() {
            let active = label.starts_with("▸");
            let style = if active {
                Style::new().fg(color).add_modifier(Modifier::BOLD)
            } else {
                prims::DIM
            };
            row_text(buf, boxr, 2, 1 + i as u16, label, style);
            if !val.is_empty() {
                row_text(buf, boxr, 30, 1 + i as u16, val, prims::DIM);
            }
        }
        row_text(buf, boxr, 2, rows.len() as u16 + 1, "[↓] expand  [b] breakpoint  [h] highlight", prims::DIM);
    }
}

// ---- Animated Text ----

pub struct AnimatedText;

impl Widget for AnimatedText {
    fn name(&self) -> &'static str {
        "animated_text"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["pulse", "wave", "rainbow", "gradient", "sparkle", "slide"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let text = match variant {
            0 => "PULSING",
            1 => "WAVE",
            2 => "RAINBOW",
            3 => "GRADIENT",
            4 => "SPARKLE",
            _ => "SLIDING",
        };
        let y = area.height / 2;
        let base = area.width.saturating_sub(text.chars().count() as u16) / 2;
        let phase = (tick as usize) % 12;
        for (i, ch) in text.chars().enumerate() {
            let color = match variant {
                0 => {
                    // Pulse: brightness oscillates.
                    let t = ((phase + i) % 12) as f64 / 12.0;
                    let lum = ((1.0 - t) * 0.6 + 0.2) as u8 * 4;
                    Color::Rgb(lum, lum.min(60) + 40, lum + 60)
                }
                1 => {
                    // Wave: height bounces per character.
                    let off = ((phase as i32 - i as i32).rem_euclid(12)) as f64;
                    let hgt = (off * 0.5).sin() * 3.0;
                    let yy = y.saturating_sub(hgt as u16);
                    let cell = &mut buf[(area.x + base + i as u16, yy)];
                    cell.set_symbol(&ch.to_string()).set_fg(prims::palette(i));
                    continue;
                }
                2 => prims::palette(i),
                3 => {
                    let t = (phase as f64 + i as f64) / 12.0;
                    prims::lerp_color(prims::palette(0), prims::palette(4), t)
                }
                4 => {
                    let sparkle = (phase as usize + i) % 3 == 0;
                    if sparkle {
                        Color::White
                    } else {
                        Color::DarkGray
                    }
                }
                _ => {
                    // Slide: letters appear progressively.
                    if phase >= 6 && i <= (phase - 6) {
                        prims::palette(i)
                    } else {
                        Color::DarkGray
                    }
                }
            };
            let cell = &mut buf[(area.x + base + i as u16, y)];
            cell.set_symbol(&ch.to_string()).set_fg(color);
            if variant == 0 {
                cell.set_style(Style::new().fg(color).add_modifier(Modifier::BOLD));
            }
        }
        row_text(buf, area, area.width.saturating_sub(30) / 2, y + 2, &format!("style: {variant}"), prims::DIM);
    }
}

// ---- Empty State ----

pub struct EmptyState;

impl Widget for EmptyState {
    fn name(&self) -> &'static str {
        "empty_state"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["folder", "search", "inbox", "trash", "error", "blank"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let (icon, title, hint, action): (&str, &str, &str, &str) = match variant {
            0 => ("📁", "This folder is empty", "Drag files here or create something new.", "New file"),
            1 => ("🔍", "No results found", "Try a different search term or filter.", "Clear filters"),
            2 => ("📭", "Inbox zero", "You're all caught up. Nice work!", "Compose"),
            3 => ("🗑", "Trash is empty", "Deleted items will appear here.", "Restore all"),
            4 => ("⚠", "Nothing to display", "Something went wrong loading this view.", "Retry"),
            _ => ("✦", "Blank slate", "Start building from scratch.", "Get started"),
        };
        let color = prims::palette(variant);
        let cy = area.height / 2;
        // Icon.
        row_text(buf, area, area.width.saturating_sub(3) / 2, cy.saturating_sub(3), icon, WHITE);
        // Title.
        row_text(buf, area, area.width.saturating_sub(title.chars().count() as u16) / 2, cy, title, Style::new().fg(color).add_modifier(Modifier::BOLD));
        // Hint.
        row_text(buf, area, area.width.saturating_sub(hint.chars().count() as u16) / 2, cy + 1, hint, prims::DIM);
        // Action button.
        let x0 = area.width.saturating_sub(action.chars().count() as u16 + 4) / 2;
        row_text_bg(buf, area, x0, cy + 3, &format!(" [{action}] "), Color::Black, color);
    }
}
