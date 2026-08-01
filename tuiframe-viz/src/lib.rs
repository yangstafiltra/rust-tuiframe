pub mod bezier;
pub mod bezier_editor;
pub mod charts;
pub mod data;
pub mod easing_presets;
pub mod engine;
pub mod prims;
pub mod widgets;

pub use charts::{ChartKind, make_chart};
pub use data::ChartData;
pub use engine::{Chart, Engine, run};
pub use widgets::Widget;

/// Convenience: run a chart by kind with the default interactive engine.
pub fn preview(kind: ChartKind) -> std::io::Result<()> {
    engine::run(make_chart(kind))
}

/// Run the engine starting with the named easing preset as the active curve,
/// opening the bezier editor immediately.
pub fn preview_easing(name: &str) -> std::io::Result<()> {
    engine::run_easing(make_chart(ChartKind::Area), name)
}

/// Run an interactive utility widget by component name. Returns `None` (as
/// an error-free `Ok(false)` distinction is awkward here, so we use an
/// `Option`-style contract: unknown names are simply not found).
pub fn preview_widget(name: &str) -> std::io::Result<bool> {
    match widgets::make(name) {
        Some(w) => {
            widgets::run_widget(w)?;
            Ok(true)
        }
        None => Ok(false),
    }
}
