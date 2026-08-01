use std::collections::HashSet;

use serde::Serialize;
use tuiframe_core::ComponentDef;
use tuiframe_core::ComponentRegistry;

#[derive(Serialize)]
struct JsonCategory<'a> {
    name: &'a str,
    count: usize,
    components: Vec<&'a ComponentDef>,
}

/// Easing preset entries shown by `list` as a dedicated category.
#[derive(Serialize)]
struct JsonEasing {
    name: String,
    count: usize,
    presets: Vec<String>,
}

pub fn cmd_list(search: Option<&str>, json: bool, reg: &ComponentRegistry) {
    let matching: Vec<JsonCategory> = {
        let mut map: std::collections::BTreeMap<&str, Vec<&ComponentDef>> =
            std::collections::BTreeMap::new();
        if let Some(keyword) = search {
            for (cat, comp) in reg.search(keyword) {
                map.entry(cat).or_default().push(comp);
            }
        } else {
            for cat in reg.categories() {
                if let Some(comps) = reg.components_for_category(cat) {
                    map.insert(cat, comps.iter().collect());
                }
            }
        }
        map.into_iter()
            .map(|(name, components)| JsonCategory {
                name,
                count: components.len(),
                components,
            })
            .collect()
    };

    // The easing preset catalog lives in tuiframe-viz; surface it as its own
    // category so `tuiframe list` shows the curves alongside components.
    let ease_entries: Vec<(&'static str, &'static str)> =
        tuiframe_viz::easing_presets::entries();
    let ease_match = search
        .map(|kw| {
            let kw = kw.to_lowercase();
            ease_entries
                .iter()
                .any(|(n, d)| n.contains(&kw) || d.to_lowercase().contains(&kw))
        })
        .unwrap_or(true);

    if json {
        let mut out = serde_json::to_value(&matching).unwrap_or_default();
        if ease_match {
            if let Some(arr) = out.as_array_mut() {
                arr.push(
                    serde_json::to_value(JsonEasing {
                        name: "easing".into(),
                        count: ease_entries.len(),
                        presets: ease_entries.iter().map(|(n, _)| String::from(*n)).collect(),
                    })
                    .unwrap_or_default(),
                );
            }
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return;
    }

    if matching.is_empty() && !ease_match {
        println!("\n  No matching components found.\n");
        return;
    }

    let total: usize = matching.iter().map(|c| c.count).sum();
    println!("\n  tuiframe — Component Catalog ({total} components)\n");

    for cat in &matching {
        println!("  ■  {} ({} components)", cat.name, cat.count);
        println!(" {}", "─".repeat(50));
        for comp in &cat.components {
            println!("    {:<20}  {}", comp.name, comp.description);
        }
        println!();
    }

    if ease_match {
        println!("  ■  easing ({} presets)", ease_entries.len());
        println!(" {}", "─".repeat(50));
        for (name, desc) in &ease_entries {
            println!("    {name:<20}  {desc}");
        }
        println!();
    }
}

fn generate_example_wrapper(body_lines: &[&str], extra_imports: &[&str]) -> String {
    const BODY_INDENT: &str = "            ";
    let mut body_lines = body_lines.to_vec();
    body_lines.insert(0, "let area = f.area();");
    let body = body_lines
        .iter()
        .map(|l| format!("{BODY_INDENT}{l}"))
        .collect::<Vec<_>>()
        .join("\n");
    let uses_canvas = body_lines
        .iter()
        .any(|l| l.contains("Canvas") || l.contains("Circle"));
    let canvas_import = if uses_canvas {
        "use ratatui::widgets::canvas::{Canvas, Circle};\n"
    } else {
        ""
    };
    let extra = if extra_imports.is_empty() {
        String::new()
    } else {
        let mut s = extra_imports.join("\n");
        s.push_str("\n\n");
        s
    };

    format!(
        "\
use ratatui::{{prelude::*, widgets::*}};
{canvas_import}use crossterm::event::{{self, Event, KeyCode, KeyEventKind}};
{extra}fn main() -> std::io::Result<()> {{
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    loop {{
        terminal.draw(|f| {{
{body}
        }})?;

        if let Event::Key(key) = event::read()? {{
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {{
                break;
            }}
        }}
    }}

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}}"
    )
}

pub(crate) fn wrap_example(example: &str) -> String {
    let trimmed = example.trim();
    if trimmed.starts_with("fn main") {
        return trimmed.to_string();
    }

    let mut extra_imports = Vec::new();
    let mut body_lines = Vec::new();
    for line in trimmed.lines() {
        let stripped = line.trim();
        if stripped.starts_with("use ") {
            extra_imports.push(stripped);
        } else if stripped == "let area = f.area();" {
            // Injected by the wrapper; drop duplicate declarations.
        } else if !stripped.is_empty() {
            body_lines.push(stripped);
        }
    }

    if body_lines.is_empty() {
        return trimmed.to_string();
    }

    generate_example_wrapper(&body_lines, &extra_imports)
}

pub fn cmd_info(name: &str, json: bool, reg: &ComponentRegistry) -> anyhow::Result<()> {
    let comp = reg
        .get_component(name)
        .ok_or_else(|| anyhow::anyhow!("Component '{name}' not found."))?;

    if json {
        let output = serde_json::to_string_pretty(comp).unwrap_or_default();
        println!("{output}");
        return Ok(());
    }

    let example = wrap_example(&comp.example);

    println!("\n  {}\n", comp.name);
    println!("  Category:     {}", comp.category);
    println!("  Description:  {}", comp.description);
    println!("  Dependencies: [{}]", comp.dependencies.join(", "));
    if !comp.reference_apps.is_empty() {
        println!("  Reference apps:");
        for app in &comp.reference_apps {
            println!("    • {app}");
        }
    }
    println!("  Features:");
    for feat in &comp.features {
        println!("    • {feat}");
    }
    if !comp.snippet.is_empty() {
        println!("\n  Snippet:\n");
        for line in comp.snippet.lines() {
            println!("    {line}");
        }
    }
    println!("\n  Example:\n");
    for line in example.lines() {
        println!("    {line}");
    }
    println!();
    Ok(())
}

pub fn cmd_code(
    name: &str,
    snippet_only: bool,
    with_deps: bool,
    reg: &ComponentRegistry,
) -> anyhow::Result<()> {
    let comp = reg
        .get_component(name)
        .ok_or_else(|| anyhow::anyhow!("Component '{name}' not found."))?;

    if snippet_only {
        let s = comp.snippet.trim();
        if s.is_empty() {
            anyhow::bail!("Component '{name}' has no snippet.");
        }
        println!("{s}");
        return Ok(());
    }

    if !with_deps {
        let example = comp.example.trim();
        if example.is_empty() {
            anyhow::bail!("Component '{name}' has no example code.");
        }
        println!("{}", wrap_example(&comp.example));
        return Ok(());
    }

    let mut all: Vec<&ComponentDef> = Vec::new();
    let mut seen = HashSet::new();
    let mut path = Vec::new();
    collect_chain(comp, reg, &mut seen, &mut path, &mut all)
        .map_err(|cycle| anyhow::anyhow!("{cycle}"))?;

    println!("use ratatui::{{prelude::*, widgets::*}};");
    println!("use crossterm::event::{{self, Event, KeyCode, KeyEventKind}};");
    for c in &all {
        for line in c.example.lines() {
            let s = line.trim();
            if s.starts_with("use ") {
                println!("{s}");
            }
        }
    }
    println!();
    println!("fn main() -> std::io::Result<()> {{");
    println!("    crossterm::terminal::enable_raw_mode()?;");
    println!("    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;");
    println!();
    println!("    loop {{");
    println!("        terminal.draw(|f| {{");

    for c in &all {
        for line in c.example.lines() {
            let s = line.trim();
            if !s.starts_with("use ") && !s.is_empty() {
                println!("            {s}");
            }
        }
    }

    println!("        }})?;");
    println!();
    println!("        if let Event::Key(key) = event::read()? {{");
    println!("            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {{");
    println!("                break;");
    println!("            }}");
    println!("        }}");
    println!("    }}");
    println!();
    println!("    crossterm::terminal::disable_raw_mode()?;");
    println!("    Ok(())");
    println!("}}");
    Ok(())
}

fn collect_chain<'a>(
    comp: &'a ComponentDef,
    reg: &'a ComponentRegistry,
    seen: &mut HashSet<String>,
    path: &mut Vec<String>,
    out: &mut Vec<&'a ComponentDef>,
) -> Result<(), String> {
    if path.contains(&comp.name) {
        return Err(format!(
            "circular dependency detected: {} → {}",
            path.join(" → "),
            comp.name
        ));
    }
    if !seen.insert(comp.name.clone()) {
        return Ok(());
    }
    path.push(comp.name.clone());
    for dep_name in &comp.dependencies {
        if let Some(dep) = reg.get_component(dep_name) {
            collect_chain(dep, reg, seen, path, out)?;
        }
    }
    path.pop();
    out.push(comp);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_example_self_contained() {
        let code = "fn main() { println!(\"hello\"); }";
        assert_eq!(wrap_example(code), code);
    }

    #[test]
    fn wrap_example_generates_main() {
        let code = "let x = 42;\nprintln!(\"{x}\");";
        let result = wrap_example(code);
        assert!(result.contains("fn main()"));
        assert!(result.contains("let x = 42;"));
        assert!(result.contains("println!(\"{x}\");"));
    }

    #[test]
    fn wrap_example_preserves_use_statements() {
        let code = "use std::collections::HashMap;\nlet mut m = HashMap::new();";
        let result = wrap_example(code);
        assert!(result.contains("use std::collections::HashMap;"));
        assert!(result.contains("let mut m = HashMap::new();"));
    }

    #[test]
    fn wrap_example_empty_body_returns_as_is() {
        let code = "";
        assert_eq!(wrap_example(code), "");
    }

    #[test]
    fn collect_chain_detects_cycle() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("a.toml"),
            "name = \"a\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = [\"b\"]\nfeatures = []\nexample = \"\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("b.toml"),
            "name = \"b\"\ncategory = \"c\"\ndescription = \"\"\ndependencies = [\"a\"]\nfeatures = []\nexample = \"\"\n",
        )
        .unwrap();
        let reg = ComponentRegistry::load_from_dir(dir.path()).unwrap();
        let a = reg.get_component("a").unwrap();
        let mut result = Vec::new();
        let err = collect_chain(a, &reg, &mut HashSet::new(), &mut Vec::new(), &mut result);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("circular dependency"));
    }

    #[test]
    fn generate_example_wrapper_basic() {
        let body = ["terminal.draw(|f| { f.render_widget(block, f.area()); });"];
        let result = generate_example_wrapper(&body, &[]);
        assert!(result.starts_with("use ratatui"));
        assert!(result.contains("enable_raw_mode"));
        assert!(result.contains("terminal.draw(|f| {"));
        assert!(result.contains("disable_raw_mode"));
    }

    #[test]
    fn generate_example_wrapper_with_imports() {
        let body = ["f.render_widget(block, f.area());"];
        let imports = ["use ratatui::widgets::Block;"];
        let result = generate_example_wrapper(&body, &imports);
        assert!(result.contains("use ratatui::widgets::Block;"));
    }
}
