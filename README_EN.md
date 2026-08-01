# tuiframe — Rust TUI Charts & Easing Components

<div align="center">

**English** | [简体中文](./README.md)

</div>

**tuiframe** is a file-driven catalog of reusable Rust TUI components for [ratatui](https://github.com/ratatui-org/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — **20 interactive chart components** and **17 bezier easing presets**.

Every chart is **real, compilable, interactive code** — not a placeholder example. Preview it live, read the source, or paste it into your own project.

> This project is continuously updated: new chart types, interactions, and easing presets are being added on an ongoing basis.

---

## ✨ Highlights

**Interactive Charts**
- **20 chart types**: area, box plot, bubble, bullet, candlestick, donut, funnel, gantt, heatmap, histogram, network graph, parallel coordinates, radar, sankey, scatter, stacked area, sunburst, treemap, violin, waterfall
- **Switch presets with number keys**: `1`-`9` cycles through data variants instantly, `←`/`→` for step-through
- **Eased transitions**: chart transitions use bezier easing curves with **parallax layering** (data animates faster than the grid)
- **Scale animation**: data and grid scale independently with resolution-adaptive density
- **Palette cycling**: press `p` to cycle through 10 color schemes with full gradients
- **Embedded bezier editor**: press `b` to open the curve editor, `i`/`d` to input custom values and fine-tune animation timing

**Bezier Easing Presets**
- **17 preset curves**: linear, ease-in/out/in-out, quad series, back, elastic, bounce, overshoot, soft-step and more
- **Live preview**: `tuiframe preview <easing>` shows the curve and lets you drag its control handles
- **Bilingual descriptions**: every preset ships with a name plus bilingual notes; browse them all via `tuiframe list`

**Live Preview**
- `tuiframe preview <name>` **compiles the example into a standalone binary** and runs it in a pseudo-terminal with full keyboard/mouse/resize forwarding
- The preview is the **exact same real code** you get from `tuiframe code <name>` — what you see is what you get

---
## Video

https://github.com/user-attachments/assets/6b418600-45cf-4104-a469-27b4c8876b71


https://github.com/user-attachments/assets/c4b60cb7-02b4-4e71-8a45-9100b809d46d

---
## 📦 Quick Start

```bash
git clone https://github.com/yangstafiltra/rust-tuiframe.git
cd rust-tuiframe
cargo run -- list
```

Or install the CLI:

```bash
cargo install --path /path/to/rust-tuiframe/tuiframe-cli --force
```

### Commands

| Command | Description |
|---------|-------------|
| `tuiframe-cli list` | Browse all components by category |
| `tuiframe-cli list --search chart` | Search by keyword (name/category/description) |
| `tuiframe-cli info <name>` | Show component details + example code |
| `tuiframe-cli code <name>` | Print the example code (pipeable) |
| `tuiframe-cli preview <name>` | Run a component live (`q` to quit) |
| `tuiframe-cli preview <easing>` | Preview a bezier easing curve |
| `tuiframe-cli browse` | Interactive TUI browser (j/k to navigate) |
| `tuiframe-cli validate` | Validate component dependency integrity |

---

## 📊 Component Catalog (click for details)

> Click a component name to view its full definition (`components/viz/*.toml`); the preview commands run it live.

### ■ viz — 20 charts + 3 support components

| Component | Description |
|-----------|-------------|
| [area_chart](./components/viz/area_chart.toml) | Filled area chart showing cumulative trends |
| [box_plot](./components/viz/box_plot.toml) | Box-and-whisker plot for statistical distributions |
| [bubble_chart](./components/viz/bubble_chart.toml) | Bubble chart with variable-size points for three-dimensional data |
| [bullet_chart](./components/viz/bullet_chart.toml) | Bullet chart for performance against a target |
| [candle_chart](./components/viz/candle_chart.toml) | Candlestick chart for financial OHLC data |
| [canvas](./components/viz/canvas.toml) | Draw arbitrary shapes using braille/dot characters |
| [donut_chart](./components/viz/donut_chart.toml) | Ring-shaped chart with center label |
| [funnel_chart](./components/viz/funnel_chart.toml) | Funnel chart for conversion analysis |
| [gantt_chart](./components/viz/gantt_chart.toml) | Gantt chart for project timeline visualization |
| [heatmap](./components/viz/heatmap.toml) | Color-coded heatmap matrix for data intensity visualization |
| [histogram](./components/viz/histogram.toml) | Histogram for frequency distribution visualization |
| [network_graph](./components/viz/network_graph.toml) | Node-link diagram for graph visualization |
| [node_graph](./components/viz/node_graph.toml) | Node graph visualization using ratatui Canvas |
| [parcoords](./components/viz/parcoords.toml) | Parallel coordinates plot for multi-dim data |
| [pie_chart](./components/viz/pie_chart.toml) | Pie chart with labeled segments and percentage display |
| [radar_chart](./components/viz/radar_chart.toml) | Radar/spider chart for multi-dimensional comparison |
| [sankey_diagram](./components/viz/sankey_diagram.toml) | Sankey flow diagram for energy or material flow visualization |
| [scatter_plot](./components/viz/scatter_plot.toml) | Scatter plot for point data visualization |
| [stacked_area_chart](./components/viz/stacked_area_chart.toml) | Stacked area chart for part-to-whole trends |
| [sunburst](./components/viz/sunburst.toml) | Radial hierarchy chart with concentric ring levels |
| [treemap](./components/viz/treemap.toml) | Treemap for hierarchical proportion display |
| [violin_plot](./components/viz/violin_plot.toml) | Violin plot showing data distribution density |
| [waterfall_chart](./components/viz/waterfall_chart.toml) | Waterfall chart for sequential contribution analysis |

### ■ easing — 17 bezier easing presets

> Presets are defined in [easing_presets.rs](./tuiframe-viz/src/easing_presets.rs); `tuiframe preview <name>` shows them live.

| Preset | Description |
|--------|-------------|
| [linear](./tuiframe-viz/src/easing_presets.rs) | Constant speed |
| [ease-in](./tuiframe-viz/src/easing_presets.rs) | Gradual acceleration, then settle |
| [ease-out](./tuiframe-viz/src/easing_presets.rs) | Fast start, gentle landing |
| [ease-in-out](./tuiframe-viz/src/easing_presets.rs) | Slow start and end |
| [ease-in-quad](./tuiframe-viz/src/easing_presets.rs) | Quadratic ease-in |
| [ease-out-quad](./tuiframe-viz/src/easing_presets.rs) | Quadratic ease-out |
| [ease-in-out-quad](./tuiframe-viz/src/easing_presets.rs) | Quadratic ease-in-out |
| [back](./tuiframe-viz/src/easing_presets.rs) | Overshoots past the end |
| [back-out](./tuiframe-viz/src/easing_presets.rs) | Pull back before starting |
| [anticipate](./tuiframe-viz/src/easing_presets.rs) | Anticipate then accelerate |
| [elastic](./tuiframe-viz/src/easing_presets.rs) | Springy elastic wobble |
| [bounce](./tuiframe-viz/src/easing_presets.rs) | Bouncing landing |
| [overshoot](./tuiframe-viz/src/easing_presets.rs) | Fast rise past the target |
| [undershoot](./tuiframe-viz/src/easing_presets.rs) | Falls below before rising |
| [soft-step](./tuiframe-viz/src/easing_presets.rs) | Soft S-shaped curve |
| [hard-step](./tuiframe-viz/src/easing_presets.rs) | Stiff near-instant switch |
| [whip](./tuiframe-viz/src/easing_presets.rs) | Snappy whip-like snap |

---

## 🎮 Interactive Keys

While previewing a chart:

| Key | Function |
|-----|----------|
| `1`-`9` | Switch data variants (demo different styles) |
| `←` / `→` | Cycle through variants |
| `p` | Cycle the color palette |
| `b` | Open the bezier editor |
| `i` / `d` | Input custom values |
| `r` | Reset transition animation |
| `q` / `Esc` | Quit |

---

## 🏗 Project Structure

```
tuiframe/
├── components/         # .toml component definitions (viz: 20 charts + 3 support)
├── templates/          # Project scaffolding templates
├── tuiframe-core/      # Library: deserializes component/layout TOML
├── tuiframe-viz/       # Library: interactive chart rendering + easing presets
├── tuiframe-cli/       # CLI: list / info / code / browse / preview
└── Cargo.toml          # Workspace root
```

## ✅ Component Validation

```bash
cargo run -- validate             # dependency graph integrity
./scripts/compile-check.sh        # every component example compiles into a runnable program
```

## 🤖 For AI Assistants

See [AI_SKILL.md](./AI_SKILL.md) for the component catalog skill definition.

## 📝 Roadmap

- [x] All 20 charts interactive preview
- [x] 17 bezier easing presets
- [ ] More chart types and variants
- [ ] Custom themes and color schemes
- [ ] More component categories (Layout / Navigation / Feedback…)

## 🤖 This project was generated by AI

## 📄 License

MIT
