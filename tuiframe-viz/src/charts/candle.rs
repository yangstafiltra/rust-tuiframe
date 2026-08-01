use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override_ymin, ticks, x_labels, y_labels};

pub struct CandleChart;

impl Chart for CandleChart {
    fn name(&self) -> &'static str {
        "candle_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.series.len() < 4 || data.series[0].values.is_empty() {
            return None;
        }
        let y_max = prims::nice_max(data.series[1].values.iter().cloned().fold(0.0_f64, f64::max));
        let y_min = data.series[2].values.iter().cloned().fold(0.0_f64, f64::min);
        Some((0.0, data.series[0].values.len() as f64, y_min, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        // Each candle is a column: open, high, low, close.
        vec![
            ds(
                "BTC Price (k$)",
                &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                &[
                    ("open", &[42.0, 44.0, 43.5, 45.2, 44.8, 46.1, 47.0]),
                    ("high", &[44.5, 44.9, 45.8, 46.2, 46.0, 47.5, 48.2]),
                    ("low", &[41.5, 43.0, 42.8, 44.5, 43.9, 45.8, 46.5]),
                    ("close", &[44.0, 43.5, 45.2, 44.8, 46.1, 47.0, 46.8]),
                ],
            ),
            ds(
                "Stock XYZ",
                &["D1", "D2", "D3", "D4", "D5", "D6"],
                &[
                    ("open", &[100.0, 102.0, 101.0, 104.0, 103.0, 105.0]),
                    ("high", &[103.0, 104.5, 103.5, 106.0, 105.5, 107.0]),
                    ("low", &[99.0, 100.5, 100.0, 102.5, 102.0, 104.0]),
                    ("close", &[102.0, 101.0, 104.0, 103.0, 105.0, 104.5]),
                ],
            ),
            ds(
                "EUR/USD Hourly",
                &["H1", "H2", "H3", "H4", "H5", "H6", "H7"],
                &[
                    ("open", &[1.085, 1.087, 1.086, 1.089, 1.088, 1.091, 1.090]),
                    ("high", &[1.088, 1.089, 1.090, 1.091, 1.092, 1.093, 1.092]),
                    ("low", &[1.083, 1.085, 1.084, 1.087, 1.086, 1.089, 1.088]),
                    ("close", &[1.087, 1.086, 1.089, 1.088, 1.091, 1.090, 1.093]),
                ],
            ),
            ds(
                "Gold Futures",
                &["Day1", "Day2", "Day3", "Day4", "Day5"],
                &[
                    ("open", &[1850.0, 1870.0, 1860.0, 1880.0, 1890.0]),
                    ("high", &[1880.0, 1890.0, 1885.0, 1900.0, 1910.0]),
                    ("low", &[1830.0, 1850.0, 1840.0, 1865.0, 1875.0]),
                    ("close", &[1870.0, 1860.0, 1880.0, 1890.0, 1905.0]),
                ],
            ),
            ds(
                "Oil Price (WTI)",
                &["Mon", "Tue", "Wed", "Thu", "Fri"],
                &[
                    ("open", &[72.0, 73.0, 72.5, 74.0, 73.5]),
                    ("high", &[74.0, 74.5, 74.0, 75.5, 75.0]),
                    ("low", &[70.5, 71.5, 71.0, 72.5, 72.0]),
                    ("close", &[73.0, 72.5, 74.0, 73.5, 74.5]),
                ],
            ),
            ds(
                "ETH Price (k$)",
                &["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8"],
                &[
                    ("open", &[2.8, 2.9, 2.85, 3.0, 2.95, 3.1, 3.05, 3.2]),
                    ("high", &[2.95, 3.0, 2.98, 3.1, 3.08, 3.2, 3.18, 3.3]),
                    ("low", &[2.7, 2.8, 2.75, 2.9, 2.85, 3.0, 2.95, 3.1]),
                    ("close", &[2.9, 2.85, 3.0, 2.95, 3.1, 3.05, 3.2, 3.15]),
                ],
            ),
            ds(
                "Tesla (TSLA)",
                &["D1", "D2", "D3", "D4", "D5", "D6"],
                &[
                    ("open", &[240.0, 245.0, 242.0, 250.0, 248.0, 255.0]),
                    ("high", &[248.0, 250.0, 249.0, 255.0, 254.0, 260.0]),
                    ("low", &[235.0, 240.0, 238.0, 246.0, 244.0, 250.0]),
                    ("close", &[245.0, 242.0, 250.0, 248.0, 255.0, 252.0]),
                ],
            ),
            ds(
                "S&P 500",
                &["Mon", "Tue", "Wed", "Thu", "Fri"],
                &[
                    ("open", &[4800.0, 4830.0, 4820.0, 4860.0, 4850.0]),
                    ("high", &[4850.0, 4870.0, 4865.0, 4890.0, 4880.0]),
                    ("low", &[4760.0, 4790.0, 4780.0, 4820.0, 4810.0]),
                    ("close", &[4830.0, 4820.0, 4860.0, 4850.0, 4875.0]),
                ],
            ),
            ds(
                "Silver",
                &["D1", "D2", "D3", "D4", "D5"],
                &[
                    ("open", &[24.0, 24.5, 24.2, 24.8, 24.6]),
                    ("high", &[24.8, 25.0, 24.9, 25.3, 25.1]),
                    ("low", &[23.5, 23.9, 23.7, 24.2, 24.0]),
                    ("close", &[24.5, 24.2, 24.8, 24.6, 25.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(7);
        let title = if data.title.is_empty() { "Candle Chart" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.series.len() < 4 || data.series[0].values.is_empty() {
            prims::text(buf, plot, 1, "needs open/high/low/close series", prims::DIM);
            return;
        }
        let opens = &data.series[0].values;
        let highs = &data.series[1].values;
        let lows = &data.series[2].values;
        let closes = &data.series[3].values;
        let n = opens.len();
        let y_max = prims::nice_max(highs.iter().cloned().fold(0.0_f64, f64::max));
        let y_min = lows.iter().cloned().fold(0.0_f64, f64::min);
        let y_t = ticks(y_min, y_max, 5);
        let (y_min, y_max) = scale_override_ymin(data, y_min, y_max);

        let mut p = Plot::new(plot, 0.0, n as f64, y_min, y_max);
        apply_grid(data, &mut p);
        p.draw_grid(&[], &y_t, crate::prims::GRID_LINE);
        let bw = ((plot.width as f64 / n as f64) * 0.5) as i32;
        for i in 0..n {
            let up = closes[i] >= opens[i];
            let c = if up { Color::LightGreen } else { Color::LightRed };
            let (cx, _) = p.px(i as f64 + 0.5, 0.0);
            let (_, yhigh) = p.px(0.0, highs[i]);
            let (_, ylow) = p.px(0.0, lows[i]);
            let (_, yopen) = p.px(0.0, opens[i]);
            let (_, yclose) = p.px(0.0, closes[i]);
            // wick
            p.canvas.vline(cx, yhigh, ylow, c);
            // body
            let top = yopen.min(yclose);
            let bot = yopen.max(yclose);
            for x in (cx - bw)..=(cx + bw) {
                p.canvas.vline(x, top, bot, c);
            }
        }
        p.render(buf);
        x_labels(buf, area, &plot, &data.labels, Color::DarkGray);
        y_labels(buf, &plot, &y_t, p.grid, Color::DarkGray);
    }
}
