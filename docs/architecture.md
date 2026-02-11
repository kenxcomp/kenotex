# Architecture

## Layered Atomic Architecture

Kenotex follows a layered atomic architecture with strict one-way dependencies (L1 → L2 → L3 → L4).

```
src/
├── main.rs                 # L1 Entry
├── coordinator/            # L2 Coordination
│   ├── app.rs              # App state (TEA pattern)
│   └── event_dispatcher.rs # Event routing
├── molecules/              # L3 Business Logic
│   ├── editor/             # Vim mode, text buffer
│   ├── list/               # Draft/archive lists
│   ├── config/             # Themes, keybindings
│   └── distribution/       # Block parser, time parser, dispatcher
├── atoms/                  # L4 Minimal Units
│   ├── widgets/            # UI components
│   ├── storage/            # File I/O
│   └── applescript/        # macOS app integration
└── types/                  # Data types
```

### L1 Entry (`main.rs`)

Terminal setup, main event loop, UI rendering. Routes events to EventDispatcher. No business logic.

### L2 Coordinator (`coordinator/`)

- **app.rs** — Central App state struct using TEA (The Elm Architecture) pattern. Holds all application state: mode, view, buffer, notes, config.
- **event_dispatcher.rs** — Routes keyboard events to appropriate handlers based on current mode and view.

### L3 Molecules (`molecules/`)

- **editor/** — TextBuffer (rope-like text storage), VimMode (key sequence handling, action generation), VisualMode, Comment, ListPrefix, MarkdownFmt, AutoPair
- **list/** — DraftList/ArchiveList (note collection management), FileChangeHandler
- **config/** — ThemeManager, keybindings
- **distribution/** — Block parser, time parser, dispatcher

### L4 Atoms (`atoms/`)

- **widgets/** — Pure UI components: EditorWidget, StatusBar, ProcessingOverlay, ConfirmOverlay, HintBar, LeaderPopup, ListItemWidget, WrapCalc, MdHighlight, SyntaxHighlighter
- **storage/** — File I/O for config and drafts, file watcher, clipboard, external editor
- **applescript/** — macOS integrations: reminders, calendar, notes, bear, obsidian

## Dependencies

- **ratatui** — Terminal UI framework
- **crossterm** — Terminal handling
- **tokio** — Async runtime
- **chrono** + **chrono-english** — Date/time parsing
- **serde** + **toml** — Configuration
- **notify** + **notify-debouncer-mini** — File system watching for live reload
- **regex** — Pattern matching
- **uuid** — Note IDs
- **syntect** — Syntax highlighting for fenced code blocks
