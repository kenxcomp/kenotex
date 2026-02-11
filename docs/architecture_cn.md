# 架构

## 分层原子架构

Kenotex 遵循分层原子架构，严格单向依赖（L1 → L2 → L3 → L4）。

```
src/
├── main.rs                 # L1 入口层
├── coordinator/            # L2 协调层
│   ├── app.rs              # 应用状态（TEA 模式）
│   └── event_dispatcher.rs # 事件路由
├── molecules/              # L3 分子层（业务逻辑）
│   ├── editor/             # Vim 模式、文本缓冲区
│   ├── list/               # 草稿/归档列表
│   ├── config/             # 主题、快捷键
│   └── distribution/       # 块解析器、时间解析器、分发器
├── atoms/                  # L4 原子层（最小单元）
│   ├── widgets/            # UI 组件
│   ├── storage/            # 文件 I/O
│   └── applescript/        # macOS 应用集成
└── types/                  # 数据类型
```

### L1 入口层 (`main.rs`)

终端设置、主事件循环、UI 渲染。将事件路由到 EventDispatcher。不包含业务逻辑。

### L2 协调层 (`coordinator/`)

- **app.rs** — 中央 App 状态结构体，使用 TEA（The Elm Architecture）模式。持有所有应用状态：模式、视图、缓冲区、笔记、配置。
- **event_dispatcher.rs** — 根据当前模式和视图将键盘事件路由到相应的处理程序。

### L3 分子层 (`molecules/`)

- **editor/** — TextBuffer（类绳索文本存储）、VimMode（按键序列处理、动作生成）、VisualMode、Comment、ListPrefix、MarkdownFmt、AutoPair
- **list/** — DraftList/ArchiveList（笔记集合管理）、FileChangeHandler
- **config/** — ThemeManager、快捷键
- **distribution/** — 块解析器、时间解析器、分发器

### L4 原子层 (`atoms/`)

- **widgets/** — 纯 UI 组件：EditorWidget、StatusBar、ProcessingOverlay、ConfirmOverlay、HintBar、LeaderPopup、ListItemWidget、WrapCalc、MdHighlight
- **storage/** — 配置和草稿的文件 I/O、文件监视器、剪贴板、外部编辑器
- **applescript/** — macOS 集成：提醒事项、日历、备忘录、Bear、Obsidian

## 依赖

- **ratatui** — 终端 UI 框架
- **crossterm** — 终端处理
- **tokio** — 异步运行时
- **chrono** + **chrono-english** — 日期/时间解析
- **serde** + **toml** — 配置管理
- **notify** + **notify-debouncer-mini** — 文件系统监视（实时重载）
- **regex** — 模式匹配
- **uuid** — 笔记 ID
