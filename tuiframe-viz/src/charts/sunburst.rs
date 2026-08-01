use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, TreeNode};
use crate::engine::Chart;
use crate::prims;

pub struct Sunburst;

impl Chart for Sunburst {
    fn name(&self) -> &'static str {
        "sunburst"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ChartData {
                title: "Disk by Category".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Apps", 30.0),
                    ("Media/Videos", 55.0),
                    ("Media/Photos", 25.0),
                    ("Media/Music", 15.0),
                    ("Documents/Work", 20.0),
                    ("Documents/Personal", 12.0),
                    ("System", 8.0),
                    ("Other", 10.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Browser Market".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Chromium/Chrome", 60.0),
                    ("Chromium/Edge", 8.0),
                    ("Safari", 20.0),
                    ("Firefox", 6.0),
                    ("Other", 6.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Website Traffic".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Direct", 35.0),
                    ("Search/Organic", 28.0),
                    ("Search/Paid", 12.0),
                    ("Social/Twitter", 6.0),
                    ("Social/Facebook", 5.0),
                    ("Referral/Blogs", 5.0),
                    ("Referral/News", 3.0),
                    ("Email", 2.0),
                    ("Other", 4.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Cloud Cost Split".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Compute/EC2", 180.0),
                    ("Compute/Lambda", 40.0),
                    ("Storage/S3", 90.0),
                    ("Storage/Glacier", 25.0),
                    ("DB/RDS", 70.0),
                    ("DB/Dynamo", 35.0),
                    ("Networking/CDN", 30.0),
                    ("Support", 20.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Food Groups".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Veg/Fresh", 20.0),
                    ("Veg/Stored", 10.0),
                    ("Protein/Meat", 25.0),
                    ("Protein/Plant", 15.0),
                    ("Grains/Rice", 10.0),
                    ("Grains/Wheat", 8.0),
                    ("Fruit", 12.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Energy Sources".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Fossil/Coal", 30.0),
                    ("Fossil/Gas", 20.0),
                    ("Renewable/Wind", 18.0),
                    ("Renewable/Solar", 14.0),
                    ("Renewable/Hydro", 8.0),
                    ("Nuclear", 5.0),
                    ("Other", 5.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "City Budget".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Infra/Roads", 30.0),
                    ("Infra/Buildings", 20.0),
                    ("Education/Schools", 15.0),
                    ("Health/Clinics", 10.0),
                    ("Parks", 8.0),
                    ("Police", 12.0),
                    ("Admin", 5.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Gaming Time".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("PC/Strategy", 25.0),
                    ("PC/Shooter", 15.0),
                    ("Console/RPG", 20.0),
                    ("Console/Sports", 10.0),
                    ("Mobile/Casual", 15.0),
                    ("Mobile/Puzzle", 8.0),
                    ("Other", 7.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Voting Split".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Party A/Core", 30.0),
                    ("Party A/Moderate", 15.0),
                    ("Party B/Core", 25.0),
                    ("Party B/Moderate", 10.0),
                    ("Independents", 12.0),
                    ("Undecided", 8.0),
                ]),
                ..Default::default()
            },
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(6);
        let title = if data.title.is_empty() { "Sunburst" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        let tree = if !data.tree.is_empty() {
            data.tree.clone()
        } else {
            vec![TreeNode {
                name: "root".to_string(),
                value: 0.0,
                children: data
                    .labels
                    .iter()
                    .zip(data.series.first().map(|s| s.values.clone()).unwrap_or_default())
                    .map(|(l, v)| TreeNode { name: l.clone(), value: v, children: Vec::new() })
                    .collect(),
            }]
        };
        let total: f64 = tree.iter().map(node_total).sum();
        if total <= 0.0 {
            prims::text(buf, inner, 1, "no data", prims::DIM);
            return;
        }
        let legend_w = 24.min(inner.width / 3);
        let chart_w = inner.width.saturating_sub(legend_w);
        let cx = inner.x + chart_w as u16 / 2;
        let cy = inner.y + inner.height / 2;
        let rad = (chart_w.min(inner.height) as f64 * 0.46) as u16;

        let max_depth = tree.iter().map(node_depth).max().unwrap_or(0);
        let stride = (0.94 - 0.06) / (max_depth + 1) as f64;

        // render rings pixel by pixel
        let pw = chart_w as usize;
        let ph = inner.height as usize * 2;
        let cxp = cx as f64;
        let cyp = cy as f64 * 2.0;
        let radp = rad as f64 * 2.0;
        let mut px = vec![None::<Color>; pw * ph];
        let depth0 = 0usize;
        for node in &tree {
            draw_node_px(&mut px, pw, ph, cxp, cyp, radp, stride, node, total, depth0, 0.0, std::f64::consts::TAU);
        }
        for y in 0..ph {
            for x in 0..pw {
                if let Some(c) = px[y * pw + x] {
                    let cell = &mut buf[(area.x + x as u16, area.y + 2 + (y / 2) as u16)];
                    if y % 2 == 0 {
                        cell.set_symbol("▀").set_fg(c);
                    } else if px.get((y - 1) * pw + x).is_some() {
                        cell.set_symbol("▀").set_fg(c);
                    } else {
                        cell.set_symbol("▄").set_fg(c);
                    }
                }
            }
        }
        // legend of top-level
        let mut ly = inner.y;
        for (i, node) in tree.iter().enumerate() {
            if ly + 1 >= inner.y + inner.height {
                break;
            }
            let c = prims::series_color(i, tree.len());
            buf[(inner.x + chart_w + 1, ly)].set_symbol("■").set_fg(c);
            let pct = node_total(node) / total * 100.0;
            let txt = format!(" {} {pct:.0}%", node.name);
            for (j, ch) in txt.chars().enumerate() {
                let col = inner.x + chart_w + 2 + j as u16;
                if col < inner.x + inner.width {
                    buf[(col, ly)].set_symbol(&ch.to_string()).set_style(prims::WHITE);
                }
            }
            ly += 1;
        }
    }
}

fn node_total(node: &TreeNode) -> f64 {
    if node.children.is_empty() {
        node.value
    } else {
        node.children.iter().map(node_total).sum()
    }
}

fn node_depth(node: &TreeNode) -> usize {
    if node.children.is_empty() {
        0
    } else {
        1 + node.children.iter().map(node_depth).max().unwrap_or(0)
    }
}

fn draw_node_px(
    px: &mut Vec<Option<Color>>,
    pw: usize,
    ph: usize,
    cxp: f64,
    cyp: f64,
    radp: f64,
    stride: f64,
    node: &TreeNode,
    total: f64,
    depth: usize,
    start_ang: f64,
    sweep: f64,
) {
    let val = node_total(node);
    if val <= 0.0 {
        return;
    }
    let frac = val / total;
    let node_sweep = sweep * frac;
    let c = prims::palette(6 + depth % 3);
    let r_in = radp * (0.06 + depth as f64 * stride);
    let r_out = radp * (0.06 + (depth + 1) as f64 * stride);
    for y in 0..ph {
        for x in 0..pw {
            let dx = x as f64 - cxp;
            let dy = y as f64 - cyp;
            let r = (dx * dx + dy * dy).sqrt();
            if r < r_in || r > r_out {
                continue;
            }
            let ang = dy.atan2(dx);
            let mut a = ang;
            while a < start_ang {
                a += std::f64::consts::TAU;
            }
            let rel = a - start_ang;
            if rel <= node_sweep + 1e-6 {
                let shade = (dx / radp * 0.3 + 0.7).clamp(0.0, 1.0);
                px[y * pw + x] = Some(crate::charts::cartesian::shade(c, shade));
            }
        }
    }
    if !node.children.is_empty() {
        let mut child_start = start_ang;
        for child in &node.children {
            let child_frac = node_total(child) / val;
            draw_node_px(
                px,
                pw,
                ph,
                cxp,
                cyp,
                radp,
                stride,
                child,
                val,
                depth + 1,
                child_start,
                node_sweep,
            );
            child_start += node_sweep * child_frac;
        }
    }
}
