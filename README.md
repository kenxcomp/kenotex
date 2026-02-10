# Kenotex

A Vim-style TUI note-taking application that intelligently distributes content to Apple Reminders, Calendar, and Notes apps.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Keybindings](#keybindings)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Dependencies](#dependencies)
- [License](#license)

## Features

### Editor

- **Vim-style Modal Editing**: Full support for Normal, Insert, Visual (Character/Line/Block), and Search modes
- **Soft-Wrap Cursor**: Cursor correctly tracks position on soft-wrapped lines in Normal, Insert, and Visual modes
- **Editor Search**: Case-insensitive forward/backward search with wrap-around, incremental match highlighting (`/` to search, `n`/`N` to navigate matches)
- **Comment Toggle**: Toggle HTML comments (`<!-- -->`) per-line with `gcc` in Normal mode or `gc` on visual selection
- **Markdown Formatting**: Toggle bold, italic, strikethrough, inline code, and code block formatting via `Space+key` in Normal and Visual modes
- **Syntax Highlighting**: Real-time visual highlighting for inline code, bold, italic, strikethrough, code blocks, list markers, and `:::` tags (both opening and closing) in the editor
- **Auto-Pair Insertion**: Automatically inserts matching pairs in Insert mode — brackets (`()`, `[]`, `{}`), quotes (`''`, `""`), markdown formatting (`**`, `~~`, `` ` ``), and closing `:::` tags after opening tag lines. Typing a closing character skips over it if already present; backspace between a pair deletes both
- **Visual Selection Wrapping**: In Visual mode, typing a bracket, quote, or formatting character wraps the selection with the matching pair
- **Clipboard Paste**: Multi-line clipboard paste with `p`/`P` (Normal mode) and `Cmd+V` (Insert mode) correctly preserves line breaks via bracketed paste support
- **List Continuation**: Auto-continue list prefixes (`- [ ]`, `-`, `1.`, `1)`) when pressing `o` or `Enter`
- **CJK/Wide-Character Support**: Full support for Chinese, Japanese, and Korean characters in all editing modes — Visual Block selection uses display-column alignment so selections remain rectangular across mixed-width lines, cursor movement tracks display columns correctly, and soft-wrap never splits a wide character

### Content Distribution

- **Smart Block Detection**: Identifies content type based on explicit tags (`:::td`, `:::cal`, `:::note`)
- **Multi-app Distribution**: Send content to Apple Reminders, Calendar, Notes, Bear, or Obsidian with real dispatch
- **Destination Skip**: Set `app = ""` to disable any destination; skipped blocks show "-" in the processing overlay
- **Comment on Success**: Successfully dispatched blocks are wrapped with `<!-- -->` in the editor buffer
- **Idempotent Dispatch**: Already-commented blocks are automatically skipped on re-dispatch, preventing duplicates
- **Auto-Archive**: When all blocks in a draft are successfully sent, the draft is automatically archived and the view returns to the draft list

### Notes Management

- **Markdown Storage**: All notes stored as markdown files in `~/.config/kenotex/drafts/`
- **Configurable Data Directory**: Store notes anywhere with `data_dir` config option (supports `~` expansion)
- **Live Reload**: Detects external file changes and reloads notes automatically with conflict resolution
- **Delete Confirmation**: Centered overlay dialog confirms before deleting notes in list views
- **Auto-save**: Configurable auto-save interval

### Customization

- **Theme Support**: Tokyo Night, Gruvbox, Nord, and Catppuccin (Mocha/Macchiato/Frappé/Latte) themes
- **Configurable Keybindings**: Full keyboard remapping via `config.toml` (see [docs/default.toml](docs/default.toml))

## Quick Start

1. **Install** via Homebrew:
   ```bash
   brew tap kenxcomp/tap && brew install kenotex
   ```

2. **Run** the application:
   ```bash
   kenotex
   ```

3. **Create a note**: Press `Space + nn` to create a new note.

4. **Write content**: Press `i` to enter Insert mode and start typing.

5. **Use tags** to mark content for distribution:
   ```
   :::td
   - Buy groceries @tomorrow
   - Call dentist @3pm
   :::

   :::cal
   Team meeting @Monday 10am
   :::
   ```

6. **Distribute**: Press `Esc` to return to Normal mode, then `Space + s` to process and send blocks to their destination apps.

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap kenxcomp/tap && brew install kenotex
```

### Build from Source

```bash
git clone https://github.com/kenxcomp/kenotex.git
cd kenotex
cargo build --release

# Run
./target/release/kenotex
```

## Usage

### Smart Block Syntax

Kenotex uses a **strict tag-only system**. Only content wrapped in explicit tag pairs is processed. Content outside tags is ignored.

**Tag format:**
- Opening tag: `:::td`, `:::cal`, or `:::note` on its own line
- Closing tag: `:::` on its own line
- Content between tags is processed

**Block types:**
- `:::td ... :::` — Reminders
- `:::cal ... :::` — Calendar events
- `:::note ... :::` — Notes (Apple Notes / Bear / Obsidian)

**Example:**

```markdown
# Meeting Notes

:::td
- Prepare presentation slides @Friday
- Review PR #123
- Update documentation
:::

:::cal
Team standup tomorrow at 10am
Room 301
:::

:::note
Remember to ask about Q2 roadmap
:::
```

**List handling** (within `:::td` blocks):
- Detects: `-`, `*`, `- [ ]`, `- []` at line start
- Creates a separate reminder for each list item
- Strips list prefix before creating the reminder

**Time expressions:**
- `@time` syntax: `@tomorrow`, `@9pm`, `@Monday`, `@明天早上8点`, `@下周一`
- Works in both `:::td` and `:::cal` blocks
- Editor highlights `@time` in bold accent color for visual feedback

**Warnings:**
- Unclosed tags show a warning with line number
- Empty blocks (no content between tags) are ignored

### List Continuation

When pressing `o` (Normal mode) or `Enter` (Insert mode) on a list line, the list prefix is automatically continued on the new line:

- `- [ ] ` / `- [x] ` / `- [X] ` → new line with `- [ ] ` (always unchecked)
- `- ` → new line with `- `
- `1. ` → new line with `2. ` (auto-incrementing)
- `1) ` → new line with `2) ` (auto-incrementing)

**Bullet.vim behavior:** If the current line contains only a list prefix with no text after it, pressing `o` or `Enter` removes the prefix and inserts a blank line instead.

Indentation (leading whitespace) is preserved.

## Keybindings

All keybindings are fully customizable. See [docs/default.toml](docs/default.toml) for the complete reference with default values.

### Normal Mode

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode |
| `I` | Enter Insert mode at line start |
| `a` | Enter Insert mode (append) |
| `A` | Enter Insert mode at line end |
| `o` | Insert line below (auto-continues list prefixes) |
| `O` | Insert line above |
| `v` | Enter Visual mode |
| `V` | Enter Visual Line mode |
| `Ctrl+V` | Enter Visual Block mode |
| `h/j/k/l` | Navigation (left/down/up/right) |
| `w/b/e` | Word forward/backward/end |
| `0/$` | Line start/end |
| `^` | First non-blank character on line |
| `gg/G` | File start/end |
| `Ctrl+U/Ctrl+D` | Scroll up/down 5 lines |
| `x` | Delete character |
| `dd` | Delete line |
| `dw/de/d$/d0/d^/dG/dg/db` | Delete with motion (word/word-end/end/start/first-non-blank/file-end/file-start/word-back) |
| `yy` | Yank (copy) line to clipboard |
| `yw/ye/y$/y0/y^/yG/yg/yb` | Yank with motion |
| `p` | Paste after cursor (or below for linewise) |
| `P` | Paste before cursor (or above for linewise) |
| `gcc` | Toggle HTML comment (`<!-- -->`) on current line |
| `>` | Indent current line |
| `<` | Dedent current line |
| `u` | Undo |
| `Ctrl+R` | Redo |
| `T` | Cycle theme |
| `/` or `f` | Enter Search mode |
| `n` | Jump to next search match |
| `N` | Jump to previous search match |
| `Ctrl+L` | Reload file from disk (useful when file changed externally) |
| `Ctrl+G` | Open buffer in external editor (`$VISUAL` / `$EDITOR` / `vi`) |
| `Esc` | Return to Normal mode / Clear search highlights |
| `Ctrl+C` or `Ctrl+Q` | Quit |

### Visual Mode

| Key | Action |
|-----|--------|
| `h/j/k/l` | Extend selection |
| `w/b/e` | Extend by word forward/backward/end |
| `0/$` | Extend to line start/end |
| `^` | Extend to first non-blank character |
| `G` | Extend to file end |
| `Ctrl+U/Ctrl+D` | Extend selection 5 lines up/down |
| `d` | Delete selection (copies to clipboard) |
| `y` | Yank (copy) selection to clipboard |
| `gc` | Toggle HTML comment on selected lines |
| `Space + b/i/x/c/C` | Toggle bold/italic/strikethrough/code/code-block on selection |
| `(` `)` `[` `]` `{` `}` `'` `"` `` ` `` `*` `~` | Wrap selection with matching pair |
| `>` | Indent selected lines |
| `<` | Dedent selected lines |
| `gg` | Move to file start |
| `Esc` | Exit Visual mode |

### Insert Mode

| Key | Action |
|-----|--------|
| `(` `[` `{` | Auto-insert matching closing bracket |
| `'` `"` | Auto-insert matching quote |
| `*` `~` `` ` `` | Auto-insert matching pair; repeat to extend (`**\|**`, `~~\|~~`, ` `` \| `` `) |
| Closing char | Skip over if already present after cursor |
| `Enter` after `:::tag` | Auto-insert closing `:::` on next line |
| `Backspace` in pair | Delete both opening and closing characters |
| `Tab` | Insert spaces (tab_width) |
| `Shift+Tab` | Dedent current line |
| `Esc` | Return to Normal mode |

### Leader Commands (Space + key)

| Key | Action |
|-----|--------|
| `Space + s` | Process and distribute blocks |
| `Space + l` | Open draft list |
| `Space + nn` | Create new note |
| `Space + q` | Quit |
| `Space + h` | Toggle shortcut hints bar |
| `Space + d` | Toggle checkbox (`- [ ]` ↔ `- [x]`) on current line |
| `Space + mc` | Insert checkbox (`- [ ] `) on current line |
| `Space + b` | Toggle **bold** (`**text**`) |
| `Space + i` | Toggle *italic* (`*text*`) |
| `Space + x` | Toggle ~~strikethrough~~ (`~~text~~`) |
| `Space + c` | Toggle inline code (`` `text` ``) |
| `Space + C` | Toggle code block (` ``` `) |

### List View

| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `Enter/l/i` | Open selected note |
| `a` | Archive note (drafts view) |
| `r` | Restore note (archive view) |
| `d` | Delete note (with confirmation) |
| `n` | Create new note |
| `A` | Toggle to archive view |
| `/` or `f` | Search notes |
| `Space` | Toggle selection |
| `Esc` | Back to editor |

## Configuration

Config file location: `~/.config/kenotex/config.toml`

See [docs/default.toml](docs/default.toml) for the complete configuration reference with all options and comments.

```toml
[general]
theme = "tokyo_night"  # tokyo_night, gruvbox, nord, catppuccin_mocha, catppuccin_macchiato, catppuccin_frappe, catppuccin_latte
leader_key = " "
auto_save_interval_ms = 5000
show_hints = true      # Show shortcut hints bar
# data_dir = "~/Documents/kenotex-notes"  # Custom note storage path
file_watch = true       # Detect external file changes
file_watch_debounce_ms = 300
tab_width = 4           # Number of spaces inserted when pressing Tab
```

### Destinations

Configure where content gets distributed in `config.toml`:

```toml
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
