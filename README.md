# tuiframe — Rust TUI 图表与缓动组件库

<div align="center">

[English](./README_EN.md) | **简体中文**

</div>

**tuiframe** 是一个基于文件驱动的 Rust TUI 组件目录，为 [ratatui](https://github.com/ratatui-org/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) 提供 **20 个可交互图表组件** 与 **17 个贝塞尔缓动预设**。

每个图表都是**真实可编译、可交互运行**的代码，而非占位示例 —— 可直接运行预览、查看源码、或复制进你的项目。

> 本项目会持续更新：持续补充新的图表类型、交互特性与缓动预设。

---

## ✨ 功能亮点

**交互式图表 (Interactive Charts)**
- **20 种图表类型**：面积图、箱线图、气泡图、子弹图、K 线图、环形图、漏斗图、甘特图、热力图、直方图、网络图、平行坐标、雷达图、桑基图、散点图、堆叠面积图、旭日图、矩形树图、小提琴图、瀑布图
- **数字键切换演示**：`1`-`9` 一键在多种数据变体间切换，`←`/`→` 循环浏览
- **缓动动画**：图表过渡采用贝塞尔缓动曲线，配合**视差分层**（数据动得快、网格动得慢）
- **比例尺动画**：数据与网格双比例尺独立过渡，分辨率越高数据密度越自适应
- **调色板切换**：`p` 键循环切换 10 套配色，渐变色全覆盖
- **内嵌贝塞尔编辑器**：`b` 键打开曲线编辑器，`i`/`d` 输入自定义值，实时调节动画节奏

**贝塞尔缓动预设 (Bezier Easing)**
- **17 个预设曲线**：linear、ease-in/out/in-out、quad 系列、back、elastic、bounce、overshoot、soft-step 等
- **实时预览**：`tuiframe preview <easing>` 直接查看曲线形态并拖动控制柄调节
- **中英双语描述**：每个预设均带名称 + 双语说明，`tuiframe list` 可一键浏览

**真实预览 (Live Preview)**
- `tuiframe preview <name>` 将组件示例**编译成独立二进制**，在伪终端中运行，键盘/鼠标/窗口缩放全量转发
- 预览的就是 `tuiframe code <name>` 拿到的**真实代码**，所见即所得

---


##视频
https://github.com/user-attachments/assets/8d57c717-cfb0-4d3e-b428-86564f4179f3

https://github.com/user-attachments/assets/e0b63028-e7aa-4993-97e8-600933794f42

---
## 📦 快速开始 (Quick Start)

```bash
git clone https://github.com/yangstafiltra/rust-tuiframe.git
cd rust-tuiframe
cargo run -- list
```

或安装 CLI：

```bash
cargo install --path /path/to/rust-tuiframe/tuiframe-cli --force
```

### 常用命令 (Commands)

| 命令 | 说明 (Description) |
|------|--------------------|
| `tuiframe-cli list` | 按分类浏览全部组件 |
| `tuiframe-cli list --search chart` | 关键字搜索（名称/分类/描述） |
| `tuiframe-cli info <name>` | 查看组件详情 + 示例代码 |
| `tuiframe-cli code <name>` | 输出示例代码（可管道输出） |
| `tuiframe-cli preview <name>` | 实时运行组件（`q` 退出） |
| `tuiframe-cli preview <easing>` | 预览贝塞尔缓动曲线 |
| `tuiframe-cli browse` | 交互式 TUI 浏览器（j/k 导航） |
| `tuiframe-cli validate` | 校验组件依赖完整性 |

---

## 📊 组件目录 (点击查看详情)

> 点击组件名可查看该组件的详细定义（`components/viz/*.toml`），点击演示命令可直接运行。

### ■ viz — 20 个图表 + 3 个支撑组件

| 组件 | 说明 |
|------|------|
| [area_chart](./components/viz/area_chart.toml) | 填充面积图，显示累积趋势 / Filled area chart showing cumulative trends |
| [box_plot](./components/viz/box_plot.toml) | 箱线图，用于统计分布 / Box-and-whisker plot for statistical distributions |
| [bubble_chart](./components/viz/bubble_chart.toml) | 气泡图，含可变大小点，用于三维数据 / Bubble chart with variable-size points for three-dimensional data |
| [bullet_chart](./components/viz/bullet_chart.toml) | 子弹图，用于与目标对比的性能展示 / Bullet chart for performance against a target |
| [candle_chart](./components/viz/candle_chart.toml) | K线图，用于金融 OHLC 数据 / Candlestick chart for financial OHLC data |
| [canvas](./components/viz/canvas.toml) | 使用盲文/点字符绘制任意形状 / Draw arbitrary shapes using braille/dot characters |
| [donut_chart](./components/viz/donut_chart.toml) | 环形图 / Ring-shaped chart with center label |
| [funnel_chart](./components/viz/funnel_chart.toml) | 漏斗图，用于转化分析 / Funnel chart for conversion analysis |
| [gantt_chart](./components/viz/gantt_chart.toml) | 甘特图，用于项目时间线可视化 / Gantt chart for project timeline visualization |
| [heatmap](./components/viz/heatmap.toml) | 颜色编码热力图矩阵，用于数据强度可视化 / Color-coded heatmap matrix for data intensity visualization |
| [histogram](./components/viz/histogram.toml) | 直方图，用于频率分布可视化 / Histogram for frequency distribution visualization |
| [network_graph](./components/viz/network_graph.toml) | 网络关系图 / Node-link diagram for graph visualization |
| [node_graph](./components/viz/node_graph.toml) | 基于 ratatui Canvas 的节点图可视化 / Node graph visualization using ratatui Canvas |
| [parcoords](./components/viz/parcoords.toml) | 平行坐标图 / Parallel coordinates plot for multi-dim data |
| [pie_chart](./components/viz/pie_chart.toml) | 带标签和百分比显示的饼图 / Pie chart with labeled segments and percentage display |
| [radar_chart](./components/viz/radar_chart.toml) | 雷达/蜘蛛图，用于多维比较 / Radar/spider chart for multi-dimensional comparison |
| [sankey_diagram](./components/viz/sankey_diagram.toml) | 桑基流程图，用于能量或物质流可视化 / Sankey flow diagram for energy or material flow visualization |
| [scatter_plot](./components/viz/scatter_plot.toml) | 散点图，用于点数据可视化 / Scatter plot for point data visualization |
| [stacked_area_chart](./components/viz/stacked_area_chart.toml) | 堆叠面积图，用于部分与整体趋势 / Stacked area chart for part-to-whole trends |
| [sunburst](./components/viz/sunburst.toml) | 旭日图 / Radial hierarchy chart with concentric ring levels |
| [treemap](./components/viz/treemap.toml) | 矩形树图，用于层级比例显示 / Treemap for hierarchical proportion display |
| [violin_plot](./components/viz/violin_plot.toml) | 小提琴图 / Violin plot showing data distribution density |
| [waterfall_chart](./components/viz/waterfall_chart.toml) | 瀑布图，用于顺序贡献分析 / Waterfall chart for sequential contribution analysis |

### ■ easing — 17 个贝塞尔缓动预设

> 缓动预设定义于 [easing_presets.rs](./tuiframe-viz/src/easing_presets.rs)，`tuiframe preview <name>` 实时预览。

| 预设 | 说明 |
|------|------|
| [linear](./tuiframe-viz/src/easing_presets.rs) | 匀速线性 / Constant speed |
| [ease-in](./tuiframe-viz/src/easing_presets.rs) | 先慢后快，逐渐加速 / Gradual acceleration, then settle |
| [ease-out](./tuiframe-viz/src/easing_presets.rs) | 先快后慢，平缓收尾 / Fast start, gentle landing |
| [ease-in-out](./tuiframe-viz/src/easing_presets.rs) | 首尾慢、中间快 / Slow start and end |
| [ease-in-quad](./tuiframe-viz/src/easing_presets.rs) | 二次方缓入 / Quadratic ease-in |
| [ease-out-quad](./tuiframe-viz/src/easing_presets.rs) | 二次方缓出 / Quadratic ease-out |
| [ease-in-out-quad](./tuiframe-viz/src/easing_presets.rs) | 二次方缓入缓出 / Quadratic ease-in-out |
| [back](./tuiframe-viz/src/easing_presets.rs) | 越过终点再回弹 / Overshoots past the end |
| [back-out](./tuiframe-viz/src/easing_presets.rs) | 起步前先回退 / Pull back before starting |
| [anticipate](./tuiframe-viz/src/easing_presets.rs) | 先蓄力后冲刺 / Anticipate then accelerate |
| [elastic](./tuiframe-viz/src/easing_presets.rs) | 弹性振荡 / Springy elastic wobble |
| [bounce](./tuiframe-viz/src/easing_presets.rs) | 落地弹跳 / Bouncing landing |
| [overshoot](./tuiframe-viz/src/easing_presets.rs) | 快速越过目标 / Fast rise past the target |
| [undershoot](./tuiframe-viz/src/easing_presets.rs) | 先下沉再回升 / Falls below before rising |
| [soft-step](./tuiframe-viz/src/easing_presets.rs) | 柔和 S 形曲线 / Soft S-shaped curve |
| [hard-step](./tuiframe-viz/src/easing_presets.rs) | 接近瞬时的硬切换 / Stiff near-instant switch |
| [whip](./tuiframe-viz/src/easing_presets.rs) | 鞭打式骤停 / Snappy whip-like snap |

---

## 🎮 交互快捷键 (Interactive Keys)

在图表预览中：

| 按键 | 功能 (Function) |
|------|-----------------|
| `1`-`9` | 切换数据变体（演示不同样式） |
| `←` / `→` | 循环切换变体 |
| `p` | 循环切换调色板 |
| `b` | 打开贝塞尔编辑器 |
| `i` / `d` | 输入自定义值 |
| `r` | 重置过渡动画 |
| `q` / `Esc` | 退出 |

---

## 🏗 项目结构 (Project Structure)

```
tuiframe/
├── components/         # .toml 组件定义（viz：20 图表 + 3 支撑）
├── templates/          # 项目脚手架模板
├── tuiframe-core/      # 库：反序列化组件/布局 TOML
├── tuiframe-viz/       # 库：交互式图表渲染 + 缓动预设
├── tuiframe-cli/       # CLI：list / info / code / browse / preview
└── Cargo.toml          # 工作区根
```

## ✅ 组件校验 (Component Validation)

```bash
cargo run -- validate             # 依赖图完整性
./scripts/compile-check.sh        # 每个组件示例都能编译成可运行程序
```

## 🤖 供 AI 助手使用 (For AI Assistants)

见 [AI_SKILL.md](./AI_SKILL.md) 了解组件目录的技能定义。

## 📝 更新计划 (Roadmap)

- [x] 20 种图表全部可交互预览
- [x] 17 种贝塞尔缓动预设
- [ ] 更多图表类型与变体
- [ ] 自定义主题与配色方案
- [ ] 更多组件分类（Layout / Navigation / Feedback…）

## 🤖 本项目由ai生成

## 📄 许可证 (License)

MIT
