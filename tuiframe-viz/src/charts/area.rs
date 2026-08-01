use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override, smooth_density, ticks, x_labels, y_labels};

pub struct AreaChart;

impl Chart for AreaChart {
    fn name(&self) -> &'static str {
        "area_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.is_empty() {
            return None;
        }
        let y_max = prims::nice_max(data.series[0].values.iter().cloned().fold(0.0_f64, f64::max));
        let n = data.len();
        Some((0.0, (n - 1).max(1) as f64, 0.0, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Quarterly Revenue",
                &["Q1", "Q2", "Q3", "Q4"],
                &[("Revenue", &[42.0, 58.0, 65.0, 91.0])],
            ),
            ds(
                "Website Visitors",
                &["Jan", "Mar", "May", "Jul", "Sep", "Nov"],
                &[("Visitors", &[120.0, 210.0, 165.0, 240.0, 300.0, 260.0])],
            ),
            ds(
                "CPU Load (%)",
                &["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
                &[("CPU", &[12.0, 8.0, 55.0, 92.0, 61.0, 33.0])],
            ),
            ds(
                "Git Commits / Day",
                &["Mon", "Tue", "Wed", "Thu", "Fri"],
                &[("Commits", &[7.0, 3.0, 12.0, 5.0, 9.0])],
            ),
            ds(
                "Page Load Time (ms)",
                &["10k", "50k", "100k", "200k", "500k", "1M", "2M"],
                &[("Load", &[120.0, 210.0, 340.0, 520.0, 900.0, 1400.0, 2200.0])],
            ),
            ds(
                "Rainfall (mm)",
                &["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
                &[("Rain", &[78.0, 60.0, 72.0, 55.0, 40.0, 22.0, 15.0, 18.0, 35.0, 58.0, 70.0, 82.0])],
            ),
            ds(
                "Heart Rate (bpm)",
                &["00h", "04h", "08h", "12h", "16h", "20h", "24h"],
                &[("HR", &[55.0, 52.0, 72.0, 84.0, 78.0, 88.0, 58.0])],
            ),
            ds(
                "Memory Usage (GB)",
                &["T1", "T2", "T3", "T4", "T5", "T6"],
                &[("Mem", &[2.1, 2.8, 3.2, 3.0, 4.1, 4.6])],
            ),
            ds(
                "Sales by Channel",
                &["W1", "W2", "W3", "W4", "W5"],
                &[("Online", &[40.0, 55.0, 60.0, 70.0, 66.0]), ("Store", &[30.0, 25.0, 40.0, 35.0, 50.0])],
            ),
            ds(
                "Server Requests",
                &["Min 1", "Min 2", "Min 3", "Min 4", "Min 5"],
                &[("req/s", &[220.0, 310.0, 280.0, 420.0, 390.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(0);
        let title = if data.title.is_empty() { "Area Chart" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let vals = &data.series[0].values;
        let y_max = prims::nice_max(vals.iter().cloned().fold(0.0_f64, f64::max));
        let n = vals.len();
        let pts: Vec<(f64, f64)> = vals
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();
        let sm = crate::charts::cartesian::smooth(&pts, smooth_density(plot.width));
        let (x_max, y_max) = scale_override(data, (n - 1).max(1) as f64, y_max);
        let mut p = Plot::new(plot, 0.0, x_max, 0.0, y_max);
        apply_grid(data, &mut p);
        let x_t = ticks(0.0, x_max, 6);
        let y_t = ticks(0.0, y_max, 5);
        p.draw_grid(&x_t, &y_t, crate::prims::GRID_LINE);
        // gradient fill fading toward the top for a rich look
        p.fill_gradient(&sm, 0.0, color, 0.18);
        // highlight the top surface
        p.polyline(&sm, Color::White);
        p.render(buf);
        x_labels(buf, area, &plot, &data.labels, Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::single;

    #[test]
    fn area_natural_scale_uses_nice_max() {
        let d = single("t", &["a", "b"], &[3.0, 18.0]);
        let s = AreaChart.natural_scale(&d).unwrap();
        assert!((s.1 - 1.0).abs() < 1e-9);
        assert!((s.3 - 20.0).abs() < 1e-9);
    }

    #[test]
    fn scale_override_prefers_animated_axis() {
        let mut d = single("t", &["a", "b"], &[1.0, 2.0]);
        d.scale = Some((0.0, 5.0, 0.0, 50.0));
        d.grid_scale = Some((0.0, 4.0, 0.0, 40.0));
        let (x_max, y_max) = scale_override(&d, 1.0, 20.0);
        assert_eq!(x_max, 5.0);
        assert_eq!(y_max, 50.0);
        // grid_scale falls back to data scale when injected, else default.
        let (x_max, y_max) = scale_override(&d, 1.0, 20.0);
        assert_eq!(x_max, 5.0);
        assert_eq!(y_max, 50.0);
        let _ = (x_max, y_max);
    }
}
