use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

// ---- Styles ----
pub const DIM: Style = Style::new().fg(Color::DarkGray);
pub const WHITE: Style = Style::new().fg(Color::Gray);
pub const BRIGHT: Style = Style::new().fg(Color::White).add_modifier(Modifier::BOLD);
pub const RED: Style = Style::new().fg(Color::LightRed);
pub const GREEN: Style = Style::new().fg(Color::LightGreen);
pub const CYAN_BOLD: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
pub const YELLOW: Style = Style::new().fg(Color::LightYellow);

/// A curated palette of distinct, legible colors for multi-series charts.
pub const PALETTE: [Color; 10] = [
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightBlue,
    Color::Rgb(255, 165, 0), // orange
    Color::Rgb(120, 220, 120),
    Color::Rgb(255, 120, 120),
    Color::Rgb(170, 130, 255),
    Color::Rgb(240, 240, 150),
];

/// Hairline grid color: dim, reserved exclusively for gridlines so the
/// rasterizer can draw them as thin solid lines instead of solid blocks.
pub const GRID_LINE: Color = Color::Rgb(0x30, 0x30, 0x30);

pub fn palette(i: usize) -> Color {
    PALETTE[(i + palette_offset()) % PALETTE.len()]
}

/// Currently selected palette rotation (0 = base palette). Shifting this
/// cycles every chart's colors together — the `p` key in the engine.
pub fn set_palette_offset(offset: usize) {
    unsafe {
        PALETTE_OFFSET = offset % PALETTE.len();
    }
}

pub fn palette_offset() -> usize {
    unsafe { PALETTE_OFFSET }
}

static mut PALETTE_OFFSET: usize = 0;

/// Interpolate between two colors by `t in [0,1]` (RGB lerp).
pub fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let rgb = |c: Color| match c {
        Color::Rgb(r, g, bl) => (r as f64, g as f64, bl as f64),
        Color::LightCyan => (120.0, 220.0, 220.0),
        Color::LightMagenta => (220.0, 120.0, 220.0),
        Color::LightYellow => (240.0, 240.0, 120.0),
        Color::LightGreen => (120.0, 220.0, 120.0),
        Color::LightBlue => (120.0, 160.0, 220.0),
        Color::Cyan => (0.0, 200.0, 200.0),
        Color::Magenta => (200.0, 0.0, 200.0),
        Color::Yellow => (200.0, 200.0, 0.0),
        Color::Green => (0.0, 200.0, 0.0),
        Color::Blue => (0.0, 0.0, 200.0),
        Color::Red => (200.0, 0.0, 0.0),
        Color::Gray => (128.0, 128.0, 128.0),
        Color::DarkGray => (80.0, 80.0, 80.0),
        Color::Black => (0.0, 0.0, 0.0),
        Color::White => (255.0, 255.0, 255.0),
        _ => (200.0, 200.0, 200.0),
    };
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    Color::Rgb(
        (ar + (br - ar) * t).round() as u8,
        (ag + (bg - ag) * t).round() as u8,
        (ab + (bb - ab) * t).round() as u8,
    )
}

/// Gradient color for series `i` of `total`: interpolates between the first
/// and an accent palette color so multi-series charts flow through a gradient
/// rather than cycling discrete hues. Respects the palette offset.
pub fn series_color(i: usize, total: usize) -> Color {
    if total <= 1 {
        return palette(i);
    }
    let t = i as f64 / (total - 1) as f64;
    lerp_color(palette(0), palette(4), t)
}

pub fn fg(color: Color) -> Style {
    Style::new().fg(color)
}

// ---- Text helpers ----

/// Write a string on one row, starting at column 0, clipped to the area.
pub fn text(buf: &mut Buffer, area: Rect, y: u16, s: &str, style: Style) {
    if y >= area.y + area.height {
        return;
    }
    let avail = area.width as usize;
    for (i, ch) in s.chars().take(avail).enumerate() {
        buf[(area.x + i as u16, area.y + y)]
            .set_symbol(&ch.to_string())
            .set_style(style);
    }
}

/// Write a string at an absolute (x, y) position, clipped to the buffer bounds.
pub fn abs_text(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) {
    let buf_area = buf.area;
    if y >= buf_area.height {
        return;
    }
    let avail = buf_area.width.saturating_sub(x);
    for (i, ch) in s.chars().take(avail as usize).enumerate() {
        let cx = x + i as u16;
        if cx < buf_area.width {
            buf[(cx, y)].set_symbol(&ch.to_string()).set_style(style);
        }
    }
}

pub fn clear_line(buf: &mut Buffer, area: Rect, y: u16) {
    if y >= area.height {
        return;
    }
    for x in 0..area.width {
        buf[(area.x + x, area.y + y)].set_symbol(" ").reset();
    }
}

pub fn put_cell(buf: &mut Buffer, area: Rect, x: u16, y: u16, ch: char, style: Style) {
    if x >= area.width || y >= area.height {
        return;
    }
    buf[(area.x + x, area.y + y)].set_symbol(&ch.to_string()).set_style(style);
}

/// Draw a horizontal border line (─) across the full width.
pub fn hline(buf: &mut Buffer, area: Rect, y: u16, style: Style) {
    for x in 0..area.width {
        put_cell(buf, area, x, y, '─', style);
    }
}

/// Draw an axis line with tick labels and optional vertical grid.
/// `min`/`max` are data values; plot height is `plot_h`.
pub fn axis(
    buf: &mut Buffer,
    area: Rect,
    plot_top: u16,
    plot_h: u16,
    y_min: f64,
    y_max: f64,
    color: Color,
) {
    let baseline = plot_top + plot_h - 1;
    for x in 0..area.width {
        put_cell(buf, area, x, baseline, '─', fg(color));
    }
    // Tick marks + labels at a few nice levels.
    let n = 4.min(plot_h.saturating_sub(2)) as usize;
    for i in 0..=n {
        let frac = i as f64 / n as f64;
        let ty = plot_top + ((plot_h as f64 - 1.0) * frac) as u16;
        put_cell(buf, area, 0, ty, '┼', fg(color));
        put_cell(buf, area, area.width - 1, ty, '┤', fg(color));
        let val = y_max - (y_max - y_min) * frac;
        let label = fmt(val);
        text(buf, area, ty, &format!("{label:<7}"), fg(color).add_modifier(Modifier::DIM));
    }
    // Baseline ticks under the axis.
    for (i, l) in labels_at(area.width as usize, 8, &label_items(&[])).iter().enumerate() {
        let _ = (i, l);
    }
    let _ = color;
}

pub fn fmt(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 {
        return "0".to_string();
    }
    if a >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if a >= 1_000.0 {
        format!("{:.1}K", v / 1_000.0)
    } else if a >= 100.0 {
        format!("{:.0}", v)
    } else if a >= 10.0 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

#[derive(Clone, Debug)]
pub struct LabelItem {
    pub text: String,
    pub index: usize,
}

pub fn label_items(names: &[String]) -> Vec<LabelItem> {
    names
        .iter()
        .enumerate()
        .map(|(i, n)| LabelItem { text: n.clone(), index: i })
        .collect()
}

/// Pick a subset of labels that fit without overlap across `width` columns.
pub fn labels_at(width: usize, min_gap: usize, items: &[LabelItem]) -> Vec<LabelItem> {
    if items.is_empty() {
        return Vec::new();
    }
    let stride = items.len().max(1);
    let max_cols = (width + min_gap - 1) / min_gap;
    let mut chosen = Vec::new();
    let step = (stride as f64 / max_cols as f64).ceil() as usize;
    let step = step.max(1);
    let mut i = 0usize;
    let mut last_end = 0usize;
    while i < items.len() {
        let item = &items[i];
        let x = if items.len() <= 1 {
            0
        } else {
            (item.index as f64 / (items.len() - 1) as f64 * (width - 1) as f64).round() as usize
        };
        if x >= last_end {
            chosen.push(item.clone());
            last_end = x + item.text.chars().count();
        }
        i += step;
    }
    chosen
}

// ---- Half-block canvas (2x vertical resolution) ----

pub const HALF_BLOCKS: [&str; 4] = [" ", "▀", "▄", "█"];

/// A pixel grid with 2x vertical resolution. Each terminal cell holds two
/// stacked pixels (top/bottom). This is what makes curves and fills smooth.
pub struct HalfCanvas {
    pub w: usize,
    pub h: usize, // rows
    pub px: usize,
    pub py: usize, // pixels (py == 2*h)
    pub fg: Vec<Option<Color>>,
}

impl HalfCanvas {
    pub fn new(w: usize, h: usize) -> Self {
        HalfCanvas {
            w,
            h,
            px: w,
            py: h * 2,
            fg: vec![None; w * h * 2],
        }
    }

    pub fn set_px(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.px as i32 || y >= self.py as i32 {
            return;
        }
        let idx = (y as usize) * self.px + x as usize;
        self.fg[idx] = Some(color);
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.set_px(x, y, color);
        }
    }

    pub fn hline_px(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set_px(x, y, color);
        }
    }

    /// Fill a polygon (list of (x,y) pixel coords) with a color.
    pub fn fill_poly(&mut self, pts: &[(f64, f64)], color: Color) {
        if pts.len() < 2 {
            return;
        }
        let ys: Vec<f64> = pts.iter().map(|p| p.1).collect();
        let ymin = ys.iter().cloned().fold(f64::INFINITY, f64::min).floor() as i32;
        let ymax = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max).ceil() as i32;
        for y in ymin..=ymax {
            let mut xs: Vec<f64> = Vec::new();
            for i in 0..pts.len() {
                let (x1, y1) = pts[i];
                let (x2, y2) = pts[(i + 1) % pts.len()];
                if (y1 <= y as f64 && y2 > y as f64) || (y2 <= y as f64 && y1 > y as f64) {
                    let t = (y as f64 - y1) / (y2 - y1);
                    xs.push(x1 + t * (x2 - x1));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut i = 0;
            while i + 1 < xs.len() {
                let x0 = xs[i].ceil() as i32;
                let x1 = xs[i + 1].floor() as i32;
                for x in x0..=x1 {
                    self.set_px(x, y, color);
                }
                i += 2;
            }
        }
    }

    /// Flush the pixel grid into the buffer.
    pub fn blit(&self, buf: &mut Buffer, area: Rect) {
        for row in 0..self.h {
            for col in 0..self.w {
                let top = self.fg[row * 2 * self.px + col];
                let bot = self.fg[(row * 2 + 1) * self.px + col];
                let cell = &mut buf[(area.x + col as u16, area.y + row as u16)];
                match (top, bot) {
                    (None, None) => {
                        cell.set_symbol(" ");
                    }
                    (Some(t), None) => {
                        if t == GRID_LINE {
                            // horizontal gridline: a single pixel row reads as
                            // a thin solid line, not a half block.
                            cell.set_symbol("─").set_fg(t);
                        } else {
                            cell.set_symbol("▀").set_fg(t);
                        }
                    }
                    (None, Some(b)) => {
                        if b == GRID_LINE {
                            cell.set_symbol("─").set_fg(b);
                        } else {
                            cell.set_symbol("▄").set_fg(b);
                        }
                    }
                    (Some(t), Some(b)) if t == b => {
                        if t == GRID_LINE {
                            // vertical gridline: a single pixel column reads as
                            // a thin solid line, not a solid block.
                            cell.set_symbol("│").set_fg(t);
                        } else {
                            cell.set_symbol("█").set_fg(t);
                        }
                    }
                    (Some(t), Some(b)) => {
                        cell.set_symbol("▀").set_fg(t).set_bg(b);
                    }
                }
            }
        }
    }

    /// Clear to blank.
    pub fn clear(&mut self) {
        self.fg.iter_mut().for_each(|v| *v = None);
    }
}

// ---- Legend ----

pub fn legend(buf: &mut Buffer, area: Rect, names: &[String], colors: &[Color]) {
    let mut x = 0usize;
    for (i, name) in names.iter().enumerate() {
        let color = colors[i % colors.len()];
        let item = format!("■ {name}  ");
        for ch in item.chars() {
            if x >= area.width as usize {
                return;
            }
            let style = if ch == '■' { fg(color) } else { WHITE };
            buf[(area.x + x as u16, area.y + area.height - 1)]
                .set_symbol(&ch.to_string())
                .set_style(style);
            x += 1;
        }
    }
}

// ---- Titles ----

pub fn centered_title(buf: &mut Buffer, area: Rect, y: u16, title: &str, color: Color) {
    let s = format!(" {title} ");
    let x = area.width.saturating_sub(s.chars().count() as u16) / 2;
    for (i, ch) in s.chars().enumerate() {
        let style = fg(color).add_modifier(Modifier::BOLD);
        buf[(area.x + x + i as u16, area.y + y)]
            .set_symbol(&ch.to_string())
            .set_style(style);
    }
}

// ---- Border ----

pub fn frame(buf: &mut Buffer, area: Rect, title: &str, color: Color) {
    let st = fg(color);
    for x in 0..area.width {
        put_cell(buf, area, x, 0, '─', st);
        put_cell(buf, area, x, area.height - 1, '─', st);
    }
    for y in 1..area.height.saturating_sub(1) {
        put_cell(buf, area, 0, y, '│', st);
        put_cell(buf, area, area.width - 1, y, '│', st);
    }
    put_cell(buf, area, 0, 0, '┌', st);
    put_cell(buf, area, area.width - 1, 0, '┐', st);
    put_cell(buf, area, 0, area.height - 1, '└', st);
    put_cell(buf, area, area.width - 1, area.height - 1, '┘', st);
    centered_title(buf, area, 0, title, color);
}

// ---- Bars ----

/// Draw vertical bars using half-blocks for a smooth top edge.
/// `values` normalized to 0..1, drawn with `plot_h` rows available above baseline.
pub fn bars(
    buf: &mut Buffer,
    area: Rect,
    baseline: u16,
    values: &[f64], // 0..1
    colors: &[Color],
) {
    let n = values.len();
    if n == 0 {
        return;
    }
    let plot_w = area.width as usize;
    let mut canvas = HalfCanvas::new(plot_w, (area.height - (baseline - area.y)) as usize);
    let base_px = canvas.py as i32 - 1;
    for i in 0..n {
        let frac = i as f64 / n as f64;
        let x0 = (frac * plot_w as f64) as i32;
        let x1 = (((i + 1) as f64 / n as f64) * plot_w as f64) as i32;
        let x1 = x1.max(x0 + 1);
        let hpx = (values[i].clamp(0.0, 1.0) * (canvas.py - 1) as f64) as i32;
        let color = colors[i % colors.len()];
        for x in x0..x1 {
            for y in (base_px - hpx).max(0)..=base_px {
                canvas.set_px(x, y, color);
            }
        }
    }
    canvas.blit(buf, Rect::new(area.x, area.y, area.width, baseline - area.y));
}

/// Draw horizontal bars (for gantt / funnel / bullet), one per row.
/// `x0`, `x1` are normalized 0..1 across the plot width.
pub fn hbar(
    buf: &mut Buffer,
    area: Rect,
    row: u16,
    x0: f64,
    x1: f64,
    color: Color,
    fill: &str,
) {
    if row >= area.height {
        return;
    }
    let w = area.width as usize;
    let a = (x0.clamp(0.0, 1.0) * w as f64) as usize;
    let b = (x1.clamp(0.0, 1.0) * w as f64) as usize;
    let chars: Vec<char> = fill.chars().collect();
    for x in a..b.min(w) {
        let ch = chars[(x - a) % chars.len()];
        put_cell(buf, area, x as u16, row, ch, fg(color));
    }
}

// ---- Scatter ----

pub fn plot_point(buf: &mut Buffer, area: Rect, x: u16, y: u16, color: Color, ch: char) {
    if x < area.width && y < area.height {
        put_cell(buf, area, x, y, ch, fg(color));
    }
}

// ---- Misc span ----

pub fn span<'a>(s: &'a str, color: Color) -> Span<'a> {
    Span::styled(s, Style::new().fg(color))
}

/// Nice ceiling for an axis: round up to a "round" number.
pub fn nice_max(v: f64) -> f64 {
    if v <= 0.0 {
        return 1.0;
    }
    let mag = 10f64.powf(v.log10().floor());
    let norm = v / mag;
    let ceil = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    };
    ceil * mag
}
