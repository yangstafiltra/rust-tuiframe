use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, ds};
use crate::engine::Chart;
use crate::prims;
use crate::charts::total;

pub struct DonutChart;

impl Chart for DonutChart {
    fn name(&self) -> &'static str {
        "donut_chart"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Budget Breakdown",
                &["Housing", "Food", "Transport", "Savings", "Fun"],
                &[("Budget", &[35.0, 20.0, 15.0, 20.0, 10.0])],
            ),
            ds(
                "Market Share",
                &["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"],
                &[("Share", &[28.0, 22.0, 18.0, 14.0, 10.0, 8.0])],
            ),
            ds(
                "Screen Time",
                &["Work", "Social", "Video", "Games", "Other"],
                &[("Hours", &[8.0, 3.0, 2.0, 1.5, 1.5])],
            ),
            ds(
                "Voting Results",
                &["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace"],
                &[("Votes", &[320.0, 280.0, 210.0, 180.0, 120.0, 90.0, 60.0])],
            ),
            ds(
                "CPU Time Distribution",
                &["User", "System", "Idle", "IO Wait", "Steal"],
                &[("Time", &[45.0, 20.0, 25.0, 7.0, 3.0])],
            ),
            ds(
                "Browser Market Share",
                &["Chrome", "Safari", "Edge", "Firefox", "Other"],
                &[("Share", &[62.0, 19.0, 8.0, 5.0, 6.0])],
            ),
            ds(
                "Monthly Expenses",
                &["Housing", "Food", "Utilities", "Transport", "Health", "Fun"],
                &[("Cost", &[1200.0, 500.0, 200.0, 300.0, 150.0, 250.0])],
            ),
            ds(
                "Traffic Sources",
                &["Direct", "Organic", "Social", "Referral", "Email"],
                &[("Visits", &[180.0, 250.0, 140.0, 90.0, 60.0])],
            ),
            ds(
                "Cloud Cost Split",
                &["Compute", "Storage", "Network", "DB", "Support"],
                &[("Cost", &[420.0, 180.0, 90.0, 240.0, 70.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(4);
        let title = if data.title.is_empty() { "Donut Chart" } else { &data.title };
        prims::frame(buf, area, title, color);

        if data.is_empty() {
            prims::text(buf, area, area.height / 2, "no data", prims::DIM);
            return;
        }
        let values = &data.series[0].values;
        let labels = if data.labels.is_empty() {
            (1..=values.len()).map(|i| format!("Item {i}")).collect()
        } else {
            data.labels.clone()
        };
        let tot = total(data).max(1e-9);

        // Layout: chart on the left, legend on the right.
        let legend_w = 26.min(area.width / 3);
        let chart_w = area.width.saturating_sub(legend_w).saturating_sub(2);
        let chart_h = area.height.saturating_sub(2);
        let cx = 2 + chart_w / 2;
        let cy = 1 + chart_h / 2;
        let rad = (chart_w.min(chart_h) as f64 * 0.38) as u16;
        let inner_rad = (rad as f64 * 0.58) as u16;

        // Draw donut segments in pixel space.
        let pw = chart_w as usize;
        let ph = chart_h as usize * 2;
        let cxp = cx as f64;
        let cyp = cy as f64 * 2.0;
        let radp = rad as f64 * 2.0;
        let inner_p = inner_rad as f64 * 2.0;
        let mut start = -std::f64::consts::FRAC_PI_2;
        let mut px = vec![None::<Color>; pw * ph];
        for (i, &v) in values.iter().enumerate() {
            let frac = v / tot;
            let sweep = frac * std::f64::consts::TAU;
            let c = prims::series_color(i, data.labels.len().max(1));
            for y in 0..ph {
                for x in 0..pw {
                    let dx = x as f64 - cxp;
                    let dy = y as f64 - cyp;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < inner_p || r > radp {
                        continue;
                    }
                    let ang = dy.atan2(dx);
                    let mut a = ang;
                    while a < start {
                        a += std::f64::consts::TAU;
                    }
                    let rel = a - start;
                    if rel <= sweep + 1e-6 {
                        let idx = y * pw + x;
                        // shading for a subtle 3D feel
                        let shade = ((dx / radp) * 0.25 + 0.75).clamp(0.0, 1.0);
                        px[idx] = Some(crate::charts::cartesian::shade(c, shade));
                    }
                }
            }
            start += sweep;
        }
        // render pixel grid
        for y in 0..ph {
            for x in 0..pw {
                if let Some(c) = px[y * pw + x] {
                    let cell = &mut buf[(area.x + x as u16, area.y + 1 + (y / 2) as u16)];
                    if y % 2 == 0 {
                        cell.set_symbol("▀").set_fg(c);
                    } else if let Some(_top) = px.get((y - 1) * pw + x) {
                        cell.set_symbol("▀").set_fg(c);
                    } else {
                        cell.set_symbol("▄").set_fg(c);
                    }
                }
            }
        }

        // Legend
        let lx = area.x + 2 + chart_w + 1;
        let mut ly = area.y + 2;
        for (i, name) in labels.iter().enumerate() {
            if ly + 1 >= area.y + area.height {
                break;
            }
            let val = values[i];
            let pct = val / tot * 100.0;
            let c = prims::series_color(i, data.labels.len().max(1));
            buf[(lx, ly)].set_symbol("■").set_fg(c);
            let text = format!(" {name}  {pct:.0}%");
            for (j, ch) in text.chars().enumerate() {
                let col = lx + 1 + j as u16;
                if col < area.x + area.width {
                    buf[(col, ly)].set_symbol(&ch.to_string()).set_style(prims::WHITE);
                }
            }
            ly += 1;
        }
        // center label
        let center = format!("{tot:.0}");
        let x = cx.saturating_sub(center.chars().count() as u16 / 2);
        for (i, ch) in center.chars().enumerate() {
            buf[(area.x + x + i as u16, cy)].set_symbol(&ch.to_string()).set_style(prims::BRIGHT);
        }
    }
}
