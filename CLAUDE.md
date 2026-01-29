# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Build release
cargo build --release

# Build debug
cargo build

# Run (debug)
cargo run

# Run (release)
./target/release/kenotex

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test distribution::parser::tests

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

Kenotex follows a **layered atomic architecture** with strict one-way dependencies (L1 → L2 → L3 → L4).

```
L1 Entry (main.rs)
    ↓
L2 Coordinator (coordinator/)
    ↓
L3 Molecules (molecules/)
    ↓
L4 Atoms (atoms/)
```

### Layer Responsibilities

**L1 Entry** (`main.rs`): Terminal setup, main event loop, UI rendering. Routes events to EventDispatcher. No business logic.

**L2 Coordinator** (`coordinator/`):
- `app.rs` - Central App state struct using TEA (The Elm Architecture) pattern. Holds all application state: mode, view, buffer, notes, config.
- `event_dispatcher.rs` - Routes keyboard events to appropriate handlers based on current mode (Normal/Insert/Visual/Search) and view (Editor/DraftList/ArchiveList).

**L3 Molecules** (`molecules/`):
- `editor/` - TextBuffer (rope-like text storage), VimMode (key sequence handling, action generation)
- `list/` - DraftList/ArchiveList (note collection management with filtering/selection)
- `config/` - ThemeManager (tokyo_night/gruvbox/nord), keybindings
- `distribution/` - Block parser (splits content, detects type via tags/patterns), time parser (chrono-english for natural language dates)

**L4 Atoms** (`atoms/`):
- `widgets/` - Pure UI components: EditorWidget, StatusBar, ProcessingOverlay
- `storage/` - File I/O for config (`~/.config/kenotex/config.toml`) and drafts (`~/.config/kenotex/drafts/`)
- `applescript/` - macOS integrations: reminders.rs, calendar.rs, notes.rs, bear.rs, obsidian.rs

### Key Data Types (`types/`)

- `AppMode` - Normal, Insert, Visual, Search, Processing
- `View` - Editor, DraftList, ArchiveList
- `SmartBlock` - Parsed content block with detected BlockType (Reminder/Calendar/Note)
- `Note` - Draft/archive with id, title, content, timestamps

### Event Flow

1. `main.rs` polls keyboard events
2. `EventDispatcher::handle_key()` receives KeyEvent
3. `VimMode::handle_key()` translates to VimAction based on current mode
4. EventDispatcher routes action to appropriate handler
5. Handler mutates App state
6. `main.rs` re-renders UI

### Smart Block Detection Priority

1. Explicit tags: `:::td` (Reminder), `:::cal` (Calendar), `:::note` (Note)
2. Checkbox pattern: `- [ ]` → Reminder
3. Time expressions (English/Chinese) → Calendar
4. Default → Note
