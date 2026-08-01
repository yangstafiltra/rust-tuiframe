use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override, ticks, x_labels, y_labels};

pub struct ViolinPlot;

impl Chart for ViolinPlot {
    fn name(&self) -> &'static str {
        "violin_plot"
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
                "Salary by Dept",
                &["Eng", "Sales", "Ops"],
                &[
                    ("Eng", &[55.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0, 95.0, 100.0, 110.0, 120.0]),
                    ("Sales", &[40.0, 45.0, 50.0, 55.0, 60.0, 70.0, 80.0, 90.0, 100.0]),
                    ("Ops", &[45.0, 50.0, 55.0, 60.0, 65.0, 70.0]),
                ],
            ),
            ds(
                "Response Times",
                &["Web", "API", "DB"],
                &[
                    ("Web", &[10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 60.0]),
                    ("API", &[20.0, 30.0, 40.0, 50.0, 60.0, 80.0]),
                    ("DB", &[5.0, 10.0, 15.0, 20.0, 30.0, 50.0, 100.0]),
                ],
            ),
            ds(
                "Protein per Food (g)",
                &["Meat", "Fish", "Beans", "Dairy"],
                &[
                    ("Meat", &[20.0, 22.0, 25.0, 26.0, 28.0, 30.0]),
                    ("Fish", &[18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0]),
                    ("Beans", &[8.0, 9.0, 10.0, 11.0, 12.0, 13.0]),
                    ("Dairy", &[3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]),
                ],
            ),
            ds(
                "Run Times (min)",
                &["5K", "10K", "Half"],
                &[
                    ("5K", &[20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0]),
                    ("10K", &[45.0, 50.0, 55.0, 60.0, 65.0]),
                    ("Half", &[100.0, 110.0, 120.0, 130.0, 140.0]),
                ],
            ),
            ds(
                "Package Weight (kg)",
                &["Small", "Medium", "Large"],
                &[
                    ("Small", &[0.2, 0.3, 0.4, 0.5, 0.6, 0.7]),
                    ("Medium", &[1.0, 1.5, 2.0, 2.5, 3.0]),
                    ("Large", &[5.0, 6.0, 7.0, 8.0, 10.0]),
                ],
            ),
            ds(
                "Website Load (ms)",
                &["Mobile", "Desktop", "Tablet"],
                &[
                    ("Mobile", &[200.0, 250.0, 300.0, 350.0, 400.0, 500.0]),
                    ("Desktop", &[100.0, 120.0, 150.0, 180.0, 220.0, 300.0]),
                    ("Tablet", &[150.0, 180.0, 220.0, 260.0, 320.0, 400.0]),
                ],
            ),
            ds(
                "Test Score Distribution",
                &["Math", "Reading", "Writing"],
                &[
                    ("Math", &[50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0]),
                    ("Reading", &[45.0, 52.0, 58.0, 66.0, 72.0, 78.0, 84.0, 92.0]),
                    ("Writing", &[48.0, 56.0, 62.0, 70.0, 76.0, 82.0, 88.0]),
                ],
            ),
            ds(
                "Blood Pressure by Group",
                &["Young", "Middle", "Elderly"],
                &[
                    ("Young", &[110.0, 115.0, 120.0, 122.0, 118.0, 116.0]),
                    ("Middle", &[120.0, 125.0, 128.0, 130.0, 135.0, 126.0, 124.0]),
                    ("Elderly", &[130.0, 138.0, 142.0, 148.0, 152.0, 140.0, 145.0]),
                ],
            ),
            ds(
                "App Session Length (min)",
                &["iOS", "Android", "Web"],
                &[
                    ("iOS", &[2.0, 3.0, 5.0, 8.0, 12.0, 18.0, 25.0]),
                    ("Android", &[1.0, 2.0, 4.0, 7.0, 10.0, 15.0, 22.0]),
                    ("Web", &[3.0, 5.0, 8.0, 10.0, 15.0, 20.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(3);
        let title = if data.title.is_empty() { "Violin Plot" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.series.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let groups = data.series.len();
        let all: Vec<f64> = data.series.iter().flat_map(|s| s.values.iter().copied()).collect();
        if all.is_empty() {
            return;
        }
        let y_min = 0.0;
        let y_max = prims::nice_max(all.iter().cloned().fold(0.0_f64, f64::max));
        let y_t = ticks(y_min, y_max, 5);
        let (gx, y_max) = scale_override(data, groups as f64, y_max);
        let mut p = Plot::new(plot, 0.0, gx, 0.0, y_max);
        apply_grid(data, &mut p);
        p.draw_grid(&[], &y_t, crate::prims::GRID_LINE);

        for (g, s) in data.series.iter().enumerate() {
            let vals = &s.values;
            if vals.is_empty() {
                continue;
            }
            let c = prims::series_color(g, groups);
            let bw = ((plot.width as f64 / groups as f64) * 0.6) as i32;
            let cx_px = p.px(g as f64 + 0.5, 0.0).0;
            // simple KDE-ish histogram into density
            let bins = (plot.height as usize * 2).clamp(8, 48);
            let mut counts = vec![0.0; bins];
            let minv = vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let maxv = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let span = (maxv - minv).max(1e-9);
            for &v in vals {
                let idx = (((v - minv) / span) * (bins - 1) as f64).round() as usize;
                counts[idx.min(bins - 1)] += 1.0;
            }
            let cmax = counts.iter().cloned().fold(1.0_f64, f64::max);
            let row_px = p.canvas.py as i32;
            for (bi, &cnt) in counts.iter().enumerate() {
                let v = minv + (bi as f64 / (bins - 1) as f64) * span;
                let (_, ypx) = p.px(0.0, v);
                let half_w = ((cnt / cmax) * bw as f64 * 0.9) as i32;
                let hw = half_w.max(1);
                // mirror: fill both sides
                for dx in 1..=hw {
                    // shade by density: denser = brighter
                    let dens = dx as f64 / hw as f64;
                    let col = crate::charts::cartesian::shade(c, 0.4 + 0.6 * dens);
                    p.canvas.set_px(cx_px - dx, ypx, col);
                    p.canvas.set_px(cx_px + dx, ypx, col);
                }
                p.canvas.set_px(cx_px, ypx, Color::White);
            }
            // center median marker
            let mut sv: Vec<f64> = vals.clone();
            sv.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = sv[sv.len() / 2];
            let (_, ymed) = p.px(0.0, med);
            p.canvas.hline_px(cx_px - bw, cx_px + bw, ymed, Color::White);
            let _ = row_px;
        }
        p.render(buf);
        let names: Vec<String> = data.series.iter().map(|s| s.name.clone()).collect();
        x_labels(buf, area, &plot, &names, Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}
