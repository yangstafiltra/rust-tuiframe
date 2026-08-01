use ratatui::{prelude::*, widgets::*};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

struct Dashboard {
    cpu: u16,
    mem: u16,
    start: Instant,
    tab: usize,
    quit: bool,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            cpu: 0,
            mem: 0,
            start: Instant::now(),
            tab: 0,
            quit: false,
        }
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.area();

        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Block::default()
            .title(" {{project_name}} Dashboard ")
            .borders(Borders::ALL);
        f.render_widget(title, vert[0]);

        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(vert[1]);

        let sidebar = Block::default()
            .title(" Menu ")
            .borders(Borders::ALL);
        let menu_items = ["CPU & Memory", "Network", "Disk", "Processes"]
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == self.tab { "▸" } else { " " };
                ListItem::new(format!("{prefix} {item}"))
            })
            .collect::<Vec<_>>();
        let menu = List::new(menu_items)
            .block(sidebar)
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 80)));
        f.render_widget(menu, horiz[0]);

        let main_block = Block::default()
            .title(" Content ")
            .borders(Borders::ALL);
        let main_inner = main_block.inner(horiz[1]);
        f.render_widget(main_block, horiz[1]);

        let gauge_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Length(5)])
            .margin(1)
            .split(main_inner);

        let cpu_gauge = Gauge::default()
            .block(Block::default().title(" CPU "))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(self.cpu);
        f.render_widget(cpu_gauge, gauge_area[0]);

        let mem_gauge = Gauge::default()
            .block(Block::default().title(" Memory "))
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(self.mem);
        f.render_widget(mem_gauge, gauge_area[1]);

        let footer = Block::default()
            .title(format!(" Uptime: {}s ", self.start.elapsed().as_secs()))
            .borders(Borders::ALL);
        f.render_widget(footer, vert[2]);
    }

    fn tick(&mut self) {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        self.cpu = ((t as f64 / 100.0).sin().abs() * 100.0) as u16;
        self.mem = ((t as f64 / 150.0).cos().abs() * 100.0) as u16;
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.tab = (self.tab + 1).min(3);
            }
            KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                self.tab = self.tab.saturating_sub(1);
            }
            _ => {}
        }
    }
}

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    terminal.clear()?;

    let mut app = Dashboard::new();

    while !app.quit {
        terminal.draw(|f| app.draw(f))?;
        app.tick();

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
