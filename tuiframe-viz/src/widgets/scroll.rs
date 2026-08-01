//! Scroll / zoom / layout widgets: scrollbar, minimap_scroll, resize_handle,
//! zoom_control, measurement_tool.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::prims;
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

// ---- Scrollbar ----

pub struct Scrollbar;

impl Widget for Scrollbar {
    fn name(&self) -> &'static str {
        "scrollbar"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["vertical", "horizontal", "block-thumb", "half-block", "short", "full"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let color = prims::palette(variant);
        let pos = ((tick / 8) % 6) as u16;
        if variant == 1 {
            // Horizontal scrollbar along the bottom.
            let y = area.height.saturating_sub(2);
            let track_w = area.width.saturating_sub(8);
            let thumb_w = 10u16;
            let tx = (pos * (track_w - thumb_w) / 6).clamp(0, track_w.saturating_sub(thumb_w));
            row_text(buf, area, 0, y, &"─".repeat(area.width as usize), prims::DIM);
            row_text_bg(buf, area, tx, y, &" ".repeat(thumb_w as usize), Color::Black, color);
            row_text_bg(buf, area, tx, y, &"╱".repeat(thumb_w as usize), Color::Black, color);
            return;
        }
        // Vertical scrollbar on the right edge.
        let sx = area.width.saturating_sub(3);
        let track_h = area.height.saturating_sub(5);
        let thumb_h = if variant == 4 { 6 } else { 12 };
        let step = if variant == 4 { 3 } else { 1 };
        let ty = (pos * step).min(track_h.saturating_sub(thumb_h));
        // Track.
        for y in 0..track_h {
            let sym = if variant == 2 { "▌" } else { "░" };
            row_text(buf, area, sx, y, sym, prims::DIM);
        }
        // Thumb.
        for i in 0..thumb_h {
            let y = ty + i;
            if y < track_h {
                row_text_bg(buf, area, sx, y, &"▐".repeat(1), Color::Black, color);
            }
        }
    }
}

// ---- Minimap Scroll ----

pub struct MinimapScroll;

impl Widget for MinimapScroll {
    fn name(&self) -> &'static str {
        "minimap_scroll"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["code", "log", "diff", "dense", "sparse", "colored"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64) {
        let color = prims::palette(variant);
        // Content column on the left.
        let _content_w = area.width.saturating_sub(8);
        for (i, line) in [
            "fn main() {",
            "    let x = 42;",
            "    println!(\"hi\");",
            "    if x > 10 {",
            "        run();",
            "    }",
            "}",
            "",
            "struct App {",
            "    name: String,",
            "}",
        ]
        .iter()
        .enumerate()
        {
            let style = if line.contains("fn") {
                Style::new().fg(prims::palette(1))
            } else if line.contains("struct") {
                Style::new().fg(prims::palette(2))
            } else {
                prims::DIM
            };
            row_text(buf, area, 1, 1 + i as u16, line, style);
        }
        // Minimap column: 1 char wide, one dot per source line.
        let mx = area.width.saturating_sub(3);
        let track_h = area.height.saturating_sub(4);
        let pos = ((tick / 10) % track_h.saturating_sub(6) as u64) as u16;
        for y in 0..track_h {
            // Build a density "wave" so the minimap looks like real code.
            let band = 2 + ((y * 7) % 5) as usize;
            let bright = band > 4;
            let c = if bright { color } else { Color::Rgb(90, 90, 90) };
            row_text_bg(buf, area, mx, y, "▮", if bright { Color::Black } else { Color::DarkGray }, c);
        }
        // Viewport window (the visible region of the code).
        let win_h = 5u16;
        let wy = pos + 2;
        for i in 0..win_h {
            let y = wy + i;
            if y < track_h {
                let cell = &mut buf[(area.x + mx, area.y + y)];
                cell.set_symbol("▮").set_fg(Color::Black).set_bg(Color::White);
            }
        }
    }
}

// ---- Resize Handle ----

pub struct ResizeHandle;

impl Widget for ResizeHandle {
    fn name(&self) -> &'static str {
        "resize_handle"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["corner", "edge", "split-h", "split-v", "cross", "dots"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        let (x, y) = match variant {
            0 => (area.width.saturating_sub(3), area.height.saturating_sub(3)),
            1 => (area.width / 2, area.height.saturating_sub(2)),
            2 => (area.width.saturating_sub(2), area.height / 2),
            3 => (area.width / 2, area.height.saturating_sub(2)),
            4 => (area.width / 2, area.height / 2),
            _ => (area.width.saturating_sub(4), area.height / 2),
        };
        let style = Style::new().fg(color).add_modifier(Modifier::BOLD);
        match variant {
            0 => {
                row_text(buf, area, x, y, "╋", style);
            }
            1 => {
                row_text(buf, area, x.saturating_sub(5), y, "⠤⠤┊⠤⠤", style);
            }
            2 => {
                for (i, s) in ["⠇", "⠿", "⠇"].iter().enumerate() {
                    row_text(buf, area, x, y + i as u16, s, style);
                }
            }
            3 => {
                row_text(buf, area, x, y.saturating_sub(2), "⋮", style);
                row_text(buf, area, x, y, "⋮", style);
                row_text(buf, area, x, y + 2, "⋮", style);
            }
            4 => {
                for i in 0..3 {
                    row_text(buf, area, x.saturating_sub(2) + i as u16, y.saturating_sub(1), "▞", style);
                    row_text(buf, area, x.saturating_sub(2) + i as u16, y + 1, "▚", style);
                }
            }
            _ => {
                for i in 0..5 {
                    row_text(buf, area, x + i as u16, y, "⠂", style);
                }
            }
        }
    }
}

// ---- Zoom Control ----

pub struct ZoomControl;

impl Widget for ZoomControl {
    fn name(&self) -> &'static str {
        "zoom_control"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["50", "75", "100", "125", "150", "200"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let pcts = [50, 75, 100, 125, 150, 200];
        let pct = pcts[variant];
        let color = prims::palette(variant);
        let y = area.height / 2;
        // Control cluster.
        let cx = area.width.saturating_sub(26) / 2;
        row_text_bg(buf, area, cx, y, " ⟨−⟩ ", Color::Black, color);
        row_text_bg(buf, area, cx + 5, y, &format!(" {:>3}% ", pct), Color::White, Color::DarkGray);
        row_text_bg(buf, area, cx + 13, y, " ⟨+⟩ ", Color::Black, color);
        // Preset buttons.
        let mut bx = cx;
        for (i, p) in pcts.iter().enumerate() {
            if i == variant {
                row_text_bg(buf, area, bx, y + 2, &format!(" {p} "), Color::Black, color);
            } else {
                row_text(buf, area, bx, y + 2, &format!(" {p} "), prims::DIM);
            }
            bx += 5;
        }
        row_text(buf, area, cx, y + 4, &format!("{pct}% of original size"), prims::DIM);
    }
}

// ---- Measurement Tool ----

pub struct MeasurementTool;

impl Widget for MeasurementTool {
    fn name(&self) -> &'static str {
        "measurement_tool"
    }

    fn variants(&self) -> Vec<&'static str> {
        vec!["panel", "region", "element", "selection", "grid", "ruler"]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, _tick: u64) {
        let color = prims::palette(variant);
        let (w, h) = match variant {
            0 => (30, 10),
            1 => (24, 8),
            2 => (16, 5),
            3 => (34, 12),
            4 => (20, 6),
            _ => (40, 4),
        };
        let bx = area.width.saturating_sub(w) / 2;
        let by = area.height.saturating_sub(h) / 2;
        let _boxr = Rect::new(bx, by, w, h);
        // Outline the measured region.
        for x in 0..w {
            row_text(buf, area, bx + x, by, "╌", prims::DIM);
            row_text(buf, area, bx + x, by + h - 1, "╌", prims::DIM);
        }
        for y in 1..h.saturating_sub(1) {
            row_text(buf, area, bx, by + y, "╎", prims::DIM);
            row_text(buf, area, bx + w - 1, by + y, "╎", prims::DIM);
        }
        // Corner markers.
        for (cx0, cy0, ch) in [(0, 0, '┌'), (w - 1, 0, '┐'), (0, h - 1, '└'), (w - 1, h - 1, '┘')] {
            row_text(buf, area, bx + cx0, by + cy0, &ch.to_string(), Style::new().fg(color));
        }
        // Size readout.
        let label = format!(" {w}×{h}  @({bx},{by}) ");
        row_text_bg(buf, area, bx, by + h + 1, &label, Color::Black, color);
        // Dimension rulers along top and left.
        let top = format!(" {:>3}px ", w);
        row_text(buf, area, bx, by.saturating_sub(1), &top, prims::DIM);
        let left = format!("{:>2}px ", h);
        row_text(buf, area, bx.saturating_sub(6), by + h / 2, &left, prims::DIM);
    }
}
