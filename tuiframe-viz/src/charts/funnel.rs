use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::data::{ChartData, ds};
use crate::engine::Chart;
use crate::prims;

pub struct FunnelChart;

impl Chart for FunnelChart {
    fn name(&self) -> &'static str {
        "funnel_chart"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ds(
                "Sales Funnel",
                &["Leads", "Qualified", "Demo", "Proposal", "Closed"],
                &[("Count", &[1000.0, 600.0, 320.0, 180.0, 90.0])],
            ),
            ds(
                "Conversion Pipeline",
                &["Visit", "Signup", "Activate", "Pay", "Renew"],
                &[("Users", &[5000.0, 2500.0, 1200.0, 800.0, 600.0])],
            ),
            ds(
                "Support Ticket Flow",
                &["Opened", "Triaged", "Assigned", "Resolved", "Closed"],
                &[("Tickets", &[800.0, 640.0, 510.0, 430.0, 410.0])],
            ),
            ds(
                "Recruiting Funnel",
                &["Applied", "Screened", "Interview", "Offer", "Hired"],
                &[("Candidates", &[1200.0, 480.0, 190.0, 60.0, 42.0])],
            ),
            ds(
                "Marketing Funnel",
                &["Impressions", "Clicks", "Leads", "MQL", "SQL"],
                &[("Count", &[100000.0, 8200.0, 1200.0, 480.0, 210.0])],
            ),
            ds(
                "Course Completion",
                &["Enrolled", "Started", "Lesson 5", "Final", "Certified"],
                &[("Students", &[900.0, 720.0, 400.0, 250.0, 180.0])],
            ),
            ds(
                "E-commerce Flow",
                &["Visit", "View Product", "Add Cart", "Checkout", "Purchased"],
                &[("Users", &[15000.0, 6200.0, 2400.0, 1100.0, 860.0])],
            ),
            ds(
                "App Onboarding",
                &["Install", "Open", "Register", "First Action", "Retained"],
                &[("Users", &[3000.0, 2600.0, 1900.0, 1500.0, 980.0])],
            ),
            ds(
                "Grant Application",
                &["Applied", "Eligible", "Shortlisted", "Interview", "Funded"],
                &[("Projects", &[640.0, 510.0, 260.0, 120.0, 70.0])],
            ),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(8);
        let title = if data.title.is_empty() { "Funnel Chart" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(4), area.height.saturating_sub(2));
        if data.is_empty() {
            prims::text(buf, inner, 1, "no data", prims::DIM);
            return;
        }
        let values = &data.series[0].values;
        let labels = if data.labels.is_empty() {
            (1..=values.len()).map(|i| format!("Stage {i}")).collect::<Vec<_>>()
        } else {
            data.labels.clone()
        };
        let max_v = values.iter().cloned().fold(0.0_f64, f64::max).max(1e-9);
        let n = values.len();
        let row_h = ((inner.height as usize / n.max(1)).max(3)) as u16;
        for (i, (&v, label)) in values.iter().zip(labels.iter()).enumerate() {
            let y0 = i as u16 * row_h;
            let frac = v / max_v;
            let half = (frac * inner.width as f64 / 2.0) as u16;
            let cx = inner.x + inner.width / 2;
            let c = prims::series_color(i, n);
            for r in 0..row_h.min(inner.height - y0) {
                let row_y = inner.y + y0 + r;
                let taper = if r < row_h / 2 { 0 } else { 1 };
                let w = half.saturating_sub(taper);
                let left = cx.saturating_sub(w);
                let right = (cx + w).min(inner.x + inner.width);
                let shade = 0.55 + 0.45 * (r as f64 / row_h.max(1) as f64);
                let cc = crate::charts::cartesian::shade(c, shade);
                for x in left..right {
                    if x < inner.x + inner.width {
                        buf[(x, row_y)].set_symbol("█").set_fg(cc);
                    }
                }
            }
            // label + value on the left and right
            let pct = v / max_v * 100.0;
            let label_txt = format!("{label} {pct:.0}%");
            let yc = y0 + row_h.min(inner.height - y0) / 2;
            if yc + 1 < inner.height {
                prims::abs_text(buf, inner.x, inner.y + yc, &label_txt, prims::fg(color));
            }
        }
    }
}
