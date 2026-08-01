use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData};
use crate::prims::{self, HalfCanvas};

/// Shared cartesian plot machinery: pixel mapping, grid, axes, line drawing.
pub struct Plot {
    pub canvas: HalfCanvas,
    pub rect: Rect, // plot area inside the buffer (canvas.blit target)
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    /// Scale used for gridlines / axis labels. When it differs from the data
    /// scale (parallax), gridlines sit on a "far" layer that lags the data.
    pub grid: Axis,
}

impl Plot {
    /// Build a plot over `rect` with the given data bounds.
    pub fn new(rect: Rect, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        let canvas = HalfCanvas::new(rect.width as usize, rect.height as usize);
        Plot {
            canvas,
            rect,
            x_min,
            x_max,
            y_min,
            y_max,
            grid: (x_min, x_max, y_min, y_max),
        }
    }

    /// Map data coords to pixel coords inside the canvas (py is 2x space).
    pub fn px(&self, x: f64, y: f64) -> (i32, i32) {
        let w = (self.canvas.px as f64).max(1.0);
        let h = (self.canvas.py as f64).max(1.0);
        let fx = ((x - self.x_min) / (self.x_max - self.x_min).max(1e-12)).clamp(0.0, 1.0);
        let fy = ((y - self.y_min) / (self.y_max - self.y_min).max(1e-12)).clamp(0.0, 1.0);
        ((fx * w) as i32, ((1.0 - fy) * h) as i32)
    }

    /// Map a coords using the grid scale (far layer).
    fn px_grid(&self, x: f64, y: f64) -> (i32, i32) {
        let w = (self.canvas.px as f64).max(1.0);
        let h = (self.canvas.py as f64).max(1.0);
        let (gx0, gx1, gy0, gy1) = self.grid;
        let fx = ((x - gx0) / (gx1 - gx0).max(1e-12)).clamp(0.0, 1.0);
        let fy = ((y - gy0) / (gy1 - gy0).max(1e-12)).clamp(0.0, 1.0);
        ((fx * w) as i32, ((1.0 - fy) * h) as i32)
    }

    /// Draw vertical + horizontal gridlines in the reserved GRID_LINE color.
    /// The rasterizer draws these as thin solid lines (`│` / `─`), not solid
    /// blocks, so they read as faint hairlines.
    pub fn draw_grid(&mut self, x_ticks: &[f64], y_ticks: &[f64], color: Color) {
        for &t in x_ticks {
            let (x, _) = self.px_grid(t, self.grid.2);
            self.canvas.vline(x, 0, self.canvas.py as i32 - 1, color);
        }
        for &t in y_ticks {
            let (_, y) = self.px_grid(self.grid.0, t);
            self.canvas.hline_px(0, self.canvas.px as i32 - 1, y, color);
        }
    }

    /// Draw a polyline through data points.
    pub fn polyline(&mut self, pts: &[(f64, f64)], color: Color) {
        if pts.len() < 2 {
            return;
        }
        for i in 0..pts.len() - 1 {
            let (x1, y1) = self.px(pts[i].0, pts[i].1);
            let (x2, y2) = self.px(pts[i + 1].0, pts[i + 1].1);
            self.line(x1, y1, x2, y2, color);
        }
    }

    /// Bresenham line in pixel space.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.canvas.set_px(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Fill below a polyline with a vertical gradient (fades toward the top).
    pub fn fill_gradient(&mut self, pts: &[(f64, f64)], baseline: f64, color: Color, top_blend: f64) {
        if pts.len() < 2 {
            return;
        }
        let base_py = self.px(baseline, self.y_min).1;
        // Determine per-column top of the filled region.
        let cols = self.canvas.px as usize;
        let mut top_py = vec![base_py; cols];
        for w in pts.windows(2) {
            let (x1, y1) = self.px(w[0].0, w[0].1);
            let (x2, y2) = self.px(w[1].0, w[1].1);
            let dx = (x2 - x1).abs().max(1);
            for i in 0..=dx {
                let x = x1 + i * (x2 - x1).signum();
                if x >= 0 && (x as usize) < cols {
                    let t = i as f64 / dx as f64;
                    let y = (y1 as f64 + (y2 - y1) as f64 * t).round() as i32;
                    top_py[x as usize] = top_py[x as usize].min(y);
                }
            }
        }
        for (x, &top) in top_py.iter().enumerate() {
            let h = base_py - top;
            if h <= 0 {
                continue;
            }
            for k in 0..h {
                let y = top + k;
                let frac = k as f64 / h as f64; // 0 near top, 1 at baseline
                // blend color toward black near the top for a fade
                let blend = frac * (1.0 - top_blend) + top_blend;
                self.canvas.set_px(x as i32, y, shade(color, blend));
            }
        }
    }

    /// Blit into the buffer, then optionally draw an axis baseline + labels.
    pub fn render(&mut self, buf: &mut Buffer) {
        self.canvas.blit(buf, self.rect);
    }
}

/// Blend a color toward black by `frac` (0 = pure, 1 = black).
pub fn shade(color: Color, frac: f64) -> Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::LightCyan => (0x00, 0xFF, 0xFF),
        Color::Cyan => (0x00, 0xD7, 0xFF),
        Color::LightMagenta => (0xFF, 0x00, 0xFF),
        Color::LightYellow => (0xFF, 0xFF, 0x00),
        Color::LightGreen => (0x00, 0xFF, 0x00),
        Color::LightBlue => (0x00, 0x7F, 0xFF),
        Color::White => (0xFF, 0xFF, 0xFF),
        Color::Gray => (0x80, 0x80, 0x80),
        Color::DarkGray => (0x40, 0x40, 0x40),
        Color::Black => (0, 0, 0),
        _ => (0x00, 0xAF, 0xDF),
    };
    let f = frac.clamp(0.0, 1.0);
    Color::Rgb(
        (r as f64 * f) as u8,
        (g as f64 * f) as u8,
        (b as f64 * f) as u8,
    )
}

/// Dim a color toward black by a fixed factor (for halos).
pub fn shade_dim(c: Color) -> Color {
    shade(c, 0.45)
}

/// Compute a sensible tick step for a range.
pub fn ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    let span = (max - min).abs();
    if span <= 0.0 {
        return vec![min];
    }
    let rough = span / count as f64;
    let mag = 10f64.powf(rough.log10().floor());
    let norm = rough / mag;
    let step = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    } * mag;
    let mut out = Vec::new();
    let start = (min / step).floor() * step;
    let mut v = start;
    let n = 0;
    let mut guard = 0;
    while v <= max + step * 0.01 && guard < 64 {
        out.push(v);
        v += step;
        guard += 1;
    }
    let _ = n;
    out
}

/// Wrap the whole chart in a frame and return the inner plot area.
pub fn framed(buf: &mut Buffer, area: Rect, title: &str, color: Color) -> Rect {
    prims::frame(buf, area, title, color);
    // leave a 1px margin, plus 2 rows for bottom labels + 1 col for left labels
    let inner = Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(3), area.height.saturating_sub(3));
    // plot excludes the bottom 2 rows used for category labels
    let plot = Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(1), inner.height.saturating_sub(2));
    plot
}

/// Draw category labels under a plot (one per slot, spread evenly).
pub fn x_labels(buf: &mut Buffer, area: Rect, plot: &Rect, labels: &[String], color: Color) {
    let n = labels.len();
    if n == 0 {
        return;
    }
    let row = plot.y + plot.height;
    if row >= area.y + area.height {
        return;
    }
    let w = plot.width as usize;
    let items = prims::label_items(labels);
    let chosen = prims::labels_at(w, 4, &items);
    for item in chosen {
        let x = if n <= 1 {
            0
        } else {
            ((item.index as f64 / (n - 1) as f64) * (w.saturating_sub(item.text.len()) as f64)) as u16
        };
        for (i, ch) in item.text.chars().enumerate() {
            let col = plot.x + x + i as u16;
            if col < area.x + area.width {
                buf[(col, row)].set_symbol(&ch.to_string()).set_style(prims::fg(color));
            }
        }
    }
}

/// Draw y-axis value labels along the left edge, positioned on the grid scale.
pub fn y_labels(buf: &mut Buffer, plot: &Rect, y_ticks: &[f64], grid: Axis, color: Color) {
    let h = plot.height as f64;
    let (_, _, gy0, gy1) = grid;
    for &t in y_ticks {
        let fy = ((t - gy0) / (gy1 - gy0).max(1e-12)).clamp(0.0, 1.0);
        let row = (plot.y as f64 + (1.0 - fy) * (h - 1.0)).round() as u16;
        if row < plot.y + plot.height {
            let label = prims::fmt(t);
            let x = plot.x.saturating_sub(label.len() as u16 + 1);
            for (i, ch) in label.chars().enumerate() {
                buf[(x + i as u16, row)].set_symbol(&ch.to_string()).set_style(prims::fg(color));
            }
        }
    }
}

/// Override a plot's y-range from the engine's animated data scale.
pub fn scale_override_ymin(data: &ChartData, y_min: f64, y_max: f64) -> (f64, f64) {
    if let Some(s) = data.scale {
        (s.2, s.3)
    } else {
        (y_min, y_max)
    }
}

/// Override a plot's x/y max from the engine's animated data scale when
/// present (glides the axis during transitions instead of snapping).
pub fn scale_override(data: &ChartData, x_max: f64, y_max: f64) -> (f64, f64) {
    if let Some(s) = data.scale {
        (s.1, s.3)
    } else {
        (x_max, y_max)
    }
}

/// Override the plot's grid scale from the engine's animated grid scale.
pub fn apply_grid(data: &ChartData, p: &mut Plot) {
    if let Some(g) = data.grid_scale {
        p.grid = g;
    }
}

/// Smoothing density adapts to the plot width so wide terminals get finer
/// curves while small ones stay cheap (resolution-adaptive detail).
pub fn smooth_density(width: u16) -> usize {
    (width as usize / 12).clamp(4, 48)
}

/// Smooth a series with Catmull-Rom, producing `density` samples between points.
pub fn smooth(pts: &[(f64, f64)], density: usize) -> Vec<(f64, f64)> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let mut out = Vec::new();
    for i in 0..pts.len() - 1 {
        let p0 = if i > 0 { pts[i - 1] } else { pts[i] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < pts.len() { pts[i + 2] } else { pts[i + 1] };
        for s in 0..density {
            let t = s as f64 / density as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let x = 0.5 * ((2.0 * p1.0) + (-p0.0 + p2.0) * t + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2 + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
            let y = 0.5 * ((2.0 * p1.1) + (-p0.1 + p2.1) * t + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2 + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
            out.push((x, y));
        }
    }
    out.push(*pts.last().unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plot_grid_scale_independent_of_data_scale() {
        // Parallax: data maps on the data scale while gridlines map on the
        // grid scale. With grid lagging, the same tick lands on a different
        // pixel row than the equivalent data value.
        let mut p = Plot::new(Rect::new(0, 0, 10, 10), 0.0, 100.0, 0.0, 100.0);
        p.grid = (0.0, 100.0, 0.0, 200.0); // far layer stretched (lags)
        let (_, data_y) = p.px(0.0, 100.0);
        // data 100 on data scale 0..100 -> top of canvas (py small)
        assert_eq!(data_y, 0);
        // gridline for value 100 on grid scale 0..200 -> middle of canvas
        let (_, grid_y) = p.px_grid(0.0, 100.0);
        assert!(grid_y > data_y, "grid line {} should lag behind data {}", grid_y, data_y);
    }

    #[test]
    fn scale_override_uses_injected_axis() {
        let d = ChartData {
            title: String::new(),
            labels: vec![],
            series: vec![],
            tree: vec![],
            edges: vec![],
            scale: Some((0.0, 50.0, 0.0, 500.0)),
            grid_scale: Some((0.0, 40.0, 0.0, 400.0)),
        };
        let (xm, ym) = scale_override(&d, 10.0, 100.0);
        assert_eq!((xm, ym), (50.0, 500.0));
        let (ymin, ymax) = scale_override_ymin(&d, 5.0, 100.0);
        assert_eq!((ymin, ymax), (0.0, 500.0));
        assert_eq!(d.grid_scale.unwrap().3, 400.0);
    }
}
