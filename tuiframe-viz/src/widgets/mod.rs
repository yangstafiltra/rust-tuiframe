//! Interactive UI widgets — the `utility` component catalog turned into
//! live previewable demos. Unlike `Chart` (data-driven, animated), these are
//! classic UI panels/controls. Each widget ships several style variants that
//! you flip through with the `1-9` number keys, mirroring chart presets.

pub mod overlay;
pub mod progress;
pub mod scroll;
pub mod status;
pub mod data;
pub mod system;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;

use crate::prims;

/// A UI widget with several style variants. The engine renders `variant`
/// (0-based index) of `variants()`; number keys and arrow keys cycle it.
pub trait Widget {
    fn name(&self) -> &'static str;
    /// Display names of every style variant (at least one).
    fn variants(&self) -> Vec<&'static str>;
    /// Draw one frame. `variant` is the selected index, `tick` an ever-growing
    /// frame counter so animated widgets (spinners, progress) can advance.
    fn render(&mut self, buf: &mut Buffer, area: Rect, variant: usize, tick: u64);
}

const FRAME: Duration = Duration::from_millis(33); // ~30 fps is plenty for UI

/// Interactive preview loop: number keys / arrows switch style variants.
pub fn run_widget(mut widget: Box<dyn Widget>) -> io::Result<()> {
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

    let n = widget.variants().len().max(1);
    let mut variant = 0usize;
    let mut tick: u64 = 0;
    let result = loop {
        tick = tick.wrapping_add(1);

        terminal.draw(|f| {
            let area = f.area();
            widget.render(f.buffer_mut(), area, variant, tick);
            let y = area.height.saturating_sub(1);
            let footer = format!(
                "{}  |  variant {} / {}  |  [1-9] switch  [←/→] cycle  [q] quit",
                widget.name(),
                variant + 1,
                n,
            );
            prims::text(f.buffer_mut(), area, y, &footer, prims::DIM);
        })?;

        let mut quit = false;
        if event::poll(FRAME)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => quit = true,
                    KeyCode::Left => {
                        variant = (variant + n - 1) % n;
                    }
                    KeyCode::Right | KeyCode::Char('\t') => {
                        variant = (variant + 1) % n;
                    }
                    KeyCode::Char('0') => variant = 9.min(n - 1),
                    KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                        let idx = c as u8 - b'1';
                        variant = (idx as usize).min(n - 1);
                    }
                    KeyCode::Char('p') => {
                        let off = (prims::palette_offset() + 1) % prims::PALETTE.len();
                        prims::set_palette_offset(off);
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        if quit {
            break Ok(());
        }
    };

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;
    result
}

/// Construct a widget by its component name. Returns `None` for unknown names.
pub fn make(name: &str) -> Option<Box<dyn Widget>> {
    let w: Box<dyn Widget> = match name {
        "popup" => Box::new(overlay::Popup),
        "dialog" => Box::new(overlay::Dialog),
        "context_menu" => Box::new(overlay::ContextMenu),
        "floating_palette" => Box::new(overlay::FloatingPalette),
        "spotlight" => Box::new(overlay::Spotlight),
        "onboarding_tip" => Box::new(overlay::OnboardingTip),
        "tutorial_step" => Box::new(overlay::TutorialStep),
        "theme_picker" => Box::new(overlay::ThemePicker),
        "status_bar" => Box::new(status::StatusBar),
        "breadcrumb_bar" => Box::new(status::BreadcrumbBar),
        "hotkey_footer" => Box::new(status::HotkeyFooter),
        "key_binding" => Box::new(status::KeyBinding),
        "loading_screen" => Box::new(progress::LoadingScreen),
        "auto_updater" => Box::new(progress::AutoUpdater),
        "scrollbar" => Box::new(scroll::Scrollbar),
        "minimap_scroll" => Box::new(scroll::MinimapScroll),
        "resize_handle" => Box::new(scroll::ResizeHandle),
        "zoom_control" => Box::new(scroll::ZoomControl),
        "measurement_tool" => Box::new(scroll::MeasurementTool),
        "history" => Box::new(data::History),
        "clipboard_view" => Box::new(data::ClipboardView),
        "command_output" => Box::new(data::CommandOutput),
        "inspector" => Box::new(data::Inspector),
        "animated_text" => Box::new(data::AnimatedText),
        "empty_state" => Box::new(data::EmptyState),
        "accessibility_helper" => Box::new(system::AccessibilityHelper),
        "error_boundary" => Box::new(system::ErrorBoundary),
        "exception_handler" => Box::new(system::ExceptionHandler),
        "screenshot_mode" => Box::new(system::ScreenshotMode),
        "welcome_screen" => Box::new(system::WelcomeScreen),
        "migration_wizard" => Box::new(system::MigrationWizard),
        _ => return None,
    };
    Some(w)
}

/// Every widget name, for `tuiframe list` / preview resolution.
pub const ALL: [&str; 31] = [
    "popup", "dialog", "context_menu", "floating_palette", "spotlight",
    "onboarding_tip", "tutorial_step", "theme_picker", "status_bar",
    "breadcrumb_bar", "hotkey_footer", "key_binding", "loading_screen",
    "auto_updater", "scrollbar", "minimap_scroll", "resize_handle",
    "zoom_control", "measurement_tool", "history", "clipboard_view",
    "command_output", "inspector", "animated_text", "empty_state",
    "accessibility_helper", "error_boundary", "exception_handler",
    "screenshot_mode", "welcome_screen", "migration_wizard",
];

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn every_widget_renders_every_variant_without_panic() {
        for name in ALL {
            let mut w = make(name).expect(name);
            let n = w.variants().len();
            assert!(n >= 1, "{name}: at least one variant");
            for v in 0..n {
                for tick in [0u64, 7] {
                    let mut terminal = Terminal::new(TestBackend::new(96, 30)).unwrap();
                    let r = terminal.draw(|f| {
                        let area = f.area();
                        w.render(f.buffer_mut(), area, v, tick);
                    });
                    assert!(r.is_ok(), "{name} variant {v} failed");
                }
            }
        }
    }
}
