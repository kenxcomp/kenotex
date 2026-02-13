# Configuration Guide

## Config File Location

`~/.config/kenotex/config.toml`

See [default.toml](default.toml) for the complete configuration reference with all options and comments.

## General Settings

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

### Option Details

| Option | Default | Description |
|--------|---------|-------------|
| `theme` | `"tokyo_night"` | Color theme name |
| `leader_key` | `" "` (Space) | Leader key for shortcuts |
| `auto_save_interval_ms` | `5000` | Auto-save interval in milliseconds |
| `show_hints` | `true` | Show keyboard shortcut hints bar |
| `data_dir` | (unset) | Custom note storage path (supports `~` expansion) |
| `file_watch` | `true` | Enable/disable filesystem watching |
| `file_watch_debounce_ms` | `300` | File watcher debounce interval |
| `tab_width` | `4` | Number of spaces inserted when pressing Tab |

## Destinations

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

### Destination Options

**Reminders** (`[destinations.reminders]`):
- `app` — `"apple"` (default) or `""` to disable
- `list` — Optional Reminders list name

**Calendar** (`[destinations.calendar]`):
- `app` — `"apple"` (default) or `""` to disable
- `calendar_name` — Optional calendar name

**Notes** (`[destinations.notes]`):
- `app` — `"apple_notes"` (default), `"bear"`, `"obsidian"`, or `""` to disable
- `folder` — Optional folder name
- `vault` — Optional vault name (Obsidian only)

## Themes

Available themes:
- **Tokyo Night** — `tokyo_night`
- **Gruvbox** — `gruvbox`
- **Nord** — `nord`
- **Catppuccin Mocha** — `catppuccin_mocha`
- **Catppuccin Macchiato** — `catppuccin_macchiato`
- **Catppuccin Frappe** — `catppuccin_frappe`
- **Catppuccin Latte** — `catppuccin_latte`

Each theme includes a syntax color palette (comment, keyword, string, type, function, constant) used for fenced code block highlighting and editor syntax coloring.

Press `T` in Normal mode to cycle through themes.

## Keybindings

All keybindings can be remapped via the `[keyboard]` section. Notable options:

| Option | Default | Description |
|--------|---------|-------------|
| `leader_organize` | `"o"` | Organize checkboxes (unchecked up, checked down) |

See the full [Keybindings Reference](keybindings.md) and [default.toml](default.toml) for all available options.

## Time Patterns

Config file location: `~/.config/kenotex/time_patterns.toml` (auto-created on first run)

This file controls how `@time` expressions in `:::td` and `:::cal` blocks are recognized and parsed. It has five sections:

### Periods

Maps time-of-day keywords to default `"HH:MM"` values:

```toml
[periods]
"早上" = "09:00"
morning = "09:00"
afternoon = "14:00"
```

Used when writing `@明天早上` (resolves to 09:00) or `@tomorrow morning`.

### Offsets

Maps relative date keywords to day offsets from today:

```toml
[offsets]
"明天" = 1
tomorrow = 1
"后天" = 2
"下周" = 7
```

### Weekdays

Maps weekday aliases to standard English weekday names:

```toml
[weekdays]
"周一" = "monday"
"星期一" = "monday"
```

### Hours

Maps Chinese numeral keywords to hour numbers (0–23), enabling expressions like `@下午七点` instead of `@下午7点`:

```toml
[hours]
"一" = 1
"二" = 2
"七" = 7
"十二" = 12
```

### Minutes

Maps Chinese numeral keywords to minute numbers (0–59), enabling expressions like `@三十分` instead of `@30分`:

```toml
[minutes]
"十五" = 15
"三十" = 30
"四十五" = 45
```

Chinese numeral hours and minutes can be mixed freely with ASCII digits: `七点30分`, `7点三十分`, and `七点三十分` all work.

### Absolute Dates

`X月Y日` format is supported for specifying exact dates:

- `@2月15日16:50` — February 15th at 16:50
- `@2026年3月1日下午3点` — March 1st, 2026 at 3pm
- `@3月1号` — March 1st (号 also accepted as day suffix)
- `@1月5日` — January 5th (auto-rolls to next year if date has passed)

### Colon Time Format

`HH:MM` notation is supported alongside Chinese `点/分` notation:

- `@明天16:50` — Tomorrow at 16:50
- `@2月15日 16:50` — February 15th at 16:50 (space between date and time is allowed)
- `@3：30pm` — 3:30 PM (Chinese full-width colon `：` also accepted)

### Customization Examples

Change morning default to 8:00 AM:
```toml
[periods]
morning = "08:00"
"早上" = "08:00"
```

Add a custom keyword:
```toml
[offsets]
next_week = 7
```

**Note**: If you override a section (e.g., `[periods]`), only the keys you specify will be active — defaults for that section are replaced entirely. Sections you omit will keep their defaults.

See [default_time_patterns.toml](default_time_patterns.toml) for the complete reference with all defaults.
