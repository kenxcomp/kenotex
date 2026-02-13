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
- `editor/` - TextBuffer (rope-like text storage), VimMode (key sequence handling, action generation), VisualMode (visual selection state with Character/Line/Block types, RenderSelection), Comment (HTML comment `<!-- -->` detection and toggling), ListPrefix (list prefix detection and continuation for `- [ ]`, `-`, `* `, `N.`, `N)`, plus `hanging_indent_width()` for soft-wrap alignment), MarkdownFmt (inline format detection/toggling for bold/italic/strikethrough/code), AutoPair (auto-pair insertion for brackets/quotes/markdown formatting and closing `:::` tags, visual selection wrapping)
- `list/` - DraftList/ArchiveList (note collection management with filtering/selection), FileChangeHandler (file event classification)
- `config/` - ThemeManager (tokyo_night/gruvbox/nord/catppuccin_mocha/catppuccin_macchiato/catppuccin_frappe/catppuccin_latte), keybindings
- `distribution/` - Block parser (splits content, detects type via tags/patterns), time parser (configurable via TimeConfig, supports CJK and English natural language dates), dispatcher (routes blocks to L4 AppleScript atoms based on config destinations, threads TimeConfig)

**L4 Atoms** (`atoms/`):
- `widgets/` - Pure UI components: EditorWidget, StatusBar, ProcessingOverlay, ConfirmOverlay (delete confirmation dialog), HintBar (dynamic keyboard shortcut hints), LeaderPopup (visual leader key popup), ListItemWidget (list view item rendering), WrapCalc (soft-wrap cursor positioning utilities with hanging indent support), MdHighlight (markdown inline syntax tokenizer for editor highlighting), SyntaxHighlighter (syntect-based fenced code block syntax highlighting)
- `storage/` - File I/O for config and drafts (see Config Path below), file watcher (notify integration), clipboard (system clipboard integration), external_editor (external editor launching), load_time_config (loads `time_patterns.toml` with defaults)
- `text/` - Pure text manipulation: CheckboxSort (checkbox list sorting by checked/unchecked state)
- `applescript/` - macOS integrations: reminders.rs, calendar.rs, notes.rs, bear.rs, obsidian.rs

### Config Path vs Data Directory

**Config directory** (`config_dir()` in `atoms/storage/config_io.rs`):
- **Unix (macOS/Linux)**: `~/.config/kenotex/` (XDG-style, preferred)
- **Fallback**: `dirs::config_dir()/kenotex/`
- Stores: `config.toml`, `time_patterns.toml`

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

### Time Patterns Config

`time_patterns.toml` (separate file in config directory, auto-created on first run):
- `[periods]` - Time-of-day keyword → `"HH:MM"` default (e.g., `早上 = "09:00"`, `morning = "09:00"`)
- `[offsets]` - Relative date keyword → day offset (e.g., `明天 = 1`, `tomorrow = 1`)
- `[weekdays]` - Weekday alias → English name (e.g., `周一 = "monday"`, `星期一 = "monday"`)
- `[hours]` - Chinese numeral keyword → hour number 0-23 (e.g., `七 = 7`, `十二 = 12`). Enables `@下午七点` alongside `@下午7点`
- `[minutes]` - Chinese numeral keyword → minute number 0-59 (e.g., `三十 = 30`, `四十五 = 45`). Enables `@七点三十分` alongside `@7点30分`
- Overriding a section replaces all defaults for that section; omitted sections keep defaults

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
- `first_non_blank` - Jump to first non-whitespace character (default: "^")
- `word_end` - Jump to end of current/next word (default: "e")
- `scroll_up` - Scroll up 5 lines (default: "ctrl+u")
- `scroll_down` - Scroll down 5 lines (default: "ctrl+d")
- `leader_organize` - Organize checkboxes (default: "o")

### Key Data Types (`types/`)

- `AppMode` - Normal, Insert, Visual(VisualType) (Character/Line/Block), Search, Processing, ConfirmDelete
- `View` - Editor, DraftList, ArchiveList
- `SmartBlock` - Parsed content block with detected BlockType (Reminder/Calendar/Note) and ProcessingStatus (Pending/Sent/Failed/Skipped)
- `BlockType` - Reminder, Calendar, Note (in `types/block.rs`)
- `Theme` - Color theme struct with bg/fg/cursor/selection/border/accent/success/warning/error/panel fields plus 6 syntax color fields: comment, keyword, string, type_name, function, constant (in `types/theme.rs`)
- `Note` - Draft/archive with id, title, content, timestamps
- `TimeConfig` - Configurable time pattern definitions with periods (keyword→"HH:MM"), offsets (keyword→days), weekdays (alias→english name), hours (Chinese numeral→hour number), minutes (Chinese numeral→minute number) HashMaps + defaults + helper methods (in `types/time_config.rs`)

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
- `MoveFirstNonBlank` - Jump to first non-whitespace character on line (^ in Normal/Visual mode)
- `MoveWordEnd` - Jump to end of current/next word (e in Normal/Visual mode)
- `MoveUp5Lines` - Scroll up 5 lines (Ctrl+U in Normal/Visual mode)
- `MoveDown5Lines` - Scroll down 5 lines (Ctrl+D in Normal/Visual mode)
- `VisualWrapPair(char, char)` - Wrap visual selection with a character pair (brackets, quotes, or markdown formatting characters in Visual mode)
- `OrganizeCheckboxes` - Sort checkbox items: unchecked (`- [ ]`) float up, checked (`- [x]`) sink down (Space+o in Normal mode)

### Visual Mode Keys

- `gg` - Jump to file start (changed from single `g` to free up `gc` for comment toggling)
- `gc` - Toggle HTML comments on selected lines
- `^` - Extend selection to first non-blank character
- `e` - Extend selection to word end
- `Ctrl+U` - Extend selection 5 lines up
- `Ctrl+D` - Extend selection 5 lines down

### Smart Block Parsing (Strict Tag-Only System)

**IMPORTANT**: Kenotex uses a **strict tag-only system**. Only content wrapped in explicit tag pairs is processed.

**Tag format**:
- Opening tag: `:::td`, `:::cal`, or `:::note` on its own line
- Closing tag: `:::` on its own line
- Content between tags is processed
- Content outside tags is ignored

**Example**:
```
:::td
- Buy milk @明天早上8点
- Walk dog @9pm
* Feed cat
:::

:::cal
Team meeting @下周一上午9点
Room 301
:::

:::note
Random thought
:::
```

**Block types**:
- `:::td ... :::` → Reminder
- `:::cal ... :::` → Calendar event
- `:::note ... :::` → Note (Apple Notes/Bear/Obsidian)

**List handling** (within `:::td` blocks):
- Detects: `-`, `*`, `- [ ]`, `- []` at line start
- Creates separate reminder for each list item
- Strips list prefix before creating reminder

**Time expressions**:
- `@time` syntax: `@明天早上8点`, `@tomorrow`, `@9pm`, `@下周一`
- Works in both `:::td` and `:::cal` blocks
- Provides explicit time specification
- Editor highlights `@time` in bold accent color for visual feedback

**Warnings**:
- Unclosed tags show warning with line number
- Empty blocks (no content between tags) are ignored
- Invalid `@time` expressions (like `@john`) are not parsed

### Auto-Archive After Processing

When blocks are processed via `Space+s`, successfully sent blocks are wrapped in HTML comments (`<!-- -->`). If **all content** in the draft ends up commented (i.e., every block was sent successfully), the draft is automatically archived and the view switches back to the DraftList.

- `is_all_commented()` in `comment.rs` (L3) detects whether the entire buffer is wrapped in multi-line HTML comment blocks
- `archive_current_note()` in `app.rs` (L2) handles the archive operation
- `finish_processing()` checks for auto-archive after processing completes
