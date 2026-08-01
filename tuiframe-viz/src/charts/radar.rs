use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct RadarChart;

impl Chart for RadarChart {
    fn name(&self) -> &'static str {
        "radar_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.is_empty() {
            return None;
        }
        let max_v = data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0_f64, f64::max).max(1e-9);
        Some((0.0, 1.0, 0.0, prims::nice_max(max_v)))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Player Stats",
                &["Attack", "Defense", "Speed", "Magic", "Luck"],
                &[("You", &[80.0, 65.0, 70.0, 90.0, 55.0]), ("Rival", &[70.0, 85.0, 60.0, 60.0, 80.0])],
            ),
            ds(
                "Car Comparison",
                &["Power", "Efficiency", "Safety", "Comfort", "Price", "Tech"],
                &[("Model A", &[85.0, 70.0, 90.0, 75.0, 60.0, 80.0]), ("Model B", &[60.0, 90.0, 80.0, 70.0, 85.0, 65.0])],
            ),
            ds(
                "Team Skills",
                &["Code", "Design", "Ops", "Sales", "Support"],
                &[("Team", &[90.0, 70.0, 80.0, 50.0, 75.0])],
            ),
            ds(
                "Product Attributes",
                &["Speed", "UI", "Reliability", "Price", "Support", "Docs"],
                &[("Us", &[80.0, 85.0, 90.0, 60.0, 70.0, 65.0]), ("Them", &[70.0, 60.0, 75.0, 85.0, 80.0, 75.0])],
            ),
            ds(
                "Athlete Profile",
                &["Speed", "Power", "Endurance", "Agility", "Skill"],
                &[("P1", &[85.0, 70.0, 60.0, 90.0, 75.0]), ("P2", &[65.0, 90.0, 80.0, 60.0, 70.0]), ("P3", &[75.0, 60.0, 95.0, 70.0, 80.0])],
            ),
            ds(
                "Phone Comparison",
                &["Camera", "Battery", "Screen", "Speed", "Price", "Build"],
                &[("A", &[90.0, 70.0, 85.0, 80.0, 55.0, 75.0]), ("B", &[70.0, 90.0, 75.0, 85.0, 65.0, 80.0]), ("C", &[80.0, 60.0, 90.0, 70.0, 85.0, 70.0])],
            ),
            ds(
                "Climate Factors",
                &["Rain", "Sun", "Wind", "Humidity", "Heat"],
                &[("Coast", &[60.0, 70.0, 80.0, 75.0, 65.0]), ("Inland", &[30.0, 90.0, 50.0, 40.0, 85.0]), ("Mountain", &[85.0, 55.0, 70.0, 60.0, 45.0])],
            ),
            ds(
                "Restaurant Ratings",
                &["Food", "Service", "Ambience", "Value", "Clean"],
                &[("R1", &[90.0, 70.0, 80.0, 60.0, 85.0]), ("R2", &[70.0, 85.0, 65.0, 80.0, 75.0]), ("R3", &[80.0, 75.0, 90.0, 70.0, 85.0])],
            ),
            ds(
                "City Quality",
                &["Jobs", "Housing", "Transit", "Nature", "Culture"],
                &[("City X", &[85.0, 55.0, 75.0, 70.0, 80.0]), ("City Y", &[60.0, 80.0, 65.0, 60.0, 60.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(1);
        let title = if data.title.is_empty() { "Radar Chart" } else { &data.title };
        prims::frame(buf, area, title, color);
        if data.series.is_empty() || data.series[0].values.is_empty() {
            prims::text(buf, area, area.height / 2, "no data", prims::DIM);
            return;
        }
        let axes = if data.labels.is_empty() {
            (0..data.series[0].values.len()).map(|i| format!("A{}", i + 1)).collect::<Vec<_>>()
        } else {
            data.labels.clone()
        };
        let n = axes.len();
        let max_v = data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0_f64, f64::max).max(1e-9);
        let max_v = prims::nice_max(max_v);
        let max_v = data
            .scale
            .map(|s| s.3)
            .unwrap_or(max_v);

        // layout
        let legend_w = 24.min(area.width / 3);
        let chart_w = area.width.saturating_sub(legend_w).saturating_sub(2);
        let chart_h = area.height.saturating_sub(2);
        let cx = 2 + chart_w / 2;
        let cy = 1 + chart_h / 2;
        let rad = (chart_w.min(chart_h) as f64 * 0.40) as f64;

        // grid rings
        for ring in 1..=4 {
            let rr = rad * ring as f64 / 4.0;
            let mut prev: Option<(u16, u16)> = None;
            for i in 0..=n {
                let ang = std::f64::consts::FRAC_PI_2 - (i as f64 / n as f64) * std::f64::consts::TAU;
                let x = (cx as f64 + ang.cos() * rr).round() as u16;
                let y = (cy as f64 - ang.sin() * rr).round() as u16;
                if let Some(p) = prev {
                    draw_line(buf, area, p.0, p.1, x, y, Color::DarkGray);
                }
                prev = Some((x, y));
            }
        }
        // spokes
        for i in 0..n {
            let ang = std::f64::consts::FRAC_PI_2 - (i as f64 / n as f64) * std::f64::consts::TAU;
            let x = (cx as f64 + ang.cos() * rad).round() as u16;
            let y = (cy as f64 - ang.sin() * rad).round() as u16;
            draw_line(buf, area, cx, cy, x, y, Color::DarkGray);
        }

        // series polygons
        for (si, s) in data.series.iter().enumerate() {
            let c = prims::series_color(si, data.series.len());
            let pts: Vec<(u16, u16)> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let ang = std::f64::consts::FRAC_PI_2 - (i as f64 / n as f64) * std::f64::consts::TAU;
                    let rr = (v / max_v).clamp(0.0, 1.0) * rad;
                    let x = (cx as f64 + ang.cos() * rr).round() as u16;
                    let y = (cy as f64 - ang.sin() * rr).round() as u16;
                    (x, y)
                })
                .collect();
            // filled polygon
            fill_poly(buf, area, &pts, crate::charts::cartesian::shade(c, 0.45));
            // outline
            for i in 0..pts.len() {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                draw_line(buf, area, a.0, a.1, b.0, b.1, c);
            }
            // vertex dots
            for (x, y) in &pts {
                buf[(*x, *y)].set_symbol("●").set_fg(Color::White);
            }
        }

        // axis labels
        for (i, label) in axes.iter().enumerate() {
            let ang = std::f64::consts::FRAC_PI_2 - (i as f64 / n as f64) * std::f64::consts::TAU;
            let lx = (cx as f64 + ang.cos() * (rad + 2.0)).round() as i32;
            let ly = (cy as f64 - ang.sin() * (rad + 2.0)).round() as i32;
            let txt_x = (lx - (label.chars().count() as i32) / 2).max(0) as u16;
            let txt_y = ly.max(0) as u16;
            if txt_y < area.height && txt_x < area.width {
                prims::abs_text(buf, txt_x, txt_y, label, prims::fg(color));
            }
        }

        // legend
        let lx = area.x + 2 + chart_w + 1;
        let mut ly = area.y + 2;
        for s in &data.series {
            if ly + 1 >= area.y + area.height {
                break;
            }
            let c = prims::series_color(data.series.iter().position(|x| std::ptr::eq(x, s)).unwrap_or(0), data.series.len());
            buf[(lx, ly)].set_symbol("■").set_fg(c);
            for (j, ch) in s.name.chars().enumerate() {
                let col = lx + 1 + j as u16;
                if col < area.x + area.width {
                    buf[(col, ly)].set_symbol(&ch.to_string()).set_style(prims::WHITE);
                }
            }
            ly += 1;
        }
    }
}

fn draw_line(buf: &mut Buffer, area: Rect, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
    if x0 >= area.width || x1 >= area.width || y0 >= area.height || y1 >= area.height {
        return;
    }
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

fn fill_poly(buf: &mut Buffer, area: Rect, pts: &[(u16, u16)], color: Color) {
    let fpts: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
    let ymin = fpts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min).floor() as i32;
    let ymax = fpts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max).ceil() as i32;
    for y in ymin.max(0)..=ymax.min(area.height as i32 - 1) {
        let mut xs: Vec<f64> = Vec::new();
        for i in 0..fpts.len() {
            let (x1, y1) = fpts[i];
            let (x2, y2) = fpts[(i + 1) % fpts.len()];
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
            for x in x0.max(0)..=x1.min(area.width as i32 - 1) {
                buf[(x as u16, y as u16)].set_symbol("█").set_fg(color);
            }
            i += 2;
        }
    }
}
