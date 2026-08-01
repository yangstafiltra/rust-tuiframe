use ratatui::{prelude::*, widgets::*};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

struct FormApp {
    name: String,
    email: String,
    focus: usize,
    submitted: bool,
    quit: bool,
}

impl FormApp {
    fn new() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            focus: 0,
            submitted: false,
            quit: false,
        }
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.area();
        let block = Block::default()
            .title(" {{project_name}} ")
            .borders(Borders::ALL);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(area);

        let name_style = if self.focus == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let name_input = Paragraph::new(self.name.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Name ")
                    .border_style(name_style),
            );
        f.render_widget(name_input, chunks[0]);

        let email_style = if self.focus == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let email_input = Paragraph::new(self.email.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Email ")
                    .border_style(email_style),
            );
        f.render_widget(email_input, chunks[1]);

        let submit_style = if self.focus == 2 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let submit_btn = Paragraph::new("[ Submit ]")
            .alignment(Alignment::Center)
            .style(submit_style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(submit_btn, chunks[2]);

        if self.submitted {
            let msg = format!("Hello, {} <{}>", self.name, self.email);
            let result = Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green));
            f.render_widget(result, chunks[3]);
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => self.focus = (self.focus + 1) % 3,
            KeyCode::BackTab => self.focus = self.focus.saturating_sub(1),
            KeyCode::Enter if self.focus == 2 => {
                if !self.name.is_empty() && !self.email.is_empty() {
                    self.submitted = true;
                }
            }
            KeyCode::Char(c) => match self.focus {
                0 => self.name.push(c),
                1 => self.email.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                0 => { self.name.pop(); }
                1 => { self.email.pop(); }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    terminal.clear()?;

    let mut app = FormApp::new();

    while !app.quit {
        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => app.quit = true,
                    k => app.handle_key(k),
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
