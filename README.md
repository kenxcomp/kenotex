# Kenotex

A Vim-style TUI note-taking application that intelligently distributes content to Apple Reminders, Calendar, and Notes apps.

## Features

- **Vim-style Modal Editing**: Full support for Normal, Insert, Visual, and Search modes
- **Smart Block Detection**: Automatically identifies content type based on tags and patterns
- **Multi-app Distribution**: Send content to Apple Reminders, Calendar, Notes, Bear, or Obsidian
- **Theme Support**: Tokyo Night, Gruvbox, and Nord themes
- **Markdown Storage**: All notes stored as markdown files in `~/.config/kenotex/drafts/`
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
| `o` | Insert line below |
| `O` | Insert line above |
| `v` | Enter Visual mode |
| `h/j/k/l` | Navigation (left/down/up/right) |
| `w/b` | Word forward/backward |
| `0/$` | Line start/end |
| `g/G` | File start/end |
| `x` | Delete character |
| `d` | Delete line |
| `T` | Cycle theme |
| `/` or `f` | Enter Search mode |
| `Esc` | Return to Normal mode |
| `Ctrl+C` or `Ctrl+Q` | Quit |

### Leader Commands (Space + key)

| Key | Action |
|-----|--------|
| `Space + s` | Process and distribute blocks |
| `Space + l` | Open draft list |
| `Space + n` | Create new note |
| `Space + w` | Save current note |

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
theme = "tokyo_night"  # tokyo_night, gruvbox, nord
leader_key = " "
auto_save_interval_ms = 5000

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
# Leader commands
leader_process = "s"
leader_list = "l"
leader_new = "n"
leader_save = "w"

[destinations.reminders]
app = "apple"
# list = "Work"

[destinations.calendar]
app = "apple"
# calendar_name = "Personal"

[destinations.notes]
app = "apple_notes"  # apple_notes, bear, obsidian
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
│   └── distribution/       # Block parser, time parser
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
- **regex** - Pattern matching
- **uuid** - Note IDs

## License

MIT
