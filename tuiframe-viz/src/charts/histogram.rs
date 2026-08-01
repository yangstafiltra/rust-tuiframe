use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override, ticks, x_labels, y_labels};

pub struct Histogram;

impl Chart for Histogram {
    fn name(&self) -> &'static str {
        "histogram"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.is_empty() {
            return None;
        }
        let y_max = prims::nice_max(data.series[0].values.iter().cloned().fold(0.0_f64, f64::max));
        Some((0.0, 1.0, 0.0, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Response Times (ms)",
                &["<50", "50-100", "100-200", "200-400", "400-800", "800+"],
                &[("Requests", &[120.0, 340.0, 510.0, 260.0, 90.0, 25.0])],
            ),
            ds(
                "Age Distribution",
                &["0-18", "19-30", "31-45", "46-60", "60+"],
                &[("Count", &[80.0, 420.0, 380.0, 210.0, 140.0])],
            ),
            ds(
                "Salary Bands (k$)",
                &["<50", "50-70", "70-100", "100-150", "150+"],
                &[("Employees", &[45.0, 130.0, 260.0, 150.0, 40.0])],
            ),
            ds(
                "Package Sizes (cm)",
                &["0-10", "10-20", "20-30", "30-40", "40-50", "50-60", "60-70", "70-80"],
                &[("Count", &[80.0, 240.0, 380.0, 420.0, 350.0, 210.0, 120.0, 50.0])],
            ),
            ds(
                "Battery Life (h)",
                &["0-4", "4-8", "8-12", "12-16", "16-20", "20-24"],
                &[("Devices", &[30.0, 160.0, 420.0, 350.0, 180.0, 60.0])],
            ),
            ds(
                "File Sizes (MB)",
                &["<1", "1-5", "5-10", "10-50", "50-100", "100-500", "500+"],
                &[("Files", &[500.0, 800.0, 640.0, 380.0, 150.0, 60.0, 12.0])],
            ),
            ds(
                "Step Counts / Day",
                &["<2k", "2-4k", "4-6k", "6-8k", "8-10k", "10-12k", "12k+"],
                &[("People", &[20.0, 90.0, 220.0, 300.0, 240.0, 100.0, 30.0])],
            ),
            ds(
                "Coffee Sales / Hour",
                &["6-8", "8-10", "10-12", "12-14", "14-16", "16-18", "18-20", "20-22"],
                &[("Cups", &[60.0, 140.0, 180.0, 160.0, 130.0, 110.0, 90.0, 40.0])],
            ),
            ds(
                "Word Counts in Emails",
                &["<50", "50-100", "100-200", "200-400", "400-800", "800+"],
                &[("Emails", &[150.0, 320.0, 410.0, 260.0, 90.0, 20.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(1);
        let title = if data.title.is_empty() { "Histogram" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let vals = &data.series[0].values;
        let y_max = prims::nice_max(vals.iter().cloned().fold(0.0_f64, f64::max));
        let n = vals.len();
        let y_t = ticks(0.0, y_max, 5);

        // bars with rounded-ish tops using half-blocks
        let bw = (plot.width as f64 / n.max(1) as f64).max(1.0);
        let (_, y_max) = scale_override(data, 1.0, y_max);
        let mut p = Plot::new(plot, 0.0, 1.0, 0.0, y_max);
        apply_grid(data, &mut p);
        p.draw_grid(&[], &y_t, crate::prims::GRID_LINE);
        for (i, &v) in vals.iter().enumerate() {
            let x0 = ((i as f64) / n as f64 * plot.width as f64) as i32;
            let x1 = (((i + 1) as f64) / n as f64 * plot.width as f64) as i32;
            let x1 = x1.max(x0 + 1);
            let hpx = (v / y_max * (p.canvas.py - 2) as f64) as i32;
            let c = prims::series_color(i, n);
            for x in x0..x1 {
                for k in 0..hpx {
                    p.canvas.set_px(x, (p.canvas.py as i32 - 1) - k, c);
                }
            }
        }
        p.render(buf);
        // vertical guides between bars
        let _ = bw;
        x_labels(buf, area, &plot, &data.labels, Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}
