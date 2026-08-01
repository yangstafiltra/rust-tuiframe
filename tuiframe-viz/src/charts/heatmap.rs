use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct Heatmap;

impl Chart for Heatmap {
    fn name(&self) -> &'static str {
        "heatmap"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.series.is_empty() || data.series[0].values.is_empty() {
            return None;
        }
        let max_v = data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0_f64, f64::max).max(1e-9);
        Some((0.0, 1.0, 0.0, max_v))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "User Activity (week x hour)",
                &["0h", "6h", "12h", "18h", "24h"],
                &[
                    ("Mon", &[2.0, 5.0, 30.0, 60.0, 20.0]),
                    ("Tue", &[3.0, 8.0, 40.0, 70.0, 15.0]),
                    ("Wed", &[1.0, 6.0, 35.0, 65.0, 18.0]),
                    ("Thu", &[4.0, 10.0, 50.0, 80.0, 22.0]),
                    ("Fri", &[5.0, 12.0, 45.0, 75.0, 25.0]),
                    ("Sat", &[8.0, 20.0, 55.0, 40.0, 10.0]),
                    ("Sun", &[6.0, 15.0, 25.0, 20.0, 5.0]),
                ],
            ),
            ds(
                "Correlation Matrix",
                &["CPU", "MEM", "IO", "NET"],
                &[
                    ("CPU", &[1.0, 0.6, 0.8, 0.3]),
                    ("MEM", &[0.6, 1.0, 0.5, 0.4]),
                    ("IO", &[0.8, 0.5, 1.0, 0.7]),
                    ("NET", &[0.3, 0.4, 0.7, 1.0]),
                ],
            ),
            ds(
                "Server Load (node x hour)",
                &["00h", "04h", "08h", "12h", "16h", "20h"],
                &[
                    ("node1", &[10.0, 15.0, 45.0, 70.0, 55.0, 30.0]),
                    ("node2", &[8.0, 12.0, 50.0, 80.0, 60.0, 25.0]),
                    ("node3", &[15.0, 20.0, 40.0, 65.0, 50.0, 35.0]),
                    ("node4", &[5.0, 10.0, 35.0, 55.0, 45.0, 20.0]),
                    ("node5", &[20.0, 25.0, 55.0, 75.0, 70.0, 40.0]),
                ],
            ),
            ds(
                "Sales by Region x Product",
                &["Prod A", "Prod B", "Prod C", "Prod D"],
                &[
                    ("North", &[80.0, 60.0, 40.0, 20.0]),
                    ("South", &[50.0, 70.0, 30.0, 60.0]),
                    ("East", &[90.0, 40.0, 70.0, 30.0]),
                    ("West", &[60.0, 80.0, 50.0, 40.0]),
                    ("Center", &[30.0, 50.0, 60.0, 70.0]),
                ],
            ),
            ds(
                "Seat Occupancy (week)",
                &["Mon", "Tue", "Wed", "Thu", "Fri"],
                &[
                    ("Floor 1", &[30.0, 55.0, 40.0, 65.0, 25.0]),
                    ("Floor 2", &[45.0, 70.0, 50.0, 80.0, 35.0]),
                    ("Floor 3", &[60.0, 40.0, 75.0, 45.0, 55.0]),
                    ("Floor 4", &[20.0, 35.0, 30.0, 50.0, 15.0]),
                    ("Floor 5", &[70.0, 60.0, 55.0, 30.0, 40.0]),
                ],
            ),
            ds(
                "Test Scores (student x test)",
                &["T1", "T2", "T3", "T4", "T5", "T6"],
                &[
                    ("Amy", &[55.0, 70.0, 80.0, 65.0, 90.0, 75.0]),
                    ("Ben", &[60.0, 55.0, 75.0, 85.0, 70.0, 80.0]),
                    ("Cy", &[75.0, 80.0, 65.0, 70.0, 55.0, 85.0]),
                    ("Dan", &[40.0, 60.0, 70.0, 50.0, 65.0, 55.0]),
                    ("Eve", &[85.0, 75.0, 90.0, 80.0, 95.0, 88.0]),
                ],
            ),
            ds(
                "Weather (city x month)",
                &["Jan", "Apr", "Jul", "Oct"],
                &[
                    ("NYC", &[2.0, 12.0, 26.0, 15.0]),
                    ("LA", &[14.0, 16.0, 24.0, 20.0]),
                    ("CHI", &[1.0, 10.0, 24.0, 12.0]),
                    ("MIA", &[20.0, 24.0, 29.0, 26.0]),
                    ("DEN", &[0.0, 8.0, 22.0, 10.0]),
                ],
            ),
            ds(
                "Site Traffic (day x hour)",
                &["6h", "10h", "14h", "18h", "22h"],
                &[
                    ("Mon", &[5.0, 40.0, 55.0, 70.0, 25.0]),
                    ("Tue", &[8.0, 45.0, 60.0, 65.0, 30.0]),
                    ("Wed", &[6.0, 50.0, 58.0, 68.0, 22.0]),
                    ("Thu", &[9.0, 48.0, 62.0, 75.0, 35.0]),
                    ("Fri", &[10.0, 52.0, 65.0, 80.0, 40.0]),
                    ("Sat", &[12.0, 35.0, 50.0, 45.0, 28.0]),
                    ("Sun", &[4.0, 30.0, 42.0, 38.0, 20.0]),
                ],
            ),
            ds(
                "Error Count (service x code)",
                &["4xx", "5xx", "Timeout"],
                &[
                    ("API", &[15.0, 8.0, 3.0]),
                    ("Web", &[22.0, 5.0, 1.0]),
                    ("DB", &[2.0, 9.0, 6.0]),
                    ("Cache", &[6.0, 2.0, 4.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(0);
        let title = if data.title.is_empty() { "Heatmap" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.series.is_empty() || data.series[0].values.is_empty() {
            prims::text(buf, inner, 1, "no data", prims::DIM);
            return;
        }
        let rows = data.series.len();
        let cols = data.series.iter().map(|s| s.values.len()).max().unwrap_or(0);
        if rows == 0 || cols == 0 {
            prims::text(buf, inner, 1, "no data", prims::DIM);
            return;
        }
        // Resolution-adaptive subdivision: every character pixel of the data
        // area samples the rows x cols grid through bilinear interpolation, so
        // a taller/wider terminal shows finer, smoother gradation — the same
        // spirit as bezier density_for(width).
        let max_v = data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0_f64, f64::max).max(1e-9);
        let max_v = data.scale.map(|s| s.3).unwrap_or(max_v);

        let label_w = 10.min(inner.width / 5);
        let data_w = inner.width.saturating_sub(label_w);
        let data_h = inner.height.saturating_sub(2);

        // Row labels at the vertical centre of each data row.
        for r in 0..rows {
            let y = inner.y + ((r as f64 + 0.5) / rows as f64 * data_h as f64).floor() as u16;
            let name = &data.series[r].name;
            let name = if name.is_empty() { format!("Row {}", r + 1) } else { name.clone() };
            let label = if name.chars().count() as u16 > label_w {
                name.chars().take(label_w as usize).collect::<String>()
            } else {
                name
            };
            prims::abs_text(buf, inner.x, y, &label, prims::WHITE);
        }

        // Data cells: bilinear sample per pixel.
        for py in 0..data_h {
            for px in 0..data_w {
                let v = sample_grid(&data.series, rows, cols, px, py, data_w, data_h);
                let frac = (v / max_v).clamp(0.0, 1.0);
                let (cr, cg, cb) = heat_color(frac);
                buf[(inner.x + label_w + px, inner.y + py)].set_symbol("█").set_fg(Color::Rgb(cr, cg, cb));
            }
        }

        // Column labels below the data area.
        let label_y = inner.y + data_h;
        for c in 0..cols {
            let x = inner.x + label_w + ((c as f64 + 0.5) / cols as f64 * data_w as f64).floor() as u16;
            if x >= inner.x + label_w + data_w {
                continue;
            }
            let l = data.labels.get(c).cloned().unwrap_or_else(|| c.to_string());
            let avail = (inner.x + label_w + data_w - x) as usize;
            let l = if l.chars().count() > avail { l.chars().take(avail).collect::<String>() } else { l };
            prims::abs_text(buf, x, label_y, &l, prims::DIM);
        }
        // legend
        let mut lx = inner.x;
        let ly = label_y + 1;
        if ly < inner.y + inner.height {
            prims::abs_text(buf, lx, ly, "low", prims::DIM);
            lx += 3;
            for frac in (0..=10).map(|i| i as f64 / 10.0) {
                let (cr, cg, cb) = heat_color(frac);
                if lx < inner.x + inner.width {
                    buf[(lx, ly)].set_symbol("█").set_fg(Color::Rgb(cr, cg, cb));
                    lx += 1;
                }
            }
            prims::abs_text(buf, lx, ly, "high", prims::DIM);
        }
    }
}

/// Bilinear interpolation of the (rows x cols) grid at a pixel position.
/// `u = px / data_w` maps onto the cell columns; `v = py / data_h` onto rows.
/// Rows shorter than `cols` read as 0.0 (mid-transition safety).
fn sample_grid(data: &[crate::data::Series], rows: usize, cols: usize, px: u16, py: u16, data_w: u16, data_h: u16) -> f64 {
    let u = (px as f64 + 0.5) / data_w.max(1) as f64 * cols as f64;
    let v = (py as f64 + 0.5) / data_h.max(1) as f64 * rows as f64;
    let x = (u - 0.5).clamp(0.0, (cols - 1) as f64);
    let y = (v - 0.5).clamp(0.0, (rows - 1) as f64);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(cols - 1);
    let y1 = (y0 + 1).min(rows - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let v00 = data[y0].values.get(x0).copied().unwrap_or(0.0);
    let v10 = data[y0].values.get(x1).copied().unwrap_or(0.0);
    let v01 = data[y1].values.get(x0).copied().unwrap_or(0.0);
    let v11 = data[y1].values.get(x1).copied().unwrap_or(0.0);
    v00 * (1.0 - fx) * (1.0 - fy) + v10 * fx * (1.0 - fy) + v01 * (1.0 - fx) * fy + v11 * fx * fy
}

fn heat_color(frac: f64) -> (u8, u8, u8) {
    // blue -> cyan -> yellow -> red
    let f = frac.clamp(0.0, 1.0);
    let stops = [
        (0.0, (40, 60, 180)),
        (0.33, (40, 160, 220)),
        (0.66, (240, 220, 60)),
        (1.0, (220, 50, 40)),
    ];
    let mut seg = 0;
    for i in 0..stops.len() - 1 {
        if f >= stops[i].0 && f <= stops[i + 1].0 {
            seg = i;
            break;
        }
    }
    let (t0, c0) = stops[seg];
    let (t1, c1) = stops[seg + 1];
    let t = if t1 > t0 { ((f - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { 0.0 };
    (
        (c0.0 as f64 + (c1.0 as f64 - c0.0 as f64) * t) as u8,
        (c0.1 as f64 + (c1.1 as f64 - c0.1 as f64) * t) as u8,
        (c0.2 as f64 + (c1.2 as f64 - c0.2 as f64) * t) as u8,
    )
}
