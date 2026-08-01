use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    event::DisableMouseCapture,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tuiframe_core::ComponentRegistry;

use crate::commands;

/// Relative path from `<root>/target/preview/<name>/Cargo.toml` back to a
/// workspace member like `tuiframe-viz` so it can be used as a path dependency.
fn pathdiff_relative(member: &str) -> String {
    // <root>/target/preview/<name>/Cargo.toml -> <root>/<member>
    // 3 levels up: <name>/preview/target
    format!("../../../{member}")
}

/// Compile a component's example into a standalone binary under `target/preview/<name>`.
/// Subsequent previews of the same component are cheap thanks to cargo's cache.
fn build_preview_binary(name: &str, code: &str) -> Result<PathBuf> {
    let root = crate::util::project_root();
    let dir = root.join("target").join("preview").join(name);
    std::fs::create_dir_all(dir.join("src"))
        .with_context(|| format!("failed to create preview dir for '{name}'"))?;
    std::fs::write(dir.join("src").join("main.rs"), code)
        .with_context(|| format!("failed to write preview source for '{name}'"))?;

    let rel_viz = pathdiff_relative("tuiframe-viz");
    let manifest = format!("[workspace]

[package]
name = \"preview\"
version = \"0.1.0\"
edition = \"2024\"

[dependencies]
ratatui = \"0.30\"
crossterm = \"0.29\"
tuiframe-viz = {{ path = \"{rel_viz}\" }}
");
    std::fs::write(dir.join("Cargo.toml"), manifest)
        .with_context(|| format!("failed to write preview manifest for '{name}'"))?;

    eprintln!("  Building preview for '{name}' (first build may take a while)...");
    let status = std::process::Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .current_dir(&dir)
        .status()
        .context("failed to run cargo build for preview")?;
    if !status.success() {
        anyhow::bail!("preview build failed for '{name}' — the example code does not compile");
    }
    Ok(dir.join("target").join("release").join("preview"))
}

/// Source for a preview binary that runs a utility widget interactively.
fn widget_preview_code(name: &str) -> Result<String> {
    Ok(format!(
        r#"fn main() -> std::io::Result<()> {{
    match tuiframe_viz::preview_widget("{name}")? {{
        true => Ok(()),
        false => {{ eprintln!("widget not found: {name}"); std::process::exit(1) }}
    }}
}}"#
    ))
}

/// Source for a preview binary that opens the engine with the named easing
/// preset active and the bezier editor already open.
fn easing_preview_code(name: &str) -> Result<String> {
    Ok(format!(
        r#"fn main() -> std::io::Result<()> {{
    tuiframe_viz::preview_easing("{name}")
}}"#
    ))
}

fn current_pty_size() -> PtySize {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Restore the local terminal fully. The child pty may have left the
        // local terminal in mouse-capture mode (via forwarded escape codes),
        // so explicitly disable it here — otherwise later mouse movement is
        // echoed to the screen as raw `\x1b[<0;x;yM` sequences.
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Run a component's compiled program inside a pty, forwarding the local
/// terminal's input and the pty's output in both directions. This makes the
/// preview fully interactive: keys, mouse, and resizes pass through.
pub fn preview(name: &str, reg: &ComponentRegistry) -> Result<()> {
    let code = if let Some(comp) = reg.get_component(name) {
        if comp.example.trim().is_empty() {
            anyhow::bail!("Component '{name}' has no example code to preview.");
        }
        commands::wrap_example(&comp.example)
    } else if tuiframe_viz::easing_presets::by_name(name).is_some() {
        easing_preview_code(name)?
    } else if tuiframe_viz::widgets::make(name).is_some() {
        widget_preview_code(name)?
    } else {
        anyhow::bail!("Component '{name}' not found.");
    };

    let binary = build_preview_binary(name, &code)?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(current_pty_size())
        .context("failed to open pseudo-terminal")?;

    let cmd = CommandBuilder::new(binary);
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("failed to spawn preview process")?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .context("failed to clone pty reader")?;
    let mut writer = pair
        .master
        .take_writer()
        .context("failed to take pty writer")?;

    // pty -> local stdout: forward the child's rendered output
    std::thread::spawn(move || {
        let mut out = std::io::stdout();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if out.write_all(&buf[..n]).and_then(|_| out.flush()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // local stdin -> pty: forward raw bytes (keys, mouse, ctrl+c) untouched
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        loop {
            match input.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if writer.write_all(&buf[..n]).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // main loop: keep the pty sized to the local terminal, wait for the child to exit.
    // The child quits on 'q' (forwarded from the local terminal), so previewing
    // ends when the child's own event loop does.
    let mut size = current_pty_size();
    while child.try_wait()?.is_none() {
        let new_size = current_pty_size();
        if new_size != size {
            let _ = pair.master.resize(new_size);
            size = new_size;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
