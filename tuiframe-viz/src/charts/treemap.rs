use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, TreeNode};
use crate::engine::Chart;
use crate::prims;

pub struct Treemap;

impl Chart for Treemap {
    fn name(&self) -> &'static str {
        "treemap"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            ChartData {
                title: "Storage Usage (GB)".to_string(),
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
                title: "Product Revenue".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Hardware/Laptops", 120.0),
                    ("Hardware/Phones", 90.0),
                    ("Hardware/Tablets", 40.0),
                    ("Software/SaaS", 80.0),
                    ("Software/Licenses", 45.0),
                    ("Services/Support", 30.0),
                    ("Services/Consulting", 25.0),
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
                    ("Social/LinkedIn", 4.0),
                    ("Referral/Blogs", 5.0),
                    ("Referral/News", 3.0),
                    ("Email", 2.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Cloud Spend".to_string(),
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
                title: "Budget Allocation".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("R&D/Research", 45.0),
                    ("R&D/Dev", 30.0),
                    ("Marketing/Ads", 15.0),
                    ("Marketing/Events", 5.0),
                    ("Sales/Team", 12.0),
                    ("Sales/Tools", 8.0),
                    ("Ops/Infra", 10.0),
                    ("Ops/Support", 6.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Disk Usage by User".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Alice/Docs", 15.0),
                    ("Alice/Photos", 40.0),
                    ("Bob/Projects", 25.0),
                    ("Bob/Backups", 35.0),
                    ("Carol/Media", 55.0),
                    ("Carol/Temp", 10.0),
                    ("Shared/Common", 20.0),
                    ("Shared/Archive", 30.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Energy Mix".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Fossil/Coal", 30.0),
                    ("Fossil/Gas", 20.0),
                    ("Renewable/Wind", 18.0),
                    ("Renewable/Solar", 14.0),
                    ("Renewable/Hydro", 8.0),
                    ("Renewable/Geo", 3.0),
                    ("Nuclear", 5.0),
                    ("Other", 2.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Traffic Sources (k sessions)".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Mobile/iOS", 60.0),
                    ("Mobile/Android", 55.0),
                    ("Desktop/Windows", 40.0),
                    ("Desktop/Mac", 30.0),
                    ("Tablet/iPad", 15.0),
                    ("Tablet/Android", 10.0),
                    ("TV/Streaming", 8.0),
                ]),
                ..Default::default()
            },
            ChartData {
                title: "Grocery Basket".to_string(),
                tree: crate::data::tree_from_flat(&[
                    ("Produce/Fruit", 12.0),
                    ("Produce/Veg", 15.0),
                    ("Dairy/Milk", 5.0),
                    ("Dairy/Cheese", 8.0),
                    ("Meat/Beef", 20.0),
                    ("Meat/Chicken", 12.0),
                    ("Bakery/Bread", 4.0),
                    ("Snacks", 6.0),
                    ("Frozen", 8.0),
                ]),
                ..Default::default()
            },
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(5);
        let title = if data.title.is_empty() { "Treemap" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.tree.is_empty() {
            // fall back to series
            if let Some(s) = data.series.first() {
                let leaves: Vec<(&str, f64)> = data
                    .labels
                    .iter()
                    .zip(s.values.iter())
                    .map(|(l, &v)| (l.as_str(), v))
                    .collect();
                let tree = crate::data::tree_from_flat(&leaves);
                draw_tree(buf, inner, &tree, 0);
            }
            return;
        }
        draw_tree(buf, inner, &data.tree, 0);
    }
}

pub fn draw_tree(buf: &mut Buffer, area: Rect, nodes: &[TreeNode], _depth: usize) {
    // flatten leaves with values
    let mut leaves: Vec<(&TreeNode, f64)> = Vec::new();
    collect_leaves(nodes, &mut leaves);
    let total: f64 = leaves.iter().map(|(_, v)| v).sum();
    if total <= 0.0 || leaves.is_empty() {
        return;
    }
    let mut x0 = area.x;
    let mut y0 = area.y;
    let mut w = area.width;
    let mut h = area.height;
    squarify(buf, area, &mut leaves, total, &mut x0, &mut y0, &mut w, &mut h, 0);
}

fn collect_leaves<'a>(nodes: &'a [TreeNode], out: &mut Vec<(&'a TreeNode, f64)>) {
    for n in nodes {
        if n.children.is_empty() {
            out.push((n, n.value));
        } else {
            collect_leaves(&n.children, out);
        }
    }
}

fn squarify(
    buf: &mut Buffer,
    area: Rect,
    leaves: &mut Vec<(&TreeNode, f64)>,
    total: f64,
    x0: &mut u16,
    y0: &mut u16,
    w: &mut u16,
    h: &mut u16,
    depth: usize,
) {
    if leaves.is_empty() || *w == 0 || *h == 0 {
        return;
    }
    let c = prims::palette(5 + depth % 3);
    // simple strip layout: split by rows
    let mut used_area = 0.0;
    let rows: Vec<(usize, f64)> = Vec::new(); // (count, total value)
    let mut i = 0;
    while i < leaves.len() {
        let row_w = *w as f64;
        let mut row_total = 0.0;
        let mut j = i;
        let mut best = f64::INFINITY;
        let mut take = 1;
        while j < leaves.len() {
            row_total += leaves[j].1;
            let row_h = row_total / total * *h as f64;
            let aspect = if row_h > 0.0 { (row_w / row_h).max(row_h / row_w) } else { f64::INFINITY };
            if aspect < best {
                best = aspect;
                take = j - i + 1;
            }
            j += 1;
        }
        let mut row_total_take = 0.0;
        for k in i..i + take {
            row_total_take += leaves[k].1;
        }
        let row_h = (row_total_take / total * *h as f64).round().max(1.0) as u16;
        // draw each item in this row
        let mut cx = *x0;
        for k in i..i + take {
            let (node, v) = leaves[k];
            let item_w = ((v / total * *w as f64)).round().max(1.0) as u16;
            // fill
            let shade = 0.5 + 0.2 * ((k + depth) % 3) as f64;
            let cc = crate::charts::cartesian::shade(c, shade);
            for yy in *y0..(*y0 + row_h).min(*y0 + *h) {
                for xx in cx..(cx + item_w).min(*x0 + *w) {
                    buf[(xx, yy)].set_symbol("█").set_fg(cc);
                }
            }
            // label (truncated)
            let label = &node.name;
            if item_w >= 4 && row_h >= 1 {
                let clip = label.chars().take((item_w as usize).saturating_sub(1)).collect::<String>();
                if !clip.is_empty() {
                    prims::abs_text(buf, cx, *y0, &clip, prims::fg(Color::White));
                }
            }
            cx += item_w;
        }
        // recurse into row if deep nodes remain (not used here since leaves already flattened)
        let _ = used_area;
        let _ = rows;
        *y0 += row_h;
        *h = h.saturating_sub(row_h);
        i += take;
        used_area += row_total_take;
        if *h == 0 {
            break;
        }
    }
    let _ = area;
}
