use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::cartesian::{Plot, apply_grid, framed, scale_override, smooth_density, ticks, x_labels, y_labels};

pub struct StackedArea;

impl Chart for StackedArea {
    fn name(&self) -> &'static str {
        "stacked_area_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.is_empty() {
            return None;
        }
        let n = data.len();
        let mut stack = vec![0.0; n];
        for s in &data.series {
            for (i, v) in s.values.iter().enumerate() {
                if i < n {
                    stack[i] += v;
                }
            }
        }
        let y_max = prims::nice_max(stack.iter().cloned().fold(0.0_f64, f64::max));
        Some((0.0, (n - 1).max(1) as f64, 0.0, y_max))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Traffic Sources",
                &["Jan", "Feb", "Mar", "Apr", "May", "Jun"],
                &[
                    ("Organic", &[30.0, 45.0, 50.0, 60.0, 70.0, 80.0]),
                    ("Social", &[20.0, 25.0, 35.0, 30.0, 45.0, 50.0]),
                    ("Direct", &[15.0, 20.0, 25.0, 22.0, 30.0, 40.0]),
                ],
            ),
            ds(
                "Monthly Budget",
                &["Rent", "Food", "Travel", "Fun", "Savings"],
                &[
                    ("Actual", &[1200.0, 600.0, 300.0, 250.0, 400.0]),
                    ("Planned", &[1100.0, 700.0, 250.0, 200.0, 500.0]),
                ],
            ),
            ds(
                "Energy Mix (MWh)",
                &["Mon", "Tue", "Wed", "Thu", "Fri"],
                &[
                    ("Wind", &[40.0, 55.0, 30.0, 70.0, 60.0]),
                    ("Solar", &[20.0, 35.0, 60.0, 40.0, 50.0]),
                    ("Gas", &[30.0, 25.0, 35.0, 20.0, 30.0]),
                ],
            ),
            ds(
                "Revenue Streams (k$)",
                &["Q1", "Q2", "Q3", "Q4"],
                &[
                    ("Products", &[120.0, 150.0, 180.0, 210.0]),
                    ("Services", &[80.0, 90.0, 110.0, 140.0]),
                    ("Licensing", &[40.0, 45.0, 55.0, 60.0]),
                    ("Ads", &[20.0, 25.0, 30.0, 35.0]),
                ],
            ),
            ds(
                "Pollution Sources (t)",
                &["Jan", "Feb", "Mar", "Apr"],
                &[
                    ("Cars", &[50.0, 55.0, 45.0, 40.0]),
                    ("Industry", &[30.0, 35.0, 38.0, 32.0]),
                    ("Power", &[20.0, 22.0, 25.0, 18.0]),
                ],
            ),
            ds(
                "Server Log Volume",
                &["H1", "H2", "H3", "H4", "H5", "H6"],
                &[
                    ("Error", &[5.0, 3.0, 8.0, 2.0, 7.0, 4.0]),
                    ("Warn", &[20.0, 15.0, 25.0, 18.0, 30.0, 22.0]),
                    ("Info", &[120.0, 130.0, 110.0, 140.0, 125.0, 135.0]),
                ],
            ),
            ds(
                "Diet Calories",
                &["Breakfast", "Lunch", "Snack", "Dinner"],
                &[
                    ("Carbs", &[150.0, 250.0, 100.0, 300.0]),
                    ("Protein", &[100.0, 200.0, 60.0, 250.0]),
                    ("Fat", &[80.0, 150.0, 40.0, 180.0]),
                ],
            ),
            ds(
                "Project Hours",
                &["W1", "W2", "W3", "W4", "W5"],
                &[
                    ("Coding", &[30.0, 35.0, 40.0, 25.0, 20.0]),
                    ("Meetings", &[10.0, 12.0, 8.0, 9.0, 7.0]),
                    ("Testing", &[5.0, 10.0, 15.0, 18.0, 10.0]),
                    ("Docs", &[3.0, 4.0, 5.0, 6.0, 8.0]),
                ],
            ),
            ds(
                "Water Usage (kL)",
                &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                &[
                    ("Bath", &[4.0, 4.0, 3.0, 5.0, 4.0, 6.0, 6.0]),
                    ("Kitchen", &[3.0, 3.0, 4.0, 3.0, 3.0, 5.0, 5.0]),
                    ("Garden", &[2.0, 1.0, 2.0, 1.0, 2.0, 6.0, 5.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(0);
        let title = if data.title.is_empty() { "Stacked Area" } else { &data.title };
        let plot = framed(buf, area, title, color);
        if data.series.is_empty() || data.series[0].values.is_empty() {
            prims::text(buf, plot, 1, "no data", prims::DIM);
            return;
        }
        let n = data.series[0].values.len();
        // cumulative stack
        let mut stack = vec![0.0; n];
        for s in &data.series {
            for (i, v) in s.values.iter().enumerate() {
                if i < n {
                    stack[i] += v;
                }
            }
        }
        let y_max = prims::nice_max(stack.iter().cloned().fold(0.0_f64, f64::max));
        let x_max = (n - 1).max(1) as f64;
        let x_t = ticks(0.0, x_max, 6);
        let y_t = ticks(0.0, y_max, 5);
        let (x_max, y_max) = scale_override(data, x_max, y_max);

        // draw series bottom-up
        let mut cum = vec![0.0; n];
        for (si, s) in data.series.iter().enumerate() {
            let color = prims::series_color(si, data.series.len());
            let pts: Vec<(f64, f64)> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64, cum[i] + v))
                .collect();
            let baseline: Vec<(f64, f64)> =
                cum.iter().enumerate().map(|(i, &c)| (i as f64, c)).collect();
            let mut p = Plot::new(plot, 0.0, x_max, 0.0, y_max);
            apply_grid(data, &mut p);
            p.draw_grid(&x_t, &y_t, crate::prims::GRID_LINE);
            let sm = crate::charts::cartesian::smooth(&pts, smooth_density(plot.width));
            let smb = crate::charts::cartesian::smooth(&baseline, smooth_density(plot.width));
            // fill between upper curve and lower curve
            let mut poly: Vec<(f64, f64)> = smb.clone();
            poly.extend(sm.iter().rev());
            let mapped: Vec<(f64, f64)> = poly.iter().map(|&(x, y)| p.px(x, y)).map(|(a, b)| (a as f64, b as f64)).collect();
            p.canvas.fill_poly(&mapped, color);
            p.polyline(&sm, Color::White);
            p.render(buf);
            for (i, v) in s.values.iter().enumerate() {
                cum[i] += v;
            }
        }
        x_labels(buf, area, &plot, &data.labels, Color::DarkGray);
        let grid = data.grid_scale.unwrap_or((0.0, x_max, 0.0, y_max));
        y_labels(buf, &plot, &y_t, grid, Color::DarkGray);
        // legend
        let names: Vec<String> = data.series.iter().map(|s| s.name.clone()).collect();
        let colors: Vec<Color> = (0..data.series.len()).map(prims::palette).collect();
        prims::legend(buf, Rect::new(plot.x, plot.y + plot.height, plot.width, 1), &names, &colors);
    }
}
