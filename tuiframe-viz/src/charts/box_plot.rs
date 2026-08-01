use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override, ticks, x_labels, y_labels};

pub struct BoxPlot;

impl Chart for BoxPlot {
    fn name(&self) -> &'static str {
        "box_plot"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        let all: Vec<f64> = data.series.iter().flat_map(|s| s.values.iter().copied()).collect();
        if all.is_empty() {
            return None;
        }
        let y_max = prims::nice_max(all.iter().cloned().fold(0.0_f64, f64::max));
        Some((0.0, data.series.len() as f64, 0.0, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Test Scores by Class",
                &["Class A", "Class B", "Class C"],
                &[
                    ("A", &[55.0, 60.0, 65.0, 70.0, 72.0, 78.0, 80.0, 85.0, 90.0, 95.0]),
                    ("B", &[40.0, 52.0, 58.0, 62.0, 68.0, 74.0, 82.0, 88.0]),
                    ("C", &[60.0, 66.0, 70.0, 75.0, 79.0, 84.0, 91.0]),
                ],
            ),
            ds(
                "Latency by Region (ms)",
                &["NA", "EU", "APAC"],
                &[
                    ("NA", &[12.0, 15.0, 18.0, 22.0, 30.0, 45.0]),
                    ("EU", &[20.0, 28.0, 32.0, 40.0, 55.0, 80.0, 120.0]),
                    ("APAC", &[50.0, 65.0, 80.0, 100.0, 140.0, 200.0]),
                ],
            ),
            ds(
                "Exam Grades by Subject",
                &["Math", "Physics", "Chem"],
                &[
                    ("Math", &[45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0]),
                    ("Physics", &[40.0, 48.0, 56.0, 60.0, 68.0, 74.0, 78.0, 84.0]),
                    ("Chem", &[52.0, 58.0, 62.0, 70.0, 76.0, 82.0, 88.0, 92.0]),
                ],
            ),
            ds(
                "Delivery Time by Courier (h)",
                &["Courier A", "Courier B", "Courier C", "Courier D"],
                &[
                    ("Courier A", &[3.0, 4.0, 5.0, 6.0, 8.0, 10.0]),
                    ("Courier B", &[2.0, 3.0, 3.5, 5.0, 7.0]),
                    ("Courier C", &[5.0, 6.0, 8.0, 10.0, 12.0, 15.0]),
                    ("Courier D", &[2.5, 4.0, 4.5, 6.0, 9.0]),
                ],
            ),
            ds(
                "Sensor Temp by Zone (C)",
                &["Zone1", "Zone2", "Zone3"],
                &[
                    ("Zone1", &[18.0, 19.0, 21.0, 22.0, 24.0, 26.0]),
                    ("Zone2", &[15.0, 17.0, 18.0, 20.0, 21.0, 23.0, 25.0]),
                    ("Zone3", &[20.0, 22.0, 24.0, 26.0, 28.0, 30.0]),
                ],
            ),
            ds(
                "App Ratings by Platform",
                &["iOS", "Android", "Web"],
                &[
                    ("iOS", &[3.0, 3.5, 4.0, 4.5, 5.0, 4.2, 4.8]),
                    ("Android", &[2.5, 3.0, 3.5, 4.0, 4.5, 4.7]),
                    ("Web", &[3.0, 3.4, 3.8, 4.1, 4.4, 4.6, 4.9]),
                ],
            ),
            ds(
                "Battery Life by Brand (h)",
                &["Brand A", "Brand B", "Brand C"],
                &[
                    ("Brand A", &[5.0, 6.0, 6.5, 7.0, 8.0]),
                    ("Brand B", &[4.0, 4.5, 5.0, 6.0, 7.5]),
                    ("Brand C", &[6.0, 7.0, 7.5, 8.5, 9.5]),
                ],
            ),
            ds(
                "Salary by Role (k$)",
                &["Junior", "Mid", "Senior", "Lead"],
                &[
                    ("Junior", &[40.0, 45.0, 48.0, 55.0]),
                    ("Mid", &[60.0, 65.0, 70.0, 78.0]),
                    ("Senior", &[85.0, 90.0, 100.0, 115.0]),
                    ("Lead", &[110.0, 120.0, 130.0, 150.0]),
                ],
            ),
            ds(
                "Wait Time by Station (min)",
                &["S1", "S2", "S3"],
                &[
                    ("S1", &[1.0, 2.0, 3.0, 5.0, 7.0]),
                    ("S2", &[2.0, 3.0, 4.0, 6.0, 9.0]),
                    ("S3", &[1.5, 2.5, 3.5, 5.5, 8.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(5);
        let title = if data.title.is_empty() { "Box Plot" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.series.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let groups = data.series.len();
        let all: Vec<f64> = data.series.iter().flat_map(|s| s.values.iter().copied()).collect();
        if all.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let y_max = prims::nice_max(all.iter().cloned().fold(0.0_f64, f64::max));
        let y_t = ticks(0.0, y_max, 5);
        let (gx, y_max) = scale_override(data, groups as f64, y_max);
        let mut p = Plot::new(plot, 0.0, gx, 0.0, y_max);
        apply_grid(data, &mut p);
        p.draw_grid(&[], &y_t, crate::prims::GRID_LINE);

        for (g, s) in data.series.iter().enumerate() {
            let mut vals: Vec<f64> = s.values.clone();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |f: f64| -> f64 {
                if vals.is_empty() {
                    return 0.0;
                }
                let idx = f * (vals.len() - 1) as f64;
                let lo = idx.floor() as usize;
                let hi = idx.ceil() as usize;
                if lo == hi {
                    vals[lo]
                } else {
                    let t = idx - lo as f64;
                    vals[lo] * (1.0 - t) + vals[hi] * t
                }
            };
            let min = *vals.first().unwrap();
            let max = *vals.last().unwrap();
            let q1 = q(0.25);
            let med = q(0.5);
            let q3 = q(0.75);
            let (cx, _) = p.px(g as f64 + 0.5, 0.0);
            let bw = ((plot.width as f64 / groups as f64) * 0.32) as i32;
            let c = prims::series_color(g, groups);
            // whiskers
            let (_, ymin) = p.px(0.0, min);
            let (_, ymax) = p.px(0.0, max);
            let (_, yq1) = p.px(0.0, q1);
            let (_, ymed) = p.px(0.0, med);
            let (_, yq3) = p.px(0.0, q3);
            p.canvas.vline(cx, ymin, ymax, c);
            // box
            for x in (cx - bw)..=(cx + bw) {
                p.canvas.vline(x, yq1, yq3, c);
            }
            // median highlight
            for x in (cx - bw)..=(cx + bw) {
                p.canvas.set_px(x, ymed, Color::White);
            }
            // whisker caps
            p.canvas.hline_px(cx - bw, cx + bw, ymin, c);
            p.canvas.hline_px(cx - bw, cx + bw, ymax, c);
        }
        p.render(buf);
        x_labels(buf, area, &plot, &data.series.iter().map(|s| if s.name.is_empty() { "?".to_string() } else { s.name.clone() }).collect::<Vec<_>>(), Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}
