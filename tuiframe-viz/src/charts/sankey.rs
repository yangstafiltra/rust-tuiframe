use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, parse_edges};
use crate::engine::Chart;
use crate::prims;

pub struct SankeyDiagram;

impl Chart for SankeyDiagram {
    fn name(&self) -> &'static str {
        "sankey_diagram"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            parse_edges(
                "Solar - Storage: 40\nSolar - Grid: 60\nWind - Storage: 35\nWind - Grid: 45\nStorage - Homes: 70\nGrid - Homes: 105\nGrid - Industry: 60\nHomes - Waste: 15",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Leads - Qualified: 600\nQualified - Demo: 320\nDemo - Proposal: 180\nProposal - Closed: 90\nLeads - Lost: 400\nQualified - Lost: 280\nDemo - Lost: 140",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "raw - process: 80\nraw - recycle: 20\nprocess - product: 60\nprocess - waste: 20\nrecycle - process: 15",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Coal - Power: 300\nGas - Power: 200\nPower - Grid: 480\nPower - Waste: 20\nSolar - Grid: 80\nWind - Grid: 120\nGrid - Homes: 350\nGrid - Industry: 250\nGrid - Export: 80",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Reach - Visit: 900\nVisit - Signup: 300\nVisit - Bounce: 600\nSignup - Demo: 120\nDemo - Purchase: 60\nDemo - Lost: 60\nPurchase - Renew: 40\nPurchase - Churn: 20",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Start - Task1: 50\nStart - Task2: 30\nTask1 - Task3: 40\nTask2 - Task3: 20\nTask3 - Done: 60\nTask1 - Blocked: 10\nBlocked - Done: 10",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Budget - Salaries: 500\nBudget - Supplies: 200\nBudget - Travel: 100\nSalaries - Product: 300\nSalaries - Ops: 200\nSupplies - Product: 150\nSupplies - Ops: 50\nTravel - Ops: 100",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Raw - Cut: 400\nCut - Sew: 350\nCut - Waste: 50\nSew - Pack: 320\nSew - Waste: 30\nPack - Ship: 300\nPack - Waste: 20",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "Inbound - Route A: 250\nInbound - Route B: 200\nRoute A - Hub: 240\nRoute A - Drop: 10\nRoute B - Hub: 190\nRoute B - Drop: 10\nHub - Deliver: 400\nHub - Hold: 30",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(8);
        let title = if data.title.is_empty() { "Sankey Diagram" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.edges.is_empty() {
            prims::text(buf, inner, 1, "no edges (use `A - B: w`)", prims::DIM);
            return;
        }
        // compute layers by longest path from source
        let mut indeg: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for e in &data.edges {
            adj.entry(e.from.clone()).or_default().push(e.to.clone());
            *indeg.entry(e.to.clone()).or_insert(0) += 1;
            indeg.entry(e.from.clone()).or_insert(0);
        }
        // topological layering via simple BFS depth
        let mut layer: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut queue: Vec<(String, usize)> = indeg
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(k, _)| (k.clone(), 0))
            .collect();
        while !queue.is_empty() {
            let (node, d) = queue.remove(0);
            if let Some(&prev) = layer.get(&node) {
                if prev >= d {
                    continue;
                }
            }
            layer.insert(node.clone(), d);
            if let Some(next) = adj.get(&node) {
                for t in next {
                    queue.push((t.clone(), d + 1));
                }
            }
        }
        // max depth
        let max_d = layer.values().copied().fold(0, usize::max);
        let depth_w = inner.width as usize / (max_d + 1).max(1);
        let x_lane: std::collections::HashMap<String, u16> = layer
            .iter()
            .map(|(k, d)| (k.clone(), inner.x + (d * depth_w + depth_w / 2) as u16))
            .collect();

        // node slots per layer
        let mut slots: std::collections::HashMap<usize, Vec<(&String, f64)>> = std::collections::HashMap::new();
        let mut out_total: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut in_total: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for e in &data.edges {
            *out_total.entry(e.from.clone()).or_insert(0.0) += e.weight;
            *in_total.entry(e.to.clone()).or_insert(0.0) += e.weight;
        }
        let all_nodes: std::collections::HashSet<String> = layer.keys().cloned().collect();
        let node_val = |n: &str| -> f64 { out_total.get(n).copied().unwrap_or(0.0).max(in_total.get(n).copied().unwrap_or(0.0)) };
        for (name, d) in &layer {
            slots.entry(*d).or_default().push((name, node_val(name)));
        }
        for v in slots.values_mut() {
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }
        // scale so total per layer fits plot height
        let max_total = slots
            .values()
            .map(|v| v.iter().map(|(_, val)| val).sum::<f64>())
            .fold(1.0_f64, f64::max);
        let plot_h = inner.height as f64 * 0.8;

        // node positions
        let mut node_y: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for (d, list) in &slots {
            let layer_total: f64 = list.iter().map(|(_, v)| v).sum();
            let mut y = inner.y as f64 + (inner.height as f64 - plot_h) / 2.0;
            for (name, val) in list {
                let hh = (val / max_total * plot_h).max(1.0);
                node_y.insert((*name).clone(), y);
                // draw node bar
                let nx = x_lane[*name];
                let c = prims::palette(8 + d % 3);
                for row in 0..(hh.round() as u16) {
                    buf[(nx, (y + row as f64).round() as u16)].set_symbol("█").set_fg(c);
                }
                // label
                if *d == max_d {
                    let lx = nx + 1;
                    if (lx + name.len() as u16) < (inner.x + inner.width) {
                        prims::abs_text(buf, lx, (y + hh / 2.0).round() as u16, name, prims::fg(Color::Gray));
                    }
                } else {
                    let lx = nx.saturating_sub(name.chars().count() as u16 + 1);
                    if lx >= inner.x {
                        prims::abs_text(buf, lx, (y + hh / 2.0).round() as u16, name, prims::fg(Color::Gray));
                    }
                }
                y += hh + 1.0;
            }
            let _ = layer_total;
        }
        // links between layers
        for e in &data.edges {
            let (x1, y1) = (x_lane[&e.from], node_y[&e.from]);
            let (x2, y2) = (x_lane[&e.to], node_y[&e.to]);
            let frac = e.weight / max_total * plot_h;
            let w = frac.clamp(1.0, 5.0) as u16;
            let c = prims::palette(8 + layer[&e.from] % 3);
            draw_link(buf, inner, x1, y1, x2, y2, w, c);
        }
        let _ = all_nodes;
    }
}

fn draw_link(buf: &mut Buffer, area: Rect, x1: u16, y1: f64, x2: u16, y2: f64, w: u16, color: Color) {
    let steps = (x2.saturating_sub(x1)) as i32;
    let steps = steps.max(1);
    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let x = x1 + s as u16;
        let y = y1 + (y2 - y1) * t;
        let yy = y.round() as u16;
        let half = w / 2;
        for dy in yy.saturating_sub(half)..=(yy + half) {
            if x >= area.x && x < area.x + area.width && dy >= area.y && dy < area.y + area.height {
                buf[(x, dy)].set_symbol("·").set_fg(color);
            }
        }
    }
}
