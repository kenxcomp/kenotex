# Kenotex

A Vim-style TUI note-taking application that intelligently distributes content to Apple Reminders, Calendar, and Notes apps.

## Features

- **Vim-style Modal Editing**: Full support for Normal, Insert, Visual, and Search modes
- **Smart Block Detection**: Automatically identifies content type based on tags and patterns
- **Multi-app Distribution**: Send content to Apple Reminders, Calendar, Notes, Bear, or Obsidian with real dispatch
- **Destination Skip**: Set `app = ""` to disable any destination; skipped blocks show "-" in the processing overlay
- **Comment on Success**: Successfully dispatched blocks are wrapped with `<!-- -->` in the editor buffer
- **Idempotent Dispatch**: Already-commented blocks are automatically skipped on re-dispatch, preventing duplicates
- **Theme Support**: Tokyo Night, Gruvbox, Nord, and Catppuccin (Mocha/Macchiato/Frappé/Latte) themes
- **Markdown Storage**: All notes stored as markdown files in `~/.config/kenotex/drafts/`
- **Configurable Data Directory**: Store notes anywhere with `data_dir` config option (supports `~` expansion)
- **Live Reload**: Detects external file changes and reloads notes automatically with conflict resolution
- **Soft-Wrap Cursor**: Cursor correctly tracks position on soft-wrapped lines in Normal, Insert, and Visual modes
- **Auto-save**: Configurable auto-save interval

## Installation

```bash
# Clone and build
git clone https://github.com/your-username/kenotex.git
cd kenotex/kenotex
cargo build --release

# Run
./target/release/kenotex
```

## Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode |
| `a` | Enter Insert mode (append) |
| `o` | Insert line below (auto-continues list prefixes) |
| `O` | Insert line above |
| `v` | Enter Visual mode |
| `h/j/k/l` | Navigation (left/down/up/right) |
| `w/b` | Word forward/backward |
| `0/$` | Line start/end |
| `g/G` | File start/end |
| `x` | Delete character |
| `dd` | Delete line |
| `dw/d$/d0/dG/dg/db` | Delete with motion (word/end/start/file-end/file-start/word-back) |
| `yy` | Yank (copy) line to clipboard |
| `yw/y$/y0/yG/yg/yb` | Yank with motion |
| `p` | Paste after cursor (or below for linewise) |
| `P` | Paste before cursor (or above for linewise) |
| `u` | Undo |
| `Ctrl+R` | Redo |
| `T` | Cycle theme |
| `/` or `f` | Enter Search mode |
| `Ctrl+L` | Reload file from disk (useful when file changed externally) |
| `Ctrl+G` | Open buffer in external editor (`$VISUAL` / `$EDITOR` / `vi`) |
| `Esc` | Return to Normal mode |
| `Ctrl+C` or `Ctrl+Q` | Quit |

### Visual Mode

| Key | Action |
|-----|--------|
| `h/j/k/l` | Extend selection |
| `w/b` | Extend by word |
| `0/$` | Extend to line start/end |
| `g/G` | Extend to file start/end |
| `d` | Delete selection (copies to clipboard) |
| `y` | Yank (copy) selection to clipboard |
| `Esc` | Exit Visual mode |

### Leader Commands (Space + key)

| Key | Action |
|-----|--------|
| `Space + s` | Process and distribute blocks |
| `Space + l` | Open draft list |
| `Space + n` | Create new note |
| `Space + q` | Quit |
| `Space + th` | Toggle shortcut hints bar |
| `Space + mc` | Insert checkbox (`- [ ] `) on current line |
| `Space + mt` | Toggle checkbox (`- [ ]` ↔ `- [x]`) on current line |

### List View

| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `Enter/l/i` | Open selected note |
| `a` | Archive note (drafts view) |
| `r` | Restore note (archive view) |
| `d` | Delete note |
| `n` | Create new note |
| `A` | Toggle to archive view |
| `/` or `f` | Search notes |
| `Space` | Toggle selection |
| `Esc` | Back to editor |

## List Continuation

When pressing `o` (Normal mode) or `Enter` (Insert mode) on a list line, the list prefix is automatically continued on the new line:

- `- [ ] ` / `- [x] ` / `- [X] ` → new line with `- [ ] ` (always unchecked)
- `- ` → new line with `- `
- `1. ` → new line with `2. ` (auto-incrementing)
- `1) ` → new line with `2) ` (auto-incrementing)

**Bullet.vim behavior:** If the current line contains only a list prefix with no text after it, pressing `o` or `Enter` removes the prefix and inserts a blank line instead.

Indentation (leading whitespace) is preserved.

## Smart Block Syntax

Kenotex automatically detects block types using these patterns:

### Explicit Tags (Highest Priority)
- `:::td` - Force block to Reminders
- `:::cal` - Force block to Calendar
- `:::note` - Force block to Notes

### Automatic Detection
- `- [ ]` checkbox items -> Reminders
- Time expressions (tomorrow, Monday, 10am, etc.) -> Calendar
- Chinese time (明天, 下周, etc.) -> Calendar
- Everything else -> Notes

### Example

```markdown
# Meeting Notes

:::cal Team standup tomorrow at 10am

- [ ] Prepare presentation slides
- [ ] Review PR #123
- [ ] Update documentation

:::note Remember to ask about Q2 roadmap
```

## Configuration

Config file location: `~/.config/kenotex/config.toml`

See [docs/default.toml](docs/default.toml) for a complete configuration reference with comments.

```toml
[general]
theme = "tokyo_night"  # tokyo_night, gruvbox, nord, catppuccin_mocha, catppuccin_macchiato, catppuccin_frappe, catppuccin_latte
leader_key = " "
auto_save_interval_ms = 5000
show_hints = true      # Show shortcut hints bar
# data_dir = "~/Documents/kenotex-notes"  # Custom note storage path
file_watch = true       # Detect external file changes
file_watch_debounce_ms = 300

[keyboard]
layout = "qwerty"
# Navigation
move_left = "h"
move_down = "j"
move_up = "k"
move_right = "l"
word_forward = "w"
word_backward = "b"
# Insert mode
insert = "i"
insert_append = "a"
# Editing
delete_char = "x"
delete_line = "d"
undo = "u"
redo = "ctrl+r"
yank = "y"
paste_after = "p"
paste_before = "P"
# Leader commands
leader_process = "s"
leader_list = "l"
leader_new = "n"
leader_quit = "q"

[destinations.reminders]
app = "apple"          # Set to "" to skip reminders
# list = "Work"

[destinations.calendar]
app = "apple"          # Set to "" to skip calendar events
# calendar_name = "Personal"

[destinations.notes]
app = "apple_notes"    # apple_notes, bear, obsidian; set to "" to skip notes
# folder = "Kenotex"
# vault = "MyVault"
```

## Architecture

The project follows a layered atomic architecture:

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

## Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Terminal handling
- **tokio** - Async runtime
- **chrono** + **chrono-english** - Date/time parsing
- **serde** + **toml** - Configuration
- **notify** + **notify-debouncer-mini** - File system watching for live reload
- **regex** - Pattern matching
- **uuid** - Note IDs

## License

MIT
