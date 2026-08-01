use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::data::{ChartData, parse_edges};
use crate::engine::Chart;
use crate::prims;

pub struct NetworkGraph;

impl Chart for NetworkGraph {
    fn name(&self) -> &'static str {
        "network_graph"
    }

    fn presets(&self) -> Vec<ChartData> {
        vec![
            parse_edges(
                "router - switch_a: 10\nrouter - switch_b: 8\nswitch_a - server1: 5\nswitch_a - server2: 6\nswitch_b - server3: 4\nswitch_b - server4: 3\nserver1 - server3: 2\nswitch_a - server3: 1\nrouter - firewall: 7\nfirewall - internet: 9",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "hub - node1: 5\nhub - node2: 4\nhub - node3: 6\nnode1 - node4: 3\nnode2 - node4: 2\nnode3 - node4: 4\nnode4 - leaf: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "core - edge1: 9\ncore - edge2: 8\nedge1 - access1: 3\nedge1 - access2: 2\nedge2 - access3: 4\naccess1 - user_a: 1\naccess2 - user_b: 1\naccess3 - user_c: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "gateway - wifi1: 7\nwifi1 - laptop1: 2\nwifi1 - laptop2: 3\nwifi1 - phone1: 1\ngateway - wifi2: 6\nwifi2 - desktop1: 4\nwifi2 - server1: 5\nserver1 - printer: 1\ngateway - wired1: 5\nwired1 - iot1: 1\nwired1 - iot2: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "city - airport: 9\ncity - bus: 6\nbus - station1: 3\nbus - station2: 2\nairport - region_a: 4\nairport - region_b: 5\nregion_a - town1: 1\nregion_a - town2: 2\nregion_b - town3: 1\nregion_b - town4: 3",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "social - alice: 5\nsocial - bob: 4\nalice - carol: 3\nalice - dave: 2\nbob - carol: 2\nbob - eve: 3\ncarol - frank: 2\ndave - frank: 1\neve - grace: 1\nfrank - grace: 1\nalice - bob: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "power - plant1: 9\npower - plant2: 7\nplant1 - home1: 3\nplant1 - home2: 2\nplant1 - office: 4\nplant2 - home3: 3\nplant2 - factory: 5\noffice - server: 2\nfactory - home4: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "root - child1: 6\nroot - child2: 5\nroot - child3: 4\nchild1 - grand1: 3\nchild1 - grand2: 2\nchild2 - grand3: 3\nchild3 - grand4: 2\nchild3 - grand5: 1\ngrand1 - leaf1: 1\ngrand2 - leaf2: 1\ngrand3 - leaf3: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
            parse_edges(
                "train - stop_a: 8\ntrain - stop_b: 7\nstop_a - stop_c: 4\nstop_b - stop_c: 3\nstop_c - end: 2\ntrain - cargo: 5\ncargo - depot: 3\ndepot - stop_b: 1",
            )
            .unwrap_or_else(|_| ChartData::single(vec![1.0])),
        ]
    }

    fn render(&mut self, buf: &mut Buffer, area: Rect, data: &ChartData) {
        let color = prims::palette(7);
        let title = if data.title.is_empty() { "Network Graph" } else { &data.title };
        prims::frame(buf, area, title, color);
        let inner = Rect::new(area.x + 2, area.y + 2, area.width.saturating_sub(4), area.height.saturating_sub(4));
        if data.edges.is_empty() {
            prims::text(buf, inner, 1, "no edges (use `A - B: w`)", prims::DIM);
            return;
        }
        // collect unique nodes, assign a fixed layout ring
        let mut nodes: Vec<String> = Vec::new();
        for e in &data.edges {
            if !nodes.contains(&e.from) {
                nodes.push(e.from.clone());
            }
            if !nodes.contains(&e.to) {
                nodes.push(e.to.clone());
            }
        }
        let n = nodes.len();
        let cx = inner.x + inner.width / 2;
        let cy = inner.y + inner.height / 2;
        let rad = (inner.width.min(inner.height) as f64 * 0.40) as f64;
        let pos: Vec<(u16, u16)> = nodes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let ang = std::f64::consts::FRAC_PI_2 - (i as f64 / n as f64) * std::f64::consts::TAU;
                let x = (cx as f64 + ang.cos() * rad).round() as u16;
                let y = (cy as f64 - ang.sin() * rad).round() as u16;
                (x, y)
            })
            .collect();
        // edges
        let max_w = data.edges.iter().map(|e| e.weight).fold(1.0_f64, f64::max);
        for e in &data.edges {
            let fi = nodes.iter().position(|x| x == &e.from).unwrap();
            let ti = nodes.iter().position(|x| x == &e.to).unwrap();
            let (x1, y1) = pos[fi];
            let (x2, y2) = pos[ti];
            let wgt = (e.weight / max_w).clamp(0.0, 1.0);
            let c = crate::charts::cartesian::shade(Color::LightCyan, 0.3 + 0.7 * wgt);
            draw_line(buf, area, x1, y1, x2, y2, c);
        }
        // nodes
        for (i, name) in nodes.iter().enumerate() {
            let (x, y) = pos[i];
            let deg = data.edges.iter().filter(|e| e.from == *name || e.to == *name).count();
            let c = prims::series_color(i, nodes.len());
            buf[(x, y)].set_symbol("●").set_fg(c);
            // label
            let lx = x.saturating_sub(name.chars().count() as u16 / 2).min(inner.x + inner.width.saturating_sub(name.chars().count() as u16));
            let ly = y + 1;
            if ly < inner.y + inner.height {
                prims::abs_text(buf, lx, ly, name, prims::fg(Color::Gray));
            }
            let _ = deg;
        }
    }
}

pub fn draw_line(buf: &mut Buffer, area: Rect, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 && (x as u16) < area.width && (y as u16) < area.height {
            buf[(x as u16, y as u16)].set_symbol("·").set_fg(color);
        }
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
