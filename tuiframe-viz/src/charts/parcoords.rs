use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct Parcoords;

impl Chart for Parcoords {
    fn name(&self) -> &'static str {
        "parcoords"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Car Dimensions",
                &["MPG", "HP", "Weight", "Price"],
                &[
                    ("Car1", &[30.0, 120.0, 1800.0, 25.0]),
                    ("Car2", &[22.0, 200.0, 2400.0, 40.0]),
                    ("Car3", &[35.0, 90.0, 1500.0, 18.0]),
                    ("Car4", &[18.0, 300.0, 3200.0, 65.0]),
                    ("Car5", &[28.0, 140.0, 2000.0, 30.0]),
                    ("Car6", &[25.0, 170.0, 2100.0, 35.0]),
                    ("Car7", &[32.0, 110.0, 1600.0, 22.0]),
                ],
            ),
            ds(
                "Athlete Performance",
                &["Speed", "Strength", "Stamina", "Agility"],
                &[
                    ("A", &[80.0, 60.0, 70.0, 90.0]),
                    ("B", &[70.0, 85.0, 65.0, 60.0]),
                    ("C", &[90.0, 55.0, 80.0, 75.0]),
                    ("D", &[60.0, 90.0, 75.0, 50.0]),
                    ("E", &[75.0, 70.0, 90.0, 80.0]),
                ],
            ),
            ds(
                "Laptop Specs",
                &["CPU", "RAM", "SSD", "Weight", "Price"],
                &[
                    ("L1", &[8.0, 16.0, 512.0, 1.4, 900.0]),
                    ("L2", &[10.0, 32.0, 1024.0, 1.8, 1400.0]),
                    ("L3", &[6.0, 8.0, 256.0, 1.2, 650.0]),
                    ("L4", &[12.0, 64.0, 2048.0, 2.2, 2200.0]),
                    ("L5", &[9.0, 16.0, 512.0, 1.6, 1100.0]),
                    ("L6", &[7.0, 8.0, 128.0, 1.1, 500.0]),
                ],
            ),
            ds(
                "Wine Characteristics",
                &["Body", "Sweet", "Acid", "Tannin", "Score"],
                &[
                    ("W1", &[8.0, 3.0, 4.0, 7.0, 92.0]),
                    ("W2", &[5.0, 6.0, 5.0, 3.0, 85.0]),
                    ("W3", &[9.0, 2.0, 3.0, 8.0, 95.0]),
                    ("W4", &[4.0, 8.0, 6.0, 2.0, 80.0]),
                    ("W5", &[7.0, 4.0, 4.0, 6.0, 90.0]),
                    ("W6", &[6.0, 5.0, 5.0, 5.0, 88.0]),
                ],
            ),
            ds(
                "Company Performance",
                &["Revenue", "Growth", "Margin", "Employees", "R&D"],
                &[
                    ("C1", &[120.0, 25.0, 18.0, 300.0, 15.0]),
                    ("C2", &[80.0, 15.0, 22.0, 180.0, 10.0]),
                    ("C3", &[200.0, 35.0, 12.0, 800.0, 25.0]),
                    ("C4", &[60.0, 10.0, 28.0, 120.0, 8.0]),
                    ("C5", &[150.0, 30.0, 15.0, 500.0, 20.0]),
                    ("C6", &[95.0, 20.0, 20.0, 250.0, 12.0]),
                ],
            ),
            ds(
                "City Metrics",
                &["Size", "Cost", "Crime", "Parks", "Transit"],
                &[
                    ("City A", &[8.0, 9.0, 3.0, 7.0, 9.0]),
                    ("City B", &[6.0, 6.0, 5.0, 5.0, 6.0]),
                    ("City C", &[9.0, 7.0, 4.0, 8.0, 8.0]),
                    ("City D", &[4.0, 5.0, 7.0, 3.0, 4.0]),
                    ("City E", &[7.0, 8.0, 2.0, 6.0, 7.0]),
                ],
            ),
            ds(
                "Phone Benchmarks",
                &["Battery", "Screen", "Camera", "Speed", "Price"],
                &[
                    ("P1", &[8.0, 7.0, 6.0, 8.0, 700.0]),
                    ("P2", &[6.0, 9.0, 9.0, 9.0, 1100.0]),
                    ("P3", &[9.0, 6.0, 5.0, 6.0, 400.0]),
                    ("P4", &[7.0, 8.0, 8.0, 7.0, 850.0]),
                    ("P5", &[5.0, 6.0, 7.0, 8.0, 600.0]),
                    ("P6", &[8.5, 8.0, 7.5, 8.5, 950.0]),
                ],
            ),
            ds(
                "Dog Breeds",
                &["Size", "Energy", "Train", "Friendly", "Grooming"],
                &[
                    ("D1", &[9.0, 5.0, 4.0, 7.0, 8.0]),
                    ("D2", &[3.0, 8.0, 6.0, 9.0, 3.0]),
                    ("D3", &[7.0, 9.0, 5.0, 6.0, 5.0]),
                    ("D4", &[5.0, 6.0, 9.0, 8.0, 7.0]),
                    ("D5", &[8.0, 4.0, 7.0, 5.0, 9.0]),
                    ("D6", &[4.0, 7.0, 5.0, 9.0, 4.0]),
                ],
            ),
            ds(
                "Employee Reviews",
                &["Attitude", "Skill", "Teamwork", "Output", "Years"],
                &[
                    ("E1", &[8.0, 7.0, 9.0, 7.0, 3.0]),
                    ("E2", &[6.0, 9.0, 6.0, 9.0, 6.0]),
                    ("E3", &[9.0, 6.0, 8.0, 6.0, 2.0]),
                    ("E4", &[7.0, 8.0, 7.0, 8.0, 5.0]),
                    ("E5", &[5.0, 5.0, 5.0, 5.0, 1.0]),
                    ("E6", &[8.5, 8.0, 8.5, 8.0, 7.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(2);
        let title = if data.title.is_empty() { "Parallel Coordinates" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.series.is_empty() {
            prims::text(buf, inner, 1, "no data", prims::DIM);
            return;
        }
        let labels = if data.labels.is_empty() {
            (0..data.series[0].values.len()).map(|i| format!("D{}", i + 1)).collect::<Vec<_>>()
        } else {
            data.labels.clone()
        };
        let dims = labels.len();
        let n = data.series.len();
        if dims == 0 {
            return;
        }
        // per-dimension min/max
        let mut mins = vec![f64::INFINITY; dims];
        let mut maxs = vec![f64::NEG_INFINITY; dims];
        for s in &data.series {
            for (i, &v) in s.values.iter().enumerate() {
                if i < dims {
                    mins[i] = mins[i].min(v);
                    maxs[i] = maxs[i].max(v);
                }
            }
        }
        for i in 0..dims {
            if !mins[i].is_finite() {
                mins[i] = 0.0;
            }
            if !maxs[i].is_finite() {
                maxs[i] = 1.0;
            }
            if (maxs[i] - mins[i]).abs() < 1e-12 {
                maxs[i] = mins[i] + 1.0;
            }
        }
        let plot_h = inner.height.saturating_sub(2);
        let plot_w = inner.width;
        let plot_y = inner.y + 1;

        // axis lines
        for i in 0..dims {
            let x = inner.x + (i as f64 / (dims - 1).max(1) as f64 * plot_w as f64).round() as u16;
            for y in plot_y..(plot_y + plot_h).min(area.y + area.height) {
                buf[(x, y)].set_symbol("│").set_fg(Color::DarkGray);
            }
            // labels
            let lbl = &labels[i];
            let lx = x
                .saturating_sub(lbl.chars().count() as u16 / 2)
                .min(inner.x + inner.width.saturating_sub(lbl.chars().count() as u16));
            prims::abs_text(buf, lx, inner.y - 1, lbl, prims::fg(color));
            // min/max at bottom/top
            let min_s = prims::fmt(mins[i]);
            let max_s = prims::fmt(maxs[i]);
            let mx = x
                .saturating_sub(max_s.chars().count() as u16)
                .min(inner.x + inner.width.saturating_sub(max_s.chars().count() as u16));
            let mnx = x
                .saturating_sub(min_s.chars().count() as u16)
                .min(inner.x + inner.width.saturating_sub(min_s.chars().count() as u16));
            prims::abs_text(buf, mx, inner.y, &max_s, prims::DIM);
            prims::abs_text(buf, mnx, inner.y + plot_h + 1, &min_s, prims::DIM);
        }

        // series polylines
        for (si, s) in data.series.iter().enumerate() {
            let c = prims::series_color(si, data.series.len());
            let pts: Vec<(u16, u16)> = s
                .values
                .iter()
                .enumerate()
                .take(dims) // interpolated series may be longer than the axes
                .map(|(i, &v)| {
                    let x = inner.x + (i as f64 / (dims - 1).max(1) as f64 * plot_w as f64).round() as u16;
                    let fy = ((v - mins[i]) / (maxs[i] - mins[i])).clamp(0.0, 1.0);
                    let y = plot_y + ((1.0 - fy) * (plot_h - 1) as f64).round() as u16;
                    (x, y)
                })
                .collect();
            for i in 0..pts.len().saturating_sub(1) {
                draw_line(buf, area, pts[i].0, pts[i].1, pts[i + 1].0, pts[i + 1].1, c);
            }
        }
        let _ = n;
    }
}

fn draw_line(buf: &mut Buffer, area: Rect, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 && (x as u16) < area.width && (y as u16) < area.height {
            buf[(x as u16, y as u16)].set_symbol("·").set_fg(color);
        }
        if x == x1 as i32 && y == y1 as i32 {
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
