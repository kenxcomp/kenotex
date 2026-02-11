# Keybindings Reference

All keybindings are fully customizable. See [default.toml](default.toml) for the complete reference with default values.

## Normal Mode

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

## Visual Mode

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

## Insert Mode

| Key | Action |
|-----|--------|
| `(` `[` `{` | Auto-insert matching closing bracket |
| `'` `"` | Auto-insert matching quote |
| `*` `~` `` ` `` | Auto-insert matching pair; repeat to extend (`**\|**`, `~~\|~~`, `` `\|` ``) |
| Closing char | Skip over if already present after cursor |
| `Enter` after `:::tag` | Auto-insert closing `:::` on next line |
| `Backspace` in pair | Delete both opening and closing characters |
| `Tab` | Insert spaces (tab_width), or indent entire line on list items |
| `Shift+Tab` | Dedent current line |
| `Esc` | Return to Normal mode |

## Leader Commands (Space + key)

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

## List View

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
