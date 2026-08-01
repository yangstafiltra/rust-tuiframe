use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, prelude::*, widgets::*};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tuiframe_core::ComponentRegistry;

struct TermGuard {
    clean_exit: bool,
}

impl TermGuard {
    fn cleanup(&self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stderr(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        if !self.clean_exit {
            self.cleanup();
        }
    }
}

pub fn browse(reg: &ComponentRegistry) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(
        stderr,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr))?;
    terminal.clear()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let mut app = BrowseApp::new(reg);

    while !app.quit && running.load(Ordering::SeqCst) {
        terminal.draw(|f| app.draw(f))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) => app.handle_key(key),
            Event::Mouse(m) => app.handle_mouse(m),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    drop(terminal);
    let _guard = TermGuard { clean_exit: true };
    _guard.cleanup();
    Ok(())
}

struct BrowseApp<'a> {
    reg: &'a ComponentRegistry,
    cat_idx: usize,
    comp_idx: usize,
    comp_scroll: usize,
    quit: bool,
    comp_list_height: usize,
}

impl BrowseApp<'_> {
    fn new(reg: &ComponentRegistry) -> BrowseApp<'_> {
        BrowseApp {
            reg,
            cat_idx: 0,
            comp_idx: 0,
            comp_scroll: 0,
            quit: false,
            comp_list_height: 14,
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();

        if self.reg.is_empty() {
            let para =
                Paragraph::new("No components found.\n\nSet TUIFRAME_DIR to the project root.")
                    .block(Block::default().borders(Borders::ALL).title(" tuiframe "))
                    .alignment(Alignment::Center);
            f.render_widget(para, area);
            return;
        }

        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        let (main_area, footer_area) = (vert[0], vert[1]);

        let footer = Paragraph::new(" j/k ↓↑  navigate  |  h/l ←→  category  |  q  quit ")
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .bg(Color::Rgb(20, 20, 40)),
            )
            .alignment(Alignment::Center);
        f.render_widget(footer, footer_area);

        let wide = main_area.width > 80;
        let constraints = if wide {
            vec![
                Constraint::Length(28),
                Constraint::Length(30),
                Constraint::Min(20),
            ]
        } else {
            vec![Constraint::Length(28), Constraint::Min(20)]
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(main_area);

        let cat_items: Vec<ListItem> = self
            .reg
            .categories()
            .iter()
            .enumerate()
            .map(|(i, cat)| {
                let prefix = if i == self.cat_idx { "▸" } else { " " };
                let count = self.reg.components_for_category(cat).map_or(0, |c| c.len());
                ListItem::new(format!("{prefix} {cat} ({count})"))
            })
            .collect();
        let cat_list = List::new(cat_items)
            .block(Block::default().borders(Borders::ALL).title(" Categories "))
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 80)));
        let mut cat_state = ListState::default().with_selected(Some(self.cat_idx));
        f.render_stateful_widget(cat_list, chunks[0], &mut cat_state);

        let Some(comps) = self.reg.get_components_in_category(self.cat_idx) else {
            return;
        };
        if comps.is_empty() {
            return;
        }

        let idx = self.comp_idx.min(comps.len().saturating_sub(1));
        self.comp_list_height = (chunks[1].height as usize).saturating_sub(2);

        let comp_items: Vec<ListItem> = comps
            .iter()
            .map(|c| ListItem::new(c.name.to_string()))
            .collect();
        let comp_list = List::new(comp_items)
            .block(Block::default().borders(Borders::ALL).title(" Components "))
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 80)));
        let mut comp_state = ListState::default()
            .with_selected(Some(idx))
            .with_offset(self.comp_scroll);
        f.render_stateful_widget(comp_list, chunks[1], &mut comp_state);

        if wide {
            if let Some(chunk) = chunks.get(2) {
                let detail = self.format_detail(comps, idx);
                let para = Paragraph::new(detail)
                    .block(Block::default().borders(Borders::ALL).title(" Detail "))
                    .wrap(Wrap { trim: false });
                f.render_widget(para, *chunk);
            }
        }
    }

    fn format_detail(&self, comps: &[tuiframe_core::ComponentDef], idx: usize) -> String {
        let comp = &comps[idx];
        let example_preview: String = comp.example.lines().take(8).collect::<Vec<_>>().join("\n");
        format!(
            "{name} | {cat} | deps: [{deps}]\n\nfeatures:\n  • {feats}\n\n{example}",
            name = comp.name,
            cat = comp.category,
            deps = comp.dependencies.join(", "),
            feats = comp.features.join("\n  • "),
            example = example_preview,
        )
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.reg.is_empty() || key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(comps) = self.reg.get_components_in_category(self.cat_idx)
                    && !comps.is_empty()
                {
                    let max = comps.len() - 1;
                    if self.comp_idx < max {
                        self.comp_idx += 1;
                        self.adjust_scroll();
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.comp_idx > 0 {
                    self.comp_idx -= 1;
                    self.adjust_scroll();
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let next = self.cat_idx + 1;
                if next < self.reg.category_count() {
                    self.cat_idx = next;
                    self.comp_idx = 0;
                    self.comp_scroll = 0;
                }
            }
            KeyCode::Char('h') | KeyCode::Left if self.cat_idx > 0 => {
                self.cat_idx -= 1;
                self.comp_idx = 0;
                self.comp_scroll = 0;
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, m: crossterm::event::MouseEvent) {
        match m.kind {
            MouseEventKind::ScrollDown => {
                if let Some(comps) = self.reg.get_components_in_category(self.cat_idx)
                    && !comps.is_empty()
                {
                    let max = comps.len() - 1;
                    if self.comp_idx < max {
                        self.comp_idx += 1;
                        self.adjust_scroll();
                    }
                }
            }
            MouseEventKind::ScrollUp if self.comp_idx > 0 => {
                self.comp_idx -= 1;
                self.adjust_scroll();
            }
            _ => {}
        }
    }

    fn adjust_scroll(&mut self) {
        let Some(comps) = self.reg.get_components_in_category(self.cat_idx) else {
            return;
        };
        if comps.is_empty() {
            return;
        }
        let h = self.comp_list_height.max(1);
        if self.comp_idx >= self.comp_scroll + h {
            self.comp_scroll = self.comp_idx.saturating_sub(h) + 1;
        }
        if self.comp_idx < self.comp_scroll {
            self.comp_scroll = self.comp_idx;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_registry() -> ComponentRegistry {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("tuiframe_browse_test_{n}"));
        let _ = std::fs::create_dir_all(&dir);

        std::fs::write(
            dir.join("block.toml"),
            r#"name = "block"
category = "core"
description = "A bordered container"
dependencies = []
features = []
example = "fn main() {}"
snippet = "Block::default().borders(Borders::ALL)"
"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("paragraph.toml"),
            r#"name = "paragraph"
category = "core"
description = "A block of styled text"
dependencies = ["block"]
features = []
example = "fn main() {}"
snippet = "Paragraph::new(\"hello\")"
"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("table.toml"),
            r#"name = "table"
category = "data"
description = "A data table"
dependencies = []
features = ["sort", "filter"]
example = "fn main() {}"
snippet = "Table::new(rows)"
"#,
        )
        .unwrap();

        ComponentRegistry::load_from_dir(&dir).unwrap()
    }

    fn press(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent {
            code,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn test_browse_app_new() {
        let reg = test_registry();
        let app = BrowseApp::new(&reg);
        assert!(!app.quit);
        assert_eq!(app.cat_idx, 0);
        assert_eq!(app.comp_idx, 0);
    }

    #[test]
    fn test_browse_app_key_navigation() {
        let reg = test_registry();
        let mut app = BrowseApp::new(&reg);

        app.handle_key(press(KeyCode::Char('j')));
        assert_eq!(app.comp_idx, 1);

        app.handle_key(press(KeyCode::Char('k')));
        assert_eq!(app.comp_idx, 0);

        app.handle_key(press(KeyCode::Char('l')));
        assert_eq!(app.cat_idx, 1);
        assert_eq!(app.comp_idx, 0);

        app.handle_key(press(KeyCode::Char('h')));
        assert_eq!(app.cat_idx, 0);

        app.handle_key(press(KeyCode::Char('q')));
        assert!(app.quit);
    }

    #[test]
    fn test_browse_app_key_bounds() {
        let reg = test_registry();
        let mut app = BrowseApp::new(&reg);

        app.handle_key(press(KeyCode::Char('k')));
        assert_eq!(app.comp_idx, 0);

        for _ in 0..100 {
            app.handle_key(press(KeyCode::Char('j')));
        }
        assert_eq!(app.comp_idx, 1);

        for _ in 0..100 {
            app.handle_key(press(KeyCode::Right));
        }
        assert_eq!(app.cat_idx, 1);
    }

    #[test]
    fn test_browse_mouse_scroll() {
        let reg = test_registry();
        let mut app = BrowseApp::new(&reg);

        app.handle_mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        });
        assert_eq!(app.comp_idx, 1);

        app.handle_mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        });
        assert_eq!(app.comp_idx, 0);
    }

    #[test]
    fn format_detail_basic() {
        let reg = test_registry();
        let app = BrowseApp::new(&reg);
        let comps = reg.components_for_category("core").unwrap();
        let detail = app.format_detail(comps, 0);
        assert!(detail.contains("block"));
        assert!(detail.contains("deps:"));
        assert!(detail.contains("features:"));
    }
}
