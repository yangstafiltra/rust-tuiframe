use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override_ymin, shade, ticks, x_labels, y_labels};

pub struct Waterfall;

impl Chart for Waterfall {
    fn name(&self) -> &'static str {
        "waterfall_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.is_empty() {
            return None;
        }
        let vals = &data.series[0].values;
        let n = vals.len();
        let mut cums = vec![0.0; n + 1];
        for i in 0..n {
            cums[i + 1] = cums[i] + vals[i];
        }
        let y_min = cums.iter().cloned().fold(0.0_f64, f64::min);
        let y_max = prims::nice_max(cums.iter().cloned().fold(0.0_f64, f64::max));
        let y_min = if y_min < 0.0 { y_min * 1.1 } else { 0.0 };
        let y_max = y_max * 1.12;
        Some((0.0, n as f64, y_min, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Quarterly P&L (k$)",
                &["Rev", "COGS", "Ops", "Tax", "Total"],
                &[("values", &[100.0, -40.0, -20.0, -15.0, 25.0])],
            ),
            ds(
                "Budget Flow",
                &["Income", "Rent", "Food", "Travel", "Balance"],
                &[("values", &[5000.0, -1500.0, -800.0, -500.0, 2200.0])],
            ),
            ds(
                "Cash Flow Statement (k$)",
                &["Op Inc", "Depr", "Tax", "Capex", "Net"],
                &[("values", &[800.0, 120.0, -180.0, -250.0, 490.0])],
            ),
            ds(
                "Startup Burn",
                &["Seed", "Payroll", "Rent", "Marketing", "Cash"],
                &[("values", &[1000.0, -350.0, -120.0, -180.0, 350.0])],
            ),
            ds(
                "Monthly Savings",
                &["Salary", "Tax", "Rent", "Groceries", "Saved"],
                &[("values", &[4000.0, -800.0, -1200.0, -500.0, 1500.0])],
            ),
            ds(
                "Project Budget (k$)",
                &["Fund", "Labor", "Materials", "Overhead", "Remaining"],
                &[("values", &[200.0, -70.0, -50.0, -30.0, 50.0])],
            ),
            ds(
                "Retail Sales (k$)",
                &["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Total"],
                &[("values", &[120.0, 90.0, 140.0, 110.0, 150.0, 130.0, 740.0])],
            ),
            ds(
                "Energy Balance (MWh)",
                &["Gen", "Export", "Load", "Loss", "Net"],
                &[("values", &[500.0, -150.0, -300.0, -30.0, 20.0])],
            ),
            ds(
                "Inventory Flow",
                &["Stock", "Used", "Waste", "Bought", "End"],
                &[("values", &[100.0, -40.0, -10.0, 60.0, 110.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(4);
        let title = if data.title.is_empty() { "Waterfall Chart" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let vals = &data.series[0].values;
        let n = vals.len();
        // running cumulative for "floating" bars
        let mut cums = vec![0.0; n + 1];
        for i in 0..n {
            cums[i + 1] = cums[i] + vals[i];
        }
        let all: Vec<f64> = cums.clone();
        let y_min = all.iter().cloned().fold(0.0_f64, f64::min);
        let y_max = prims::nice_max(all.iter().cloned().fold(0.0_f64, f64::max));
        let y_min = if y_min < 0.0 { y_min * 1.1 } else { 0.0 };
        let y_max = y_max * 1.12;
        let y_t = ticks(y_min, y_max, 5);
        let (y_min, y_max) = scale_override_ymin(data, y_min, y_max);

        let mut p = Plot::new(plot, 0.0, n as f64, y_min, y_max);
        apply_grid(data, &mut p);
        p.draw_grid(&[], &y_t, crate::prims::GRID_LINE);
        // zero baseline
        let (_, yz) = p.px(0.0, 0.0);
        p.canvas.hline_px(0, p.canvas.px as i32 - 1, yz, Color::White);

        for i in 0..n {
            let base = cums[i];
            let top = cums[i + 1];
            let up = vals[i] >= 0.0;
            // gradient through the series; keep the running total magenta as
            // the semantic "final" bar.
            let c = if i == n - 1 {
                Color::LightMagenta
            } else {
                prims::series_color(i, n)
            };
            let bw = ((plot.width as f64 / n as f64) * 0.55) as i32;
            let (cx, _) = p.px(i as f64 + 0.5, 0.0);
            let (_, yb) = p.px(0.0, base);
            let (_, yt) = p.px(0.0, top);
            let ylo = yb.min(yt);
            let yhi = yb.max(yt);
            let span = (yhi - ylo).max(1);
            // Per-bar vertical gradient: bright at the top, shaded toward the
            // bottom, so each block has interior shading instead of flat fill.
            for x in (cx - bw)..=(cx + bw) {
                for y in ylo..=yhi {
                    let frac = (y - ylo) as f64 / span as f64;
                    let cc = shade(c, 0.15 + frac * 0.45);
                    p.canvas.set_px(x, y, cc);
                }
            }
            // value label on top
            let (_, ylab) = p.px(0.0, top);
            let label = prims::fmt(vals[i]);
            let lx = cx - label.chars().count() as i32 / 2 + 1;
            let ly = plot.y + (ylab as i32 - 1).clamp(0, plot.height as i32 - 1) as u16;
            prims::abs_text(buf, plot.x + lx as u16, ly, &label, prims::DIM);
            let _ = up;
        }
        p.render(buf);
        x_labels(buf, area, &plot, &data.labels, Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::data::single;

    #[test]
    fn waterfall_natural_scale_matches_render_bounds() {
        let d = single("t", &["A", "B", "C"], &[100.0, -40.0, -20.0]);
        let s = Waterfall.natural_scale(&d).unwrap();
        // cumulative: [0, 100, 60, 40]; min 0, max 100 -> nice_max 100, *1.12
        assert!((s.2 - 0.0).abs() < 1e-9);
        assert!((s.3 - 112.0).abs() < 1e-9);
        assert!((s.1 - 3.0).abs() < 1e-9);
    }

    #[test]
    fn waterfall_natural_scale_handles_negative_dip() {
        let d = single("t", &["A", "B"], &[50.0, -80.0]);
        let s = Waterfall.natural_scale(&d).unwrap();
        // cumulative: [0, 50, -30]; min -30 -> *1.1 = -33
        assert!((s.2 + 33.0).abs() < 1e-9, "y_min = {}", s.2);
    }

    #[test]
    fn waterfall_bar_has_interior_gradient() {
        use ratatui::style::Color;
        let mut chart = Waterfall;
        let data = chart.presets()[0].clone();
        let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
        terminal.clear().ok();
        let area = ratatui::layout::Rect::new(0, 0, 96, 30);
        terminal.draw(|f| chart.render(f.buffer_mut(), area, &data)).ok();
        let buf = terminal.backend().buffer();
        // Find a vertical run of colored pixels inside the first bar and
        // confirm at least two distinct shades appear (interior gradient).
        let mut distinct: Vec<(u8, u8, u8)> = Vec::new();
        'outer: for y in 5..buf.area.height.saturating_sub(5) {
            for x in 10..60 {
                if let Some(cell) = buf.cell((x, y)) {
                    if let Color::Rgb(r, g, b) = cell.fg {
                        if !distinct.contains(&(r, g, b)) {
                            distinct.push((r, g, b));
                        }
                        if distinct.len() >= 2 {
                            break 'outer;
                        }
                    }
                }
            }
        }
        assert!(
            distinct.len() >= 2,
            "expected >=2 distinct shades inside a bar, got {}",
            distinct.len()
        );
    }
}
