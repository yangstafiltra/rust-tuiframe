mod browse;
mod commands;
mod preview;
mod util;

use clap::{Parser, Subcommand};
use std::path::Path;
use tuiframe_core::ComponentRegistry;

#[derive(Parser)]
#[command(
    name = "tuiframe",
    version,
    about = "Rust TUI component catalog & scaffolding tool"
)]
#[command(long_about = concat!(
    "tuiframe — file-driven Rust TUI component catalog.\n\n",
    "Browse, search, and copy component examples. ",
    "Scaffold new ratatui projects from templates.\n\n",
    "EXAMPLES:\n",
    "  tuiframe list                  List all components\n",
    "  tuiframe list --search chart   Search components by keyword\n",
    "  tuiframe info block            Show component details\n",
    "  tuiframe info block --json     Show component as JSON\n",
    "  tuiframe code block            Print example code (pipeable)\n",
    "  tuiframe code block >> main.rs Append example to a file\n",
    "  tuiframe code block --snippet  Print just the widget snippet\n",
    "  tuiframe code block --with-deps  Print component + its dependencies\n",
    "  tuiframe browse                Interactive TUI browser (j/k/h/l/q)\n",
    "  tuiframe preview block         Run a component live in an interactive preview\n",
    "  tuiframe scaffold mini-app my-app  Create a new project\n",
    "  tuiframe validate                 Check component dependencies\n",
))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long)]
        search: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Info {
        name: String,
        #[arg(long)]
        json: bool,
    },
    Code {
        name: String,
        #[arg(long)]
        snippet: bool,
        #[arg(long)]
        with_deps: bool,
    },
    Browse,
    Preview {
        name: String,
    },
    Validate,
    Scaffold {
        template: String,
        name: String,
    },
}

fn cmd_validate(reg: &ComponentRegistry) -> anyhow::Result<()> {
    let issues = reg.validate_dependencies();
    if issues.is_empty() {
        println!("All component dependencies are valid.");
        return Ok(());
    }
    for (comp, missing) in &issues {
        println!(
            "  [warn] {} depends on missing components: {}",
            comp,
            missing.join(", ")
        );
    }
    println!(
        "\n  {count} component(s) have missing dependencies.",
        count = issues.len()
    );
    Ok(())
}

const NAME_PLACEHOLDER: &str = "{{project_name}}";

fn copy_template(src: &Path, dst: &Path, project_name: &str) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_template(&entry.path(), &dst_path, project_name)?;
        } else if ty.is_file() {
            let content = std::fs::read_to_string(entry.path())?;
            let rendered = content.replace(NAME_PLACEHOLDER, project_name);
            std::fs::write(dst_path, rendered)?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reg = util::load_registry()?;

    match cli.command {
        None => commands::cmd_list(None, false, &reg),
        Some(Commands::List { search, json }) => commands::cmd_list(search.as_deref(), json, &reg),
        Some(Commands::Info { name, json }) => commands::cmd_info(&name, json, &reg)?,
        Some(Commands::Code {
            name,
            snippet,
            with_deps,
        }) => {
            commands::cmd_code(&name, snippet, with_deps, &reg)?;
        }
        Some(Commands::Browse) => browse::browse(&reg)?,
        Some(Commands::Preview { name }) => preview::preview(&name, &reg)?,
        Some(Commands::Validate) => cmd_validate(&reg)?,
        Some(Commands::Scaffold { template, name }) => {
            if !util::is_safe_name(&template) {
                anyhow::bail!(
                    "Invalid template name '{template}': use only letters, digits, hyphens, and underscores"
                );
            }
            if !util::is_safe_name(&name) {
                anyhow::bail!(
                    "Invalid project name '{name}': use only letters, digits, hyphens, and underscores"
                );
            }
            let root = util::project_root();
            let tmpl_dir = root.join("templates").join(&template);
            let dst = Path::new(&name);

            if dst.exists() {
                anyhow::bail!("Directory '{name}' already exists");
            }

            if tmpl_dir.exists() {
                copy_template(&tmpl_dir, dst, &name)?;
                println!("  ✓ Created project '{name}' from template '{template}'");
            } else {
                std::fs::create_dir_all(dst.join("src"))?;
                let main_rs = format!(
                    r##"use ratatui::{{prelude::*, widgets::*}};
use crossterm::event;

fn main() -> std::io::Result<()> {{
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    terminal.clear()?;
    loop {{
        terminal.draw(|f| {{
            let block = Block::default().title(" {name} ").borders(Borders::ALL);
            f.render_widget(block, f.area());
        }})?;
        if let event::Event::Key(key) = event::read()? {{
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {{
                break;
            }}
        }}
    }}
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}}
"##
                );
                std::fs::write(dst.join("src/main.rs"), main_rs)?;
                let cargo_toml = format!(
                    r##"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"
"##
                );
                std::fs::write(dst.join("Cargo.toml"), cargo_toml)?;
                println!("  ✓ Created project '{name}'");
            }
            println!("  cd {name} && cargo run");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_template_placeholder_substitution() {
        let src = std::env::temp_dir().join("tuiframe_tmpl_src");
        let dst = std::env::temp_dir().join("tuiframe_tmpl_dst");
        let _ = std::fs::create_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);

        std::fs::write(
            src.join("main.rs"),
            format!("fn main() {{ let name = \"{NAME_PLACEHOLDER}\"; }}"),
        )
        .unwrap();

        copy_template(&src, &dst, "my_app").unwrap();
        let content = std::fs::read_to_string(dst.join("main.rs")).unwrap();
        assert_eq!(content, "fn main() { let name = \"my_app\"; }");
        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }
}
