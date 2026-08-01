use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{Axis, ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct GanttChart;

impl Chart for GanttChart {
    fn name(&self) -> &'static str {
        "gantt_chart"
    }

    fn natural_scale(&self, data: &ChartData) -> Option<Axis> {
        if data.series.len() < 2 || data.series[0].values.is_empty() {
            return None;
        }
        let end_max = data.series[0]
            .values
            .iter()
            .zip(data.series[1].values.iter())
            .map(|(s, d)| s + d)
            .fold(0.0_f64, f64::max);
        Some((0.0, prims::nice_max(end_max), 0.0, 1.0))
    }

    fn presets(&self) -> Vec<ChartData> {
        // series[0] = start, series[1] = duration
        vec![
            ds(
                "Project Plan",
                &["Design", "Dev", "Tests", "Docs", "Launch"],
                &[
                    ("start", &[0.0, 3.0, 8.0, 11.0, 14.0]),
                    ("duration", &[3.0, 5.0, 3.0, 3.0, 2.0]),
                ],
            ),
            ds(
                "Sprint Tasks",
                &["Backend", "Frontend", "QA", "Design", "Ship"],
                &[
                    ("start", &[1.0, 2.0, 6.0, 4.0, 9.0]),
                    ("duration", &[4.0, 4.0, 3.0, 3.0, 1.0]),
                ],
            ),
            ds(
                "Software Release Plan",
                &["Planning", "Dev", "Testing", "Beta", "GA"],
                &[
                    ("start", &[0.0, 4.0, 10.0, 14.0, 18.0]),
                    ("duration", &[4.0, 6.0, 4.0, 4.0, 2.0]),
                ],
            ),
            ds(
                "Event Preparation",
                &["Venue", "Invites", "Catering", "Rehearsal", "Event"],
                &[
                    ("start", &[0.0, 2.0, 6.0, 10.0, 14.0]),
                    ("duration", &[6.0, 4.0, 4.0, 2.0, 1.0]),
                ],
            ),
            ds(
                "Data Migration",
                &["Audit", "Schema", "Transfer", "Validate", "Cutover"],
                &[
                    ("start", &[0.0, 2.0, 6.0, 9.0, 12.0]),
                    ("duration", &[2.0, 4.0, 3.0, 3.0, 1.0]),
                ],
            ),
            ds(
                "Manufacturing Run",
                &["Design", "Procure", "Assemble", "QC", "Ship"],
                &[
                    ("start", &[0.0, 3.0, 8.0, 11.0, 13.0]),
                    ("duration", &[3.0, 5.0, 3.0, 2.0, 1.0]),
                ],
            ),
            ds(
                "Marketing Campaign",
                &["Research", "Creative", "Launch", "Optimize", "Wrap"],
                &[
                    ("start", &[0.0, 2.0, 7.0, 10.0, 14.0]),
                    ("duration", &[2.0, 5.0, 3.0, 4.0, 1.0]),
                ],
            ),
            ds(
                "Construction Timeline",
                &["Permits", "Foundation", "Frame", "Finish", "Inspect"],
                &[
                    ("start", &[0.0, 4.0, 9.0, 16.0, 22.0]),
                    ("duration", &[4.0, 5.0, 7.0, 6.0, 2.0]),
                ],
            ),
            ds(
                "Academic Semester",
                &["Coursework", "Project", "Midterms", "Finals", "Done"],
                &[
                    ("start", &[0.0, 4.0, 8.0, 12.0, 15.0]),
                    ("duration", &[4.0, 5.0, 2.0, 2.0, 1.0]),
                ],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(9);
        let title = if data.title.is_empty() { "Gantt Chart" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.series.len() < 2 || data.series[0].values.is_empty() {
            prims::text(buf, inner, 1, "needs start + duration series", prims::DIM);
            return;
        }
        let starts = &data.series[0].values;
        let durs = &data.series[1].values;
        let n = starts.len();
        let end_max = starts.iter().zip(durs.iter()).map(|(s, d)| s + d).fold(0.0_f64, f64::max);
        let scale_max = prims::nice_max(end_max);
        let scale_max = data
            .scale
            .map(|s| s.1)
            .unwrap_or(scale_max);
        let label_w = 14.min(inner.width / 4);
        let plot_x = inner.x + label_w;
        let plot_w = inner.width - label_w;
        let row_h = (inner.height as u16 / n as u16).max(1);

        for i in 0..n {
            let y = inner.y + i as u16 * row_h;
            // task label
            let label = data.labels.get(i).cloned().unwrap_or_else(|| format!("Task {}", i + 1));
            let label = if label.chars().count() as u16 > label_w {
                label.chars().take(label_w as usize).collect::<String>()
            } else {
                label
            };
            prims::abs_text(buf, inner.x, y, &label, prims::WHITE);
            // timeline row
            let s0 = ((starts[i] / scale_max) * plot_w as f64) as u16;
            let s1 = (((starts[i] + durs[i]) / scale_max) * plot_w as f64) as u16;
            let s1 = s1.max(s0 + 1);
            let c = prims::series_color(i, n);
            for x in s0..s1.min(plot_w) {
                buf[(plot_x + x, y)].set_symbol("█").set_fg(c);
            }
            // progress shading on the right half
            if s1 > s0 + 1 {
                let mid = s0 + (s1 - s0) / 2;
                for x in mid..s1.min(plot_w) {
                    buf[(plot_x + x, y)].set_symbol("█").set_fg(crate::charts::cartesian::shade(c, 0.6));
                }
            }
            // vertical gridlines every ~quarter
        }
        // timeline header
        prims::abs_text(buf, plot_x, inner.y - 1, "0 days", prims::DIM);
        let t2 = format!("{} days", (scale_max / 2.0).round() as u32);
        prims::abs_text(buf, plot_x + plot_w / 2, inner.y - 1, &t2, prims::DIM);
        let t3 = format!("{} days", scale_max.round() as u32);
        prims::abs_text(buf, plot_x + plot_w - t3.chars().count() as u16, inner.y - 1, &t3, prims::DIM);
        // vertical gridlines
        for frac in [0.25, 0.5, 0.75] {
            let gx = plot_x + (plot_w as f64 * frac) as u16;
            for y in inner.y..(inner.y + row_h * n as u16).min(inner.y + inner.height) {
                if gx < plot_x + plot_w {
                    buf[(gx, y)].set_symbol("│").set_fg(Color::DarkGray);
                }
            }
        }
    }
}
