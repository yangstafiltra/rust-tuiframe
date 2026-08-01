use ratatui::{prelude::*, widgets::*};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let block = Block::default()
                .title(" {{project_name}} ")
                .borders(Borders::ALL);
            let inner = block.inner(area);
            f.render_widget(block, area);

            let text = Paragraph::new("Press 'q' to quit")
                .alignment(Alignment::Center);
            f.render_widget(text, inner);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
