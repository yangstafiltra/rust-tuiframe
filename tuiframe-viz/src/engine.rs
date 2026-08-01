use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use crate::bezier_editor::BezierEditor;
use crate::data::{Axis, ChartData, interpolate, parse_data};
use crate::prims;

/// A chart: knows how to render its data into a buffer.
pub trait Chart {
    fn name(&self) -> &'static str;
    fn presets(&self) -> Vec<ChartData>;
    fn render(&mut self, buf: &mut ratatui::buffer::Buffer, area: Rect, data: &ChartData);
    /// Natural axis range (x_min, x_max, y_min, y_max) for this dataset, used
    /// by the engine to animate the scale during transitions. Return `None`
    /// if the chart has no animatable cartesian scale.
    fn natural_scale(&self, _data: &ChartData) -> Option<Axis> {
        None
    }
}

const FRAME: Duration = Duration::from_millis(16); // ~60 fps
const TWEEN_SECS: f32 = 0.85; // seconds per PPT-style transition
const GRID_TWEEN_SECS: f32 = 1.6; // grid lags data (parallax: far layer)

pub struct Engine {
    chart: Box<dyn Chart>,
    presets: Vec<ChartData>,
    current: ChartData,
    from: ChartData,
    target: ChartData,
    tween: f32,
    grid_tween: f32,
    preset_idx: usize,
    editor: Option<BezierEditor>,
    /// The easing curve currently in effect (persists after the editor closes).
    active_curve: crate::bezier::Bezier,
    /// Footer label for the active curve: preset name, slot number, or "custom".
    active_ease_label: String,
    input_mode: bool,
    input_buf: String,
    input_err: Option<String>,
    msg: Option<String>,
    msg_until: Option<Instant>,
    quit: bool,
}

impl Engine {
    pub fn new(chart: Box<dyn Chart>) -> Self {
        let presets = chart.presets();
        let first = presets.first().cloned().unwrap_or_else(|| ChartData::single(vec![0.0]));
        let zero = first.zeroed();
        // Default easing: ease-in-out (entrance animations and the editor).
        let default_ease = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "ease-in-out")
            .map(|p| p.bezier())
            .unwrap_or_else(crate::bezier::Bezier::linear);
        let engine = Engine {
            chart,
            presets,
            current: zero.clone(),
            from: zero,
            target: first.clone(),
            tween: 0.0,
            grid_tween: 0.0,
            preset_idx: 0,
            editor: None,
            active_curve: default_ease,
            active_ease_label: "ease-in-out".to_string(),
            input_mode: false,
            input_buf: String::new(),
            input_err: None,
            msg: None,
            msg_until: None,
            quit: false,
        };
        engine
    }

    fn flash(&mut self, text: String) {
        self.msg = Some(text);
        self.msg_until = Some(Instant::now() + Duration::from_secs(2));
    }

    /// The current easing function — driven by the bezier editor when open,
    /// otherwise the active curve (which persists across editor sessions).
    fn ease(&self) -> impl Fn(f64) -> f64 + '_ {
        let curve = self
            .editor
            .as_ref()
            .map(|e| e.curve.clone())
            .unwrap_or_else(|| self.active_curve.clone());
        move |t| curve.sample(t)
    }

    /// Animate the axis range: the data scale eases at the main tween speed,
    /// the grid scale lags on the slower grid clock. Charts read these via
    /// `ChartData::scale` / `grid_scale` and fall back to computing their own
    /// natural range when absent.
    fn stamp_scales(&mut self) {
        let from_ax = self.chart.natural_scale(&self.from);
        let to_ax = self.chart.natural_scale(&self.target);
        let (Some(f), Some(t)) = (from_ax, to_ax) else {
            self.current.scale = None;
            self.current.grid_scale = None;
            return;
        };
        let e_data = self.ease()(self.tween as f64);
        let e_grid = self.ease()(self.grid_tween as f64);
        self.current.scale = Some(lerp_axis(f, t, e_data));
        self.current.grid_scale = Some(lerp_axis(f, t, e_grid));
    }

    fn switch_preset(&mut self, idx: usize) {
        if idx >= self.presets.len() {
            return;
        }
        let target = self.presets[idx].clone();
        if self.tween < 1.0 && self.preset_idx == idx {
            return;
        }
        self.from = self.current.clone();
        self.target = target;
        self.tween = 0.0;
        self.grid_tween = 0.0;
        self.preset_idx = idx;
        self.input_mode = false;
        self.flash(format!("Dataset {} / {}", idx + 1, self.presets.len()));
    }

    fn apply_input(&mut self, text: &str) {
        match parse_data(text) {
            Ok(mut d) => {
                d.title = self.chart.name().to_string();
                self.from = self.current.clone();
                self.target = d;
                self.tween = 0.0;
                self.grid_tween = 0.0;
                self.input_mode = false;
                self.flash("Applied custom data".to_string());
            }
            Err(e) => {
                self.input_err = Some(e);
            }
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        let mut last = Instant::now();
        loop {
            let elapsed = last.elapsed();
            last = Instant::now();

            // Advance transition using the bezier easing curve.
            if self.tween < 1.0 {
                self.tween = (self.tween + elapsed.as_secs_f32() / TWEEN_SECS).min(1.0);
                self.current = interpolate(&self.from, &self.target, self.tween as f64, self.ease());
            }
            // Grid/axis scale rides a slower clock (parallax: far layer lags data).
            if self.grid_tween < 1.0 {
                self.grid_tween = (self.grid_tween + elapsed.as_secs_f32() / GRID_TWEEN_SECS).min(1.0);
            }
            self.stamp_scales();

            // Advance the editor's live preview when it's open.
            if let Some(editor) = &mut self.editor {
                editor.tick(elapsed.as_secs_f64());
            }

            // Render one frame.
            terminal.draw(|f| {
                let area = f.area();
                if let Some(editor) = &mut self.editor {
                    editor.render(f.buffer_mut(), area);
                } else {
                    let inner = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(3));
                    self.chart.render(f.buffer_mut(), inner, &self.current);
                    self.draw_footer(f.buffer_mut(), area);
                }
            })?;

            // Event polling with a frame timeout so animation ticks even when idle.
            if event::poll(FRAME)? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(editor) = &mut self.editor {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    // Persist the editor's curve + label after exit.
                                    self.active_curve = editor.curve.clone();
                                    self.active_ease_label = editor.label();
                                    self.editor = None;
                                }
                                KeyCode::Tab => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        editor.prev_preset();
                                    } else {
                                        editor.next_preset();
                                    }
                                }
                                KeyCode::Left => editor.prev_preset(),
                                KeyCode::Right => editor.next_preset(),
                                KeyCode::Char(c) if ('1'..='3').contains(&c) => {
                                    let i = (c as u8 - b'0' - 1) as usize;
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        if editor.load_slot(i) {
                                            self.flash(format!("Loaded slot {}", i + 1));
                                        } else {
                                            self.flash(format!("Slot {} is empty", i + 1));
                                        }
                                    } else {
                                        editor.save_slot(i);
                                        self.flash(format!("Saved to slot {}", i + 1));
                                    }
                                }
                                KeyCode::Char('x') => {
                                    editor.delete_selected();
                                }
                                _ => {}
                            }
                        } else if self.input_mode {
                            self.handle_input_key(key);
                        } else {
                            self.handle_chart_key(key);
                        }
                    }
                    Event::Mouse(m) => {
                        if self.editor.is_some() {
                            if let Some(editor) = &mut self.editor {
                                editor.handle_mouse(&m);
                            }
                        } else if m.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                            // Clicking opens the bezier editor too.
                            self.open_editor();
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
            if self.quit {
                break;
            }
        }
        Ok(())
    }

    fn open_editor(&mut self) {
        // Start the editor where the active curve is: if it matches a preset,
        // re-apply that preset (so template dimming works); otherwise carry the
        // custom/slot curve over via from_curve.
        let editor = if let Some(i) = crate::easing_presets::PRESETS.iter().position(|p| p.bezier() == self.active_curve) {
            let mut e = BezierEditor::new();
            e.apply_preset(i);
            e
        } else {
            BezierEditor::from_curve(self.active_curve.clone(), &self.active_ease_label)
        };
        self.editor = Some(editor);
        self.input_mode = false;
        self.flash("Bezier editor — drag handles, [Tab] presets, [Esc] back".to_string());
    }


    fn handle_chart_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.quit = true;
            }
            KeyCode::Char('b') => {
                self.open_editor();
            }
            KeyCode::Char('p') => {
                let off = (prims::palette_offset() + 1) % prims::PALETTE.len();
                prims::set_palette_offset(off);
                self.flash(format!("Palette {} / {}", off + 1, prims::PALETTE.len()));
            }
            KeyCode::Char('i') | KeyCode::Char('d') => {
                self.input_mode = true;
                self.input_buf.clear();
                self.input_err = None;
            }
            KeyCode::Char('r') => {
                self.tween = 1.0;
                self.grid_tween = 1.0;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let n = (c as u8 - b'0') as usize;
                self.switch_preset(if n == 0 { 9 } else { n - 1 });
            }
            KeyCode::Left => {
                let n = self.presets.len();
                self.switch_preset((self.preset_idx + n - 1) % n);
            }
            KeyCode::Right => self.switch_preset((self.preset_idx + 1) % self.presets.len()),
            KeyCode::Enter => {
                let text = self.input_buf.clone();
                self.apply_input(&text);
            }
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = false;
                self.input_err = None;
            }
            KeyCode::Enter => {
                let text = self.input_buf.trim().to_string();
                if !text.is_empty() {
                    self.apply_input(&text);
                }
            }
            KeyCode::Backspace => {
                self.input_buf.pop();
                self.input_err = None;
            }
            KeyCode::Char(c) => {
                let allow = c.is_ascii_alphanumeric() || matches!(c, ',' | ';' | ' ' | '.' | '-' | '_' | ':' | '/' | '\t');
                if allow || key.modifiers.contains(KeyModifiers::SHIFT) {
                    if allow {
                        self.input_buf.push(c);
                    } else if c.is_ascii_punctuation() {
                        self.input_buf.push(c);
                    }
                }
                self.input_err = None;
            }
            _ => {}
        }
    }

    fn draw_footer(&self, buf: &mut ratatui::buffer::Buffer, area: Rect) {
        let y = area.height.saturating_sub(3);
        let status = format!(
            "{}  |  dataset {} / {}  |  ease: {}  |  [1-9] switch  [b] bezier  [p] palette  [i] input  [q] quit",
            self.chart.name(),
            self.preset_idx + 1,
            self.presets.len(),
            self.active_ease_label
        );
        prims::text(buf, area, y, &status, prims::DIM);

        if self.input_mode {
            let label = " data: ";
            let cx = area.width as usize - 1;
            let prefix = ">>";
            prims::clear_line(buf, area, y + 1);
            prims::text(buf, area, y + 1, prefix, prims::CYAN_BOLD);
            prims::text(buf, area, y + 1, label, prims::DIM);
            let x0 = prefix.len() + label.len();
            prims::abs_text(buf, area.x + x0 as u16, area.y + y + 1, &self.input_buf, prims::WHITE);
            let _ = cx;
            if let Some(err) = &self.input_err {
                prims::text(buf, area, y + 2, &format!("error: {err}"), prims::RED);
            } else {
                prims::text(
                    buf,
                    area,
                    y + 2,
                    "Enter applies, Esc cancels. e.g. 10,20,30,40  or  A: 1,2,3 / B: 4,5,6",
                    prims::DIM,
                );
            }
        } else if let Some(msg) = &self.msg {
            if self.msg_until.map(|t| Instant::now() < t).unwrap_or(false) {
                prims::text(buf, area, y + 1, msg, prims::GREEN);
            }
        }
    }
}

/// Top-level entry point: create terminal, run engine on it.
pub fn run(chart: Box<dyn Chart>) -> io::Result<()> {
    run_impl(chart, None)
}

/// Like [`run`], but start with the named easing preset as the active curve
/// and open the bezier editor immediately — used by `tuiframe preview <ease>`.
pub fn run_easing(chart: Box<dyn Chart>, preset: &str) -> io::Result<()> {
    run_impl(chart, Some(preset))
}

fn run_impl(chart: Box<dyn Chart>, preset: Option<&str>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::cursor::Hide,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut engine = Engine::new(chart);
    if let Some(name) = preset {
        if let Some(p) = crate::easing_presets::by_name(name) {
            engine.active_curve = p.bezier();
            engine.active_ease_label = p.name.to_string();
            engine.open_editor();
        }
    }
    let result = engine.run(&mut terminal);

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;
    result
}

/// Lerp two axis ranges by progress `e` in [0,1].
fn lerp_axis(f: Axis, t: Axis, e: f64) -> Axis {
    let l = |a: f64, b: f64| a + (b - a) * e;
    (l(f.0, t.0), l(f.1, t.1), l(f.2, t.2), l(f.3, t.3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_axis_tween_full_range() {
        let f: Axis = (0.0, 0.0, 0.0, 20.0);
        let t: Axis = (0.0, 5.0, 0.0, 50.0);
        let mid = lerp_axis(f, t, 0.5);
        assert_eq!(mid.1, 2.5);
        assert_eq!(mid.3, 35.0);
        assert_eq!(lerp_axis(f, t, 0.0), f);
        assert_eq!(lerp_axis(f, t, 1.0), t);
    }

    #[test]
    fn natural_scale_glides_smoothly_not_stepwise() {
        // The whole point of scale animation: a tiny data change must not
        // snap the axis (nice_max(18)=20 but nice_max(21)=50 would step).
        // Interpolating axis endpoints is continuous by construction.
        let f: Axis = (0.0, 1.0, 0.0, 20.0);
        let t: Axis = (0.0, 1.0, 0.0, 50.0);
        let a = lerp_axis(f, t, 0.499);
        let b = lerp_axis(f, t, 0.501);
        assert!((b.3 - a.3).abs() < 1.0, "axis should glide, delta was {}", b.3 - a.3);
    }

    #[test]
    fn default_ease_is_ease_in_out() {
        let e = Engine::new(crate::charts::make_chart(crate::charts::ChartKind::Area));
        assert_eq!(e.active_ease_label, "ease-in-out");
        let ease_in_out = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "ease-in-out")
            .map(|p| p.bezier())
            .unwrap();
        assert_eq!(e.active_curve, ease_in_out);
        // Opening the editor should land on the ease-in-out preset bar item.
        let mut e = e;
        e.open_editor();
        assert_eq!(e.editor.as_ref().unwrap().preset_idx, Some(3));
    }

    #[test]
    fn footer_shows_active_ease_label() {
        let mut e = Engine::new(crate::charts::make_chart(crate::charts::ChartKind::Area));
        e.active_ease_label = "ease-in-out".to_string();
        let area = ratatui::layout::Rect::new(0, 0, 120, 20);
        let buf = ratatui::buffer::Buffer::empty(area);
        let mut buf = buf;
        e.draw_footer(&mut buf, area);
        let line: String = (0..buf.area.width)
            .map(|x| buf.cell((x, area.height - 3)).map(|c| c.symbol().to_string()).unwrap_or_default())
            .collect();
        assert!(line.contains("ease: ease-in-out"), "footer was: {line:?}");
    }

    #[test]
    fn footer_shows_slot_label_for_saved_curve() {
        let mut e = Engine::new(crate::charts::make_chart(crate::charts::ChartKind::Area));
        e.active_ease_label = "2".to_string();
        let area = ratatui::layout::Rect::new(0, 0, 120, 20);
        let buf = ratatui::buffer::Buffer::empty(area);
        let mut buf = buf;
        e.draw_footer(&mut buf, area);
        let line: String = (0..buf.area.width)
            .map(|x| buf.cell((x, area.height - 3)).map(|c| c.symbol().to_string()).unwrap_or_default())
            .collect();
        assert!(line.contains("ease: 2"), "footer was: {line:?}");
    }
}
