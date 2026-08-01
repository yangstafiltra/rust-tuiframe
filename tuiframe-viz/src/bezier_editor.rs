use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crossterm::event::{MouseEvent, MouseEventKind};

use crate::bezier::{Bezier, density_for};
use crate::prims;

/// Interactive bezier easing editor.
///
/// Drag any interior control handle to reshape the curve, click empty space on
/// the curve to insert a new control point, `x` deletes the selected point.
/// `1/2/3` saves the current curve into one of three slots; `Shift+1/2/3`
/// loads a slot back. A live preview at the bottom animates a dot whose
/// vertical position follows the curve, so the easing feel is visible
/// immediately.
///
/// The curve you are editing is drawn bright; the template it came from is
/// drawn dim behind it so a drag's effect is obvious.
pub struct BezierEditor {
    pub curve: Bezier,
    /// The curve this one started from (last applied preset / slot). Drawn dim
    /// while `curve` is being edited, so the diff is visible.
    template: Bezier,
    pub preset_idx: Option<usize>,
    /// Dragged handle index into `curve.points`, when any.
    drag: Option<usize>,
    preview_t: f64,
    last_plot: Option<Rect>,
    /// Three save slots, valid for the lifetime of this editor session.
    slots: [Option<Bezier>; 3],
    /// Where the current curve came from: a preset name, a slot number, or a
    /// custom edit. Shown in the main footer as the active easing label.
    source: CurveSource,
}

/// Origin of the current curve, for display in the footer (`ease: ...`).
#[derive(Clone, PartialEq, Debug)]
pub enum CurveSource {
    /// From a named preset (e.g. "ease-in-out").
    Preset(String),
    /// Loaded from a save slot (1..=3).
    Slot(usize),
    /// Manually edited / no matching preset.
    Custom,
}

const HANDLE_RADIUS: u16 = 2;

impl BezierEditor {
    pub fn new() -> Self {
        let curve = Bezier::linear();
        BezierEditor {
            template: curve.clone(),
            curve,
            preset_idx: Some(0),
            drag: None,
            preview_t: 0.0,
            last_plot: None,
            slots: [None, None, None],
            source: CurveSource::Preset("linear".to_string()),
        }
    }

    /// Human-readable label of the current curve, for the footer.
    pub fn label(&self) -> String {
        match &self.source {
            CurveSource::Preset(name) => name.clone(),
            CurveSource::Slot(n) => n.to_string(),
            CurveSource::Custom => "custom".to_string(),
        }
    }

    /// Build an editor preloaded with an existing curve and its footer label,
    /// without re-resolving a preset. Used when reopening the editor with a
    /// custom or slot-loaded curve.
    pub fn from_curve(curve: Bezier, label: &str) -> Self {
        let mut e = BezierEditor::new();
        e.curve = curve.clone();
        e.template = curve;
        e.preset_idx = None;
        e.source = match label.parse::<usize>() {
            Ok(n) if (1..=3).contains(&n) => CurveSource::Slot(n),
            _ if label == "custom" => CurveSource::Custom,
            _ => CurveSource::Preset(label.to_string()),
        };
        e
    }

    pub fn apply_preset(&mut self, idx: usize) {
        if let Some(p) = crate::easing_presets::PRESETS.get(idx) {
            self.curve = p.bezier();
            self.template = self.curve.clone();
            self.preset_idx = Some(idx);
            self.drag = None;
            self.source = CurveSource::Preset(p.name.to_string());
        }
    }

    pub fn next_preset(&mut self) {
        let n = crate::easing_presets::PRESETS.len();
        let cur = self.preset_idx.unwrap_or(0);
        self.apply_preset((cur + 1) % n);
    }

    pub fn prev_preset(&mut self) {
        let n = crate::easing_presets::PRESETS.len();
        let cur = self.preset_idx.unwrap_or(0);
        self.apply_preset((cur + n - 1) % n);
    }

    /// Advance the live preview animation by `dt` seconds.
    pub fn tick(&mut self, dt: f64) {
        self.preview_t += dt * 0.8;
        if self.preview_t > 1.0 {
            self.preview_t -= 1.0;
        }
    }

    /// Handle a mouse event. Returns true if the event was consumed.
    pub fn handle_mouse(&mut self, ev: &MouseEvent) -> bool {
        match ev.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                self.drag = self.hit_handle(ev.column, ev.row);
                if self.drag.is_some() {
                    return true;
                }
                // Not on a handle: try adding a control point near the click.
                if self.add_point_at(ev.column, ev.row) {
                    self.drag = self.hit_handle(ev.column, ev.row);
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                if let Some(idx) = self.hit_handle(ev.column, ev.row) {
                    self.curve.remove_point(idx);
                    self.preset_idx = None;
                    self.drag = None;
                    self.source = CurveSource::Custom;
                    return true;
                }
                false
            }
            MouseEventKind::Drag(..) => {
                if let Some(idx) = self.drag {
                    self.move_handle(idx, ev.column, ev.row);
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Up(..) => {
                let was = self.drag.is_some();
                self.drag = None;
                was
            }
            _ => false,
        }
    }

    /// Save the current curve into slot `i in 0..3`. After saving, the curve is
    /// labeled by that slot so the footer can show `ease: <n>`.
    pub fn save_slot(&mut self, i: usize) {
        if i < 3 {
            self.slots[i] = Some(self.curve.clone());
            self.source = CurveSource::Slot(i + 1);
        }
    }

    /// Delete the currently dragged/selected control point. No-op when none.
    pub fn delete_selected(&mut self) {
        if let Some(idx) = self.drag {
            self.curve.remove_point(idx);
            self.drag = None;
            self.preset_idx = None;
            self.source = CurveSource::Custom;
        }
    }

    /// Load slot `i` back into the editor. Returns true if a slot held a curve.
    pub fn load_slot(&mut self, i: usize) -> bool {
        if let Some(c) = self.slots.get(i).and_then(|s| s.clone()) {
            self.template = self.curve.clone();
            self.curve = c;
            self.preset_idx = None;
            self.drag = None;
            self.source = CurveSource::Slot(i + 1);
            true
        } else {
            false
        }
    }

    fn hit_handle(&self, cx: u16, cy: u16) -> Option<usize> {
        let plot = self.last_plot?;
        let near = |hx: f64, hy: f64| {
            let hcol = plot.x + (hx * (plot.width - 1) as f64).round() as u16;
            let hrow = plot.y + ((1.0 - hy) * (plot.height - 1) as f64).round() as u16;
            cx.abs_diff(hcol) <= HANDLE_RADIUS && cy.abs_diff(hrow) <= HANDLE_RADIUS
        };
        self.curve.handles().find(|(_, p)| near(p.0, p.1)).map(|(i, _)| i)
    }

    /// Insert a new control point where the click landed, if it's inside the
    /// plot and close to the current curve path.
    fn add_point_at(&mut self, cx: u16, cy: u16) -> bool {
        let Some(plot) = self.last_plot else {
            return false;
        };
        if plot.width < 2 || plot.height < 2 {
            return false;
        }
        let nx = ((cx.saturating_sub(plot.x) as f64 / (plot.width - 1) as f64)).clamp(0.0, 1.0);
        let ny = 1.0 - (cy.saturating_sub(plot.y) as f64 / (plot.height - 1) as f64).clamp(0.0, 1.0);
        // Only add when the click is near the curve (within ~3 cells).
        let (ccx, ccy) = self.to_cell(plot, nx, ny);
        let curve_pt = self.curve.sample(nx);
        let (pcx, pcy) = self.to_cell(plot, nx, curve_pt);
        if ccx.abs_diff(pcx) > 3 || ccy.abs_diff(pcy) > 3 {
            return false;
        }
        let target = curve_pt;
        // Insert between the two control points bracketing x=nx.
        let mut idx = self.curve.points.len() - 1;
        for (i, p) in self.curve.points.iter().enumerate() {
            if p.0 > nx {
                idx = i;
                break;
            }
        }
        self.curve.insert_point(idx, (nx, target));
        self.preset_idx = None;
        self.source = CurveSource::Custom;
        true
    }

    fn move_handle(&mut self, idx: usize, cx: u16, cy: u16) {
        let Some(plot) = self.last_plot else {
            return;
        };
        if plot.width < 2 || plot.height < 2 {
            return;
        }
        if idx >= self.curve.points.len() {
            return;
        }
        let nx = ((cx.saturating_sub(plot.x) as f64 / (plot.width - 1) as f64)).clamp(0.0, 1.0);
        let ny = 1.0 - (cy.saturating_sub(plot.y) as f64 / (plot.height - 1) as f64).clamp(0.0, 1.0);
        self.curve.points[idx] = (nx, ny);
        self.preset_idx = None;
        self.source = CurveSource::Custom;
    }

    fn plot_rect(&self, area: Rect) -> Rect {
        Rect::new(
            area.x + 2,
            area.y + 2,
            area.width.saturating_sub(4),
            area.height.saturating_sub(6),
        )
    }

    /// Render the editor into the buffer.
    pub fn render(&mut self, buf: &mut Buffer, area: Rect) {
        let color = prims::palette(0);
        prims::frame(buf, area, "Bezier Easing Editor", color);
        let plot = self.plot_rect(area);
        self.last_plot = Some(plot);

        self.render_presets(buf, area);
        self.render_grid(buf, plot);
        self.render_polygon(buf, plot);
        self.render_curve(buf, plot);
        self.render_handles(buf, plot);
        self.render_slots(buf, area);
        self.render_preview(buf, area);
    }

    fn render_presets(&self, buf: &mut Buffer, area: Rect) {
        let names = crate::easing_presets::names();
        let n = names.len();
        let cur = self.preset_idx.unwrap_or(0);
        let max_chars = (area.width.saturating_sub(4)) as usize;
        let label_len = |i: usize| names[i].len() + 3; // spacing around each name
        let total = |s: usize, e: usize| (s..e).map(label_len).sum::<usize>();
        // Sliding window grown around the current preset, trimmed to fit width.
        let mut start = cur.min(n.saturating_sub(1));
        let mut end = (start + 1).min(n);
        loop {
            if start > 0 && total(start - 1, end) <= max_chars {
                start -= 1;
            } else if end < n && total(start, end + 1) <= max_chars {
                end += 1;
            } else {
                break;
            }
        }
        let mut x = area.x + 2;
        let y = area.y + 1;
        if start > 0 {
            prims::abs_text(buf, x, y, "…", prims::DIM);
            x += 1;
        }
        for i in start..end {
            let is_cur = Some(i) == self.preset_idx;
            let label = if is_cur { format!("[{}]", names[i]) } else { format!(" {} ", names[i]) };
            for ch in label.chars() {
                if x >= area.x + area.width - 1 {
                    break;
                }
                let style = if is_cur {
                    Style::new().fg(Color::Black).bg(prims::palette(0))
                } else {
                    prims::DIM
                };
                buf[(x, y)].set_symbol(&ch.to_string()).set_style(style);
                x += 1;
            }
            x += 1;
        }
        if end < n {
            prims::abs_text(buf, x, y, "…", prims::DIM);
        }
    }

    fn render_grid(&self, buf: &mut Buffer, plot: Rect) {
        let st = prims::fg(Color::DarkGray);
        for cx in plot.x..plot.x + plot.width {
            buf[(cx, plot.y)].set_symbol("─").set_style(st);
            buf[(cx, plot.y + plot.height - 1)].set_symbol("─").set_style(st);
        }
        for cy in plot.y..plot.y + plot.height {
            buf[(plot.x, cy)].set_symbol("│").set_style(st);
            buf[(plot.x + plot.width - 1, cy)].set_symbol("│").set_style(st);
        }
        buf[(plot.x, plot.y)].set_symbol("┌").set_style(st);
        buf[(plot.x + plot.width - 1, plot.y)].set_symbol("┐").set_style(st);
        buf[(plot.x, plot.y + plot.height - 1)].set_symbol("└").set_style(st);
        buf[(plot.x + plot.width - 1, plot.y + plot.height - 1)].set_symbol("┘").set_style(st);
        let mx = plot.x + plot.width / 2;
        for cy in plot.y + 1..plot.y + plot.height - 1 {
            buf[(mx, cy)].set_symbol("┆").set_style(st);
        }
        let my = plot.y + plot.height / 2;
        for cx in plot.x + 1..plot.x + plot.width - 1 {
            buf[(cx, my)].set_symbol("┄").set_style(st);
        }
        prims::abs_text(buf, plot.x + 1, plot.y, "y=1", prims::DIM);
        prims::abs_text(buf, plot.x + 1, plot.y + plot.height, "t→", prims::DIM);
    }

    fn to_cell(&self, plot: Rect, x: f64, y: f64) -> (u16, u16) {
        (
            plot.x + (x.clamp(0.0, 1.0) * (plot.width - 1) as f64).round() as u16,
            plot.y + ((1.0 - y.clamp(0.0, 1.0)) * (plot.height - 1) as f64).round() as u16,
        )
    }

    fn render_polygon(&self, buf: &mut Buffer, plot: Rect) {
        let st = prims::fg(Color::DarkGray).add_modifier(Modifier::DIM);
        let mut prev: Option<(u16, u16)> = None;
        for p in &self.curve.points {
            let (x, y) = self.to_cell(plot, p.0, p.1);
            if let Some((px, py)) = prev {
                line(buf, px, py, x, y, st);
            }
            prev = Some((x, y));
        }
    }

    fn render_curve(&self, buf: &mut Buffer, plot: Rect) {
        // Template (origin) curve: dim, drawn first so the edited curve stands out.
        if self.template != self.curve {
            let density = density_for(plot.width);
            let pts = self.template.samples(density);
            let style = prims::fg(Color::DarkGray).add_modifier(Modifier::DIM);
            for w in pts.windows(2) {
                let (ax, ay) = self.to_cell(plot, w[0].0, w[0].1);
                let (bx, by) = self.to_cell(plot, w[1].0, w[1].1);
                line(buf, ax, ay, bx, by, style);
            }
        }
        // Active curve: bright. The `eased` marker shows the live preview point.
        let density = density_for(plot.width);
        let pts = self.curve.samples(density);
        let style = prims::fg(prims::palette(0)).add_modifier(Modifier::BOLD);
        for w in pts.windows(2) {
            let (ax, ay) = self.to_cell(plot, w[0].0, w[0].1);
            let (bx, by) = self.to_cell(plot, w[1].0, w[1].1);
            line(buf, ax, ay, bx, by, style);
        }
        // Preview dot travels along the curve.
        let (px, py) = self.to_cell(plot, self.preview_t, self.curve.sample(self.preview_t));
        buf[(px, py)].set_symbol("◉").set_style(prims::fg(Color::Yellow).add_modifier(Modifier::BOLD));
    }

    fn render_handles(&self, buf: &mut Buffer, plot: Rect) {
        for (i, p) in self.curve.handles() {
            let (hx, hy) = self.to_cell(plot, p.0, p.1);
            let active = self.drag == Some(i);
            let style = if active {
                prims::fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                prims::fg(Color::White)
            };
            buf[(hx, hy)].set_symbol("●").set_style(style);
        }
        let (x0, y0) = self.to_cell(plot, 0.0, 0.0);
        let (x3, y3) = self.to_cell(plot, 1.0, 1.0);
        buf[(x0, y0)].set_symbol("○").set_style(prims::DIM);
        buf[(x3, y3)].set_symbol("○").set_style(prims::DIM);
    }

    fn render_slots(&self, buf: &mut Buffer, area: Rect) {
        let y = area.y + area.height - 4;
        prims::abs_text(buf, area.x + 2, y, "slots:", prims::DIM);
        let mut x = area.x + 2 + "slots:".len() as u16;
        for (i, slot) in self.slots.iter().enumerate() {
            let label = match slot {
                Some(_) => format!(" [{}:●]", i + 1),
                None => format!(" [{}: ]", i + 1),
            };
            prims::abs_text(buf, x, y, &label, if slot.is_some() { prims::fg(prims::palette(0)) } else { prims::DIM });
            x += label.len() as u16;
        }
        let hint = "[1-3] save  [Shift+1-3] load  [click] add pt  [x/right-click] delete";
        if x + (hint.len() as u16) < area.x + area.width {
            prims::abs_text(buf, x + 2, y, hint, prims::DIM);
        }
    }

    fn render_preview(&self, buf: &mut Buffer, area: Rect) {
        let y = area.y + area.height - 2;
        let label = format!(
            "preview: {}",
            self.preset_idx.map(|i| crate::easing_presets::PRESETS[i].name).unwrap_or("custom")
        );
        prims::abs_text(buf, area.x + 2, y - 1, &label, prims::DIM);
        let x0 = area.x + 2;
        let x1 = area.x + area.width - 3;
        let n = (x1 - x0) as usize;
        let progress = self.curve.sample(self.preview_t);
        let marker = (self.preview_t * n as f64) as usize;
        for i in 0..n {
            let (ch, st) = if i == marker {
                ("▮", prims::fg(prims::palette(0)))
            } else {
                ("─", prims::fg(Color::DarkGray))
            };
            buf[(x0 + i as u16, y)].set_symbol(ch).set_style(st);
        }
        let height = (progress * (y.saturating_sub(area.y + 2)) as f64).round() as u16;
        let dot_row = y.saturating_sub(1).saturating_sub(height);
        buf[(x0 + marker as u16, dot_row)].set_symbol("●").set_style(prims::fg(prims::palette(0)));
        let txt = format!("t {:.2} → y {:.2}", self.preview_t, progress);
        prims::abs_text(buf, area.x + 2, y, &txt, prims::DIM);
    }
}

impl Default for BezierEditor {
    fn default() -> Self {
        Self::new()
    }
}

// ---- line drawing ----

fn line(buf: &mut Buffer, x0: u16, y0: u16, x1: u16, y1: u16, style: Style) {
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 && (x as u16) < buf.area.width && (y as u16) < buf.area.height {
            let cell = &buf[(x as u16, y as u16)];
            if cell.symbol() == " " {
                buf[(x as u16, y as u16)].set_symbol("·").set_style(style);
            } else {
                buf[(x as u16, y as u16)].set_style(style);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

    fn editor_with_plot() -> (BezierEditor, Rect) {
        let mut e = BezierEditor::new();
        let area = Rect::new(0, 0, 100, 32);
        // Force last_plot by rendering once.
        let buf = ratatui::buffer::Buffer::empty(area);
        let mut buf = buf;
        e.render(&mut buf, area);
        (e, area)
    }

    #[test]
    fn handles_iterate_interior_points() {
        let b = crate::bezier::Bezier::cubic((0.0, 0.0), (0.3, 0.4), (0.7, 0.6), (1.0, 1.0));
        let hs: Vec<_> = b.handles().collect();
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].1, (0.3, 0.4));
        assert_eq!(hs[1].1, (0.7, 0.6));
    }

    #[test]
    fn insert_point_grows_curve_and_keeps_anchors() {
        let mut b = crate::bezier::Bezier::cubic((0.0, 0.0), (0.3, 0.4), (0.7, 0.6), (1.0, 1.0));
        b.insert_point(2, (0.5, 0.5));
        assert_eq!(b.points.len(), 5);
        assert_eq!(b.p0(), (0.0, 0.0));
        assert_eq!(b.p3(), (1.0, 1.0));
        // sampling still works over the extended curve
        assert!((b.sample(0.0)).abs() < 1e-6);
        assert!((b.sample(1.0) - 1.0).abs() < 1e-6);
        b.remove_point(2);
        assert_eq!(b.points.len(), 4);
    }

    #[test]
    fn remove_point_never_below_two() {
        let mut b = crate::bezier::Bezier::linear();
        b.remove_point(1);
        b.remove_point(1);
        assert_eq!(b.points.len(), 2); // both interior handles removable
        let mut tiny = crate::bezier::Bezier::cubic((0.0, 0.0), (0.2, 0.2), (0.8, 0.8), (1.0, 1.0));
        tiny.remove_point(1);
        assert_eq!(tiny.points.len(), 3);
        tiny.remove_point(1);
        assert_eq!(tiny.points.len(), 2);
        tiny.remove_point(1);
        assert_eq!(tiny.points.len(), 2); // cannot drop below the two anchors
    }

    #[test]
    fn slots_save_and_load() {
        let mut e = BezierEditor::new();
        e.apply_preset(3); // ease-in-out
        let saved = e.curve.clone();
        e.save_slot(0);
        e.apply_preset(0); // linear
        assert!(e.load_slot(0));
        assert_eq!(e.curve, saved);
        assert!(!e.load_slot(1));
    }

    #[test]
    fn source_label_tracks_preset_slot_custom() {
        let mut e = BezierEditor::new();
        e.apply_preset(3); // ease-in-out
        assert_eq!(e.label(), "ease-in-out");
        e.save_slot(1); // slot 2
        assert_eq!(e.label(), "2");
        e.load_slot(1);
        assert_eq!(e.label(), "2");
        e.apply_preset(0);
        assert_eq!(e.label(), "linear");
        e.curve.points[1] = (0.4, 0.6); // manual edit
        e.source = CurveSource::Custom;
        assert_eq!(e.label(), "custom");
    }

    #[test]
    fn from_curve_carries_slot_label() {
        let b = crate::bezier::Bezier::cubic((0.0, 0.0), (0.2, 0.3), (0.8, 0.9), (1.0, 1.0));
        let e = BezierEditor::from_curve(b, "3");
        assert_eq!(e.label(), "3");
        assert_eq!(e.curve.points.len(), 4);
        let e2 = BezierEditor::from_curve(crate::bezier::Bezier::linear(), "custom");
        assert_eq!(e2.label(), "custom");
    }

    #[test]
    fn template_dims_until_edited() {
        let mut e = BezierEditor::new();
        e.apply_preset(3);
        assert_eq!(e.template, e.curve);
        // simulate a drag on the p1 handle
        let (mut ed, area) = editor_with_plot();
        ed.apply_preset(3);
        let plot = ed.last_plot.unwrap();
        let (hx, hy) = ed.to_cell(plot, 0.5, 0.5);
        let ev = MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: hx, row: hy, modifiers: crossterm::event::KeyModifiers::NONE };
        ed.handle_mouse(&ev);
        assert!(ed.drag.is_some());
        let drag = MouseEvent { kind: MouseEventKind::Drag(MouseButton::Left), column: hx + 10, row: hy, modifiers: crossterm::event::KeyModifiers::NONE };
        ed.handle_mouse(&drag);
        assert_ne!(ed.curve, ed.template);
        let _ = area;
    }

    #[test]
    fn curve_samples_through_extended_points() {
        // 5-point curve should still be monotonic-ish for linear-ish points
        let mut b = crate::bezier::Bezier::cubic((0.0, 0.0), (0.2, 0.0), (0.8, 1.0), (1.0, 1.0));
        b.insert_point(2, (0.5, 0.5));
        let prev = b.sample(0.25);
        let cur = b.sample(0.5);
        let next = b.sample(0.75);
        assert!(prev < cur && cur < next);
    }
}
