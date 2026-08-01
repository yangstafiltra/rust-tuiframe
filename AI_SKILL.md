# tuiframe Component Catalog — AI Assistant Skill

## Overview / 概述

tuiframe is a **file-driven Rust TUI component framework**. Every component is defined as a `.toml` file in `components/<category>/`. To add a component, create a new `.toml` file — no Rust code changes needed for the catalog to recognize it.

tuiframe 是一个**文件驱动的 Rust TUI 组件框架**。每个组件定义为一个 `components/<category>/` 下的 `.toml` 文件。添加组件只需创建新 `.toml` 文件，目录会自动识别。

## Categories / 分类

| Category | Path | Count | Description |
|----------|------|-------|-------------|
| Core | `components/core/` | 26 | Base primitives (Block, Paragraph, Text, Span, Line, Rect, Style…) |
| Navigation | `components/navigation/` | 26 | Tabs, Sidebar, Dock, Toolbar, ActionBar, Outline, StepIndicator |
| Data | `components/data/` | 36 | Table, Chart, DataGrid, PivotTable, Counter, CalendarHeatmap |
| Data Science | `components/data_science/` | 20 | DataFrameViewer, CorrelationMatrix, NeuralNetViz, ROC |
| Input | `components/input/` | 37 | TextInput, Toggle, Form, Autocomplete, ColorInput, RatingInput |
| Layout | `components/layout/` | 27 | Split, FlexLayout, DockArea, CardLayout, Masonry, FlowLayout |
| Media | `components/media/` | 41 | Image, AudioViz, VideoPlayer, AlbumArt, ParticleSystem |
| Music | `components/music/` | 19 | PianoRoll, StepSequencer, MixerChannel, SpectrumAnalyzer |
| Feedback | `components/feedback/` | 30 | Toast, ProgressRing, ConfettiEffect, Shimmer, ParticleEffect |
| Advanced | `components/advanced/` | 56 | CodeEditor, CommandPalette, PaneWorkspace, MultiCursor |
| Utility | `components/utility/` | 31 | Scrollbar, Spotlight, Inspector, ZoomControl, LoadingScreen |
| Decoration | `components/decoration/` | 29 | Badge, Tag, GradientText, BorderVariants, AnimatedBorder |
| Viz | `components/viz/` | 20 | Histogram, Heatmap, Treemap, Candlestick, Radar, Sankey |
| Protocol | `components/protocol/` | 23 | Sixel, KittyGraphics, Hyperlink, Clipboard, MouseCapture |
| Gaming | `components/gaming/` | 32 | Snake, Tetris, Minesweeper, Chess, Sudoku, Blackjack |
| AI | `components/ai/` | 20 | StreamingText, ConversationView, TokenCounter |
| DevTools | `components/devtools/` | 83 | Picker, BranchGraph, LogViewer, K8s, Docker, CI Pipeline |
| DevOps | `components/devops/` | 26 | K8sDashboard, TerraformPlan, AnsiblePlaybook, Helm |
| Presentation | `components/presentation/` | 20 | SlideDeck, CodeSlide, LaserPointer, PresenterNotes |
| Productivity | `components/productivity/` | 39 | Pomodoro, Kanban, Mindmap, Flashcards, Calculator |
| Network | `components/network/` | 24 | HttpClient, WebSocket, MQTT, GraphQL, DNS, Redis |
| System Tools | `components/system_tools/` | 27 | DiskAnalyzer, ServiceManager, DockerManager, SSH |
| Framework | `components/framework/` | 20 | EventLoop, RenderLoop, WidgetTree, Compositor |
| Education | `components/education/` | 10 | CodeTutor, QuizViewer, LessonViewer, TextbookViewer |

**Total: 722 components** (across 25 categories)

## Component TOML Format

```toml
name = "component_name"           # 组件名
category = "category_name"        # 分类
description = "EN / CN desc"      # 中英双语描述
dependencies = ["dep1", "dep2"]   # 依赖的其他组件
features = ["feat1", "feat2"]     # 功能列表
example = '''code_example'''      # Rust 代码示例
reference_apps = ["app1", "app2"] # 参考应用
```

## How AI Should Use This

When a user asks to build a TUI app:

1. **Analyze requirements** → Determine which categories apply
2. **Search components/ by keyword** → Find matching `.toml` files
3. **Recommend component combination** → List suitable components
4. **Generate code** → Use `example` fields and reference apps for patterns
5. **Scaffold** → Use `tuiframe scaffold <template> <name>` to generate project

### Example AI Workflow

> User: "I want an AI chat TUI"
> 
> AI: "Based on `components/ai/conversation_view.toml` + `components/ai/streaming_text.toml` +
>      `components/ai/model_selector.toml` + `components/ai/token_counter.toml` +
>      `components/ai/system_prompt_editor.toml`:
> Recommended stack: ratatui + crossterm
> Key components: ConversationView, StreamingText, ModelSelector, TokenCounter, SystemPromptEditor
> Reference: tchak-rs, tokai-tui"

## Reference Applications

| App | Stars | Key Components Used |
|-----|-------|-------------------|
| [yazi](https://github.com/sxyazi/yazi) | 40.6k | FileExplorer, TreeView, Image, Tabs, StatusBar |
| [bottom](https://github.com/ClementTsang/bottom) | 10.5k | Chart, Sparkline, Gauge, Table, List |
| [gitui](https://github.com/extrawurst/gitui) | 18.5k | Table, List, DiffViewer, StatusBar, TextInput |
| [Trawl](https://github.com/yangstafiltra/Trawl) | local | Tabs, Image, List, TextInput, Split, Scrollbar |
| [spotify-tui](https://github.com/Rigellute/spotify-tui) | 17.5k | List, Table, Image, TextInput, Tabs |
| [broot](https://github.com/Canop/broot) | 12.8k | TreeView, FileExplorer, Preview |
| [bandwhich](https://github.com/imsnif/bandwhich) | 10.5k | BarChart, Sparkline, List |
| [zellij](https://github.com/zellij-org/zellij) | 22.5k | PaneWorkspace, TabView, StatusBar, Terminal |
| [frankentui](https://github.com/marler8997/frankentui) | 1.5k | PaneWorkspace, ColorPicker, CommandPalette |
| [AbstractTUI](https://github.com/RedDocMD/AbstractTUI) | - | Compositor, Theme, 3D, VoiceBindings |

## Quick Stats

- **722 components** across 25 categories
- **All bilingual**: Chinese + English descriptions
- **Each has**: name, category, description, dependencies, features, example, reference_apps
- **Framework-agnostic**: Components are pure TOML metadata; any Rust TUI framework can implement them

## Project Structure

```
tuiframe/
├── components/         # Component catalog (722 files)
│   ├── advanced/       #   56 components
│   ├── ai/             #   20 components
│   ├── core/           #   26 components
│   ├── data/           #   36 components
│   ├── data_science/   #   20 components
│   ├── decoration/     #   29 components
│   ├── devops/         #   26 components
│   ├── devtools/       #   83 components
│   ├── education/      #   10 components
│   ├── feedback/       #   30 components
│   ├── framework/      #   20 components
│   ├── gaming/         #   32 components
│   ├── input/          #   37 components
│   ├── layout/         #   27 components
│   ├── media/          #   41 components
│   ├── music/          #   19 components
│   ├── navigation/     #   26 components
│   ├── network/        #   24 components
│   ├── presentation/   #   20 components
│   ├── productivity/   #   39 components
│   ├── protocol/       #   23 components
│   ├── system_tools/   #   27 components
│   ├── utility/        #   31 components
│   └── viz/            #   20 components
├── tuiframe-core/      # Core library crate
├── tuiframe-cli/       # CLI tool crate
└── Cargo.toml          # workspace root
```

## Commands

```bash
tuiframe list                    # List all components
tuiframe info <name>             # Show component details
tuiframe browse                  # TUI browser
tuiframe scaffold <name> <tmpl>  # Generate project
```
