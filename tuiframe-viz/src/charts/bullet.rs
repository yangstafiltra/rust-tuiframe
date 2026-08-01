use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct BulletChart;

impl Chart for BulletChart {
    fn name(&self) -> &'static str {
        "bullet_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.series.len() < 2 || data.series[0].values.is_empty() {
            return None;
        }
        let cur = data.series[0].values[0];
        let target = data.series[1].values[0];
        let scale_max = prims::nice_max(target.max(cur) * 1.15);
        Some((0.0, scale_max, 0.0, 1.0))
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Monthly Revenue Goal",
                &["Revenue"],
                &[("current", &[82.0]), ("target", &[100.0])],
            ),
            ds(
                "Team Velocity",
                &["Velocity"],
                &[("current", &[34.0]), ("target", &[50.0])],
            ),
            ds(
                "Disk Usage",
                &["Storage"],
                &[("current", &[71.0]), ("target", &[90.0])],
            ),
            ds(
                "CPU Utilization",
                &["CPU"],
                &[("current", &[58.0]), ("target", &[80.0])],
            ),
            ds(
                "Battery Level",
                &["Battery"],
                &[("current", &[63.0]), ("target", &[100.0])],
            ),
            ds(
                "Production Output",
                &["Units"],
                &[("current", &[4100.0]), ("target", &[5000.0])],
            ),
            ds(
                "Network Bandwidth",
                &["Bandwidth"],
                &[("current", &[72.0]), ("target", &[100.0])],
            ),
            ds(
                "Customer Satisfaction",
                &["CSAT"],
                &[("current", &[4.2]), ("target", &[5.0])],
            ),
            ds(
                "Attendance Rate",
                &["Attendance"],
                &[("current", &[88.0]), ("target", &[100.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(6);
        let title = if data.title.is_empty() { "Bullet Chart" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.series.len() < 2 || data.series[0].values.is_empty() {
            prims::text(buf, inner, 1, "needs current + target series", prims::DIM);
            return;
        }
        let cur = data.series[0].values[0];
        let target = data.series[1].values[0];
        let scale_max = prims::nice_max(target.max(cur) * 1.15);
        let scale_max = data
            .scale
            .map(|s| s.1)
            .unwrap_or(scale_max);

        // Label + value
        let label = if data.labels.is_empty() { "Value" } else { &data.labels[0] };
        let mut ly = inner.y;
        prims::abs_text(buf, inner.x, ly, label, prims::BRIGHT);
        let val_txt = format!("{cur:.0} / {target:.0}");
        prims::abs_text(buf, inner.x + inner.width - val_txt.chars().count() as u16, ly, &val_txt, prims::fg(color));
        ly += 1;

        // Background zones (qualitative ranges)
        let bar_row = ly;
        let bar_w = inner.width;
        let zone_colors = [
            crate::charts::cartesian::shade(color, 0.22),
            crate::charts::cartesian::shade(color, 0.5),
            crate::charts::cartesian::shade(color, 0.78),
        ];
        for (z, c) in zone_colors.iter().enumerate() {
            let fracs = [0.5, 0.75, 1.0];
            let start = if z == 0 { 0.0 } else { fracs[z - 1] };
            let end = fracs[z];
            for x in (start * bar_w as f64) as u16..(end * bar_w as f64) as u16 {
                if x < bar_w {
                    buf[(inner.x + x, bar_row)].set_symbol("░").set_fg(*c);
                }
            }
        }
        // Value bar
        let cur_x = (cur / scale_max * bar_w as f64) as u16;
        for x in 0..cur_x.min(bar_w) {
            buf[(inner.x + x, bar_row)].set_symbol("█").set_fg(color);
        }
        // Target marker
        let tgt_x = (target / scale_max * bar_w as f64) as u16;
        if tgt_x < bar_w {
            buf[(inner.x + tgt_x, bar_row)].set_symbol("▏").set_fg(Color::White);
        }
        ly += 2;

        // Scale labels
        let scale_txt = format!("0                          {:.0}                        {:.0}", scale_max / 2.0, scale_max);
        prims::abs_text(buf, inner.x, ly, &scale_txt, prims::DIM);
        let _ = ly;
    }
}
