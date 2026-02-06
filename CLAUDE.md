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
- `event_dispatcher.rs` - Routes keyboard events to appropriate handlers based on current mode (Normal/Insert/Visual/Search/ConfirmDelete) and view (Editor/DraftList/ArchiveList).

**L3 Molecules** (`molecules/`):
- `editor/` - TextBuffer (rope-like text storage), VimMode (key sequence handling, action generation), VisualMode (visual selection state with Character/Line/Block types, RenderSelection), Comment (HTML comment `<!-- -->` detection and toggling), ListPrefix (list prefix detection and continuation for `- [ ]`, `N.`, `N)`), MarkdownFmt (inline format detection/toggling for bold/italic/strikethrough/code)
- `list/` - DraftList/ArchiveList (note collection management with filtering/selection), FileChangeHandler (file event classification)
- `config/` - ThemeManager (tokyo_night/gruvbox/nord/catppuccin_mocha/catppuccin_macchiato/catppuccin_frappe/catppuccin_latte), keybindings
- `distribution/` - Block parser (splits content, detects type via tags/patterns), time parser (chrono-english for natural language dates), dispatcher (routes blocks to L4 AppleScript atoms based on config destinations)

**L4 Atoms** (`atoms/`):
- `widgets/` - Pure UI components: EditorWidget, StatusBar, ProcessingOverlay, ConfirmOverlay (delete confirmation dialog), HintBar (dynamic keyboard shortcut hints), LeaderPopup (visual leader key popup), ListItemWidget (list view item rendering), WrapCalc (soft-wrap cursor positioning utilities), MdHighlight (markdown inline syntax tokenizer for editor highlighting)
- `storage/` - File I/O for config and drafts (see Config Path below), file watcher (notify integration), clipboard (system clipboard integration), external_editor (external editor launching)
- `applescript/` - macOS integrations: reminders.rs, calendar.rs, notes.rs, bear.rs, obsidian.rs

### Config Path vs Data Directory

**Config directory** (`config_dir()` in `atoms/storage/config_io.rs`):
- **Unix (macOS/Linux)**: `~/.config/kenotex/` (XDG-style, preferred)
- **Fallback**: `dirs::config_dir()/kenotex/`
- Stores: `config.toml`

**Data directory** (`resolve_data_dir()` in `atoms/storage/config_io.rs`):
- When `data_dir` is set in config: uses that path (supports `~` expansion)
- When unset: falls back to config directory
- Stores: `drafts/` (draft notes), `archives/` (archived notes)

**Important**:
- All draft I/O functions accept `base_dir: &Path` — they do NOT import `config_dir`. Path resolution happens once in `App::new()`.
- Do NOT use `dirs::config_dir()` directly elsewhere. Always use `config_dir()` or `resolve_data_dir()` from `config_io.rs`.

### File Watcher

Live reload uses `notify` (v7) + `notify-debouncer-mini` for filesystem watching:
- `atoms/storage/file_watcher.rs` (L4) — wraps notify, produces `FileEvent` via `mpsc` channel
- `molecules/list/file_change_handler.rs` (L3) — classifies events, suppresses self-saves (500ms window)
- `coordinator/app.rs` (L2) — handles events: silent reload (clean buffer), conflict message (dirty buffer)
- `main.rs` (L1) — starts watcher, integrates via non-blocking `try_recv()` in event loop
- Config: `file_watch = true` (default), `file_watch_debounce_ms = 300`

### General Config Options

`config.toml` `[general]` section supports:
- `theme` - Color theme name (see ThemeManager for available themes)
- `leader_key` - Leader key for shortcuts (default: Space)
- `auto_save_interval_ms` - Auto-save interval in milliseconds
- `show_hints` - Show keyboard shortcut hints bar
- `data_dir` - Custom data directory path (supports `~` expansion)
- `file_watch` - Enable/disable filesystem watching (default: true)
- `file_watch_debounce_ms` - File watcher debounce interval (default: 300)
- `tab_width` - Tab width in spaces (default: 4)

### Destinations Config

`config.toml` `[destinations]` section routes parsed blocks to macOS apps:
- `[destinations.reminders]` - `app` (default: "apple"), `list` (optional Reminders list name)
- `[destinations.calendar]` - `app` (default: "apple"), `calendar_name` (optional calendar name)
- `[destinations.notes]` - `app` (apple_notes/bear/obsidian, default: apple_notes), `folder` (optional), `vault` (optional, Obsidian only)

### Keyboard Config

`config.toml` `[keyboard]` section supports remapping of all keybindings. Notable entries:
- `leader_comment` - Toggle HTML comment on current line (default: "c", triggered as Space+c in Normal mode)
- `visual_comment` - Toggle HTML comment on selected lines in Visual mode (default: "gc")
- `visual_line_mode` - Enter Visual Line mode (default: "V")
- `visual_block_mode` - Enter Visual Block mode (default: "ctrl+v")
- `leader_bold` - Toggle bold formatting (default: "b")
- `leader_italic` - Toggle italic formatting (default: "i")
- `leader_strikethrough` - Toggle strikethrough formatting (default: "x")
- `leader_code` - Toggle inline code formatting (default: "c")
- `leader_code_block` - Toggle code block formatting (default: "C")

### Key Data Types (`types/`)

- `AppMode` - Normal, Insert, Visual(VisualType) (Character/Line/Block), Search, Processing, ConfirmDelete
- `View` - Editor, DraftList, ArchiveList
- `SmartBlock` - Parsed content block with detected BlockType (Reminder/Calendar/Note) and ProcessingStatus (Pending/Sent/Failed/Skipped)
- `BlockType` - Reminder, Calendar, Note (in `types/block.rs`)
- `Theme` - Color theme struct with bg/fg/cursor/selection/border/accent/success/warning/error/panel fields (in `types/theme.rs`)
- `Note` - Draft/archive with id, title, content, timestamps

### Event Flow

1. `main.rs` polls keyboard events
2. `EventDispatcher::handle_key()` receives KeyEvent
3. `VimMode::handle_key()` translates to VimAction based on current mode
4. EventDispatcher routes action to appropriate handler
5. Handler mutates App state
6. `main.rs` re-renders UI

### Key VimActions

- `ToggleComment` - Toggle HTML comment (`<!-- -->`) on current line (Space+c in Normal mode)
- `VisualToggleComment` - Toggle HTML comment on selected lines (gc in Visual mode). Smart toggling: all uncommented → comment all; all commented → uncomment all; mixed → comment remaining. Empty lines are skipped.

### Visual Mode Keys

- `gg` - Jump to file start (changed from single `g` to free up `gc` for comment toggling)
- `gc` - Toggle HTML comments on selected lines

### Smart Block Detection Priority

1. Explicit tags: `:::td` (Reminder), `:::cal` (Calendar), `:::note` (Note)
2. Checkbox pattern: `- [ ]` → Reminder
3. Time expressions (English/Chinese) → Calendar
4. Default → Note
