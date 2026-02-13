# 配置指南

## 配置文件位置

`~/.config/kenotex/config.toml`

完整配置参考及所有选项说明请查看 [default.toml](default.toml)（含中英文注释）。

## 通用设置

```toml
[general]
theme = "tokyo_night"  # tokyo_night, gruvbox, nord, catppuccin_mocha, catppuccin_macchiato, catppuccin_frappe, catppuccin_latte
leader_key = " "
auto_save_interval_ms = 5000
show_hints = true      # 显示快捷键提示栏
# data_dir = "~/Documents/kenotex-notes"  # 自定义笔记存储路径
file_watch = true       # 检测外部文件更改
file_watch_debounce_ms = 300
tab_width = 4           # 按 Tab 键时插入的空格数
```

### 选项详情

| 选项 | 默认值 | 说明 |
|--------|---------|-------------|
| `theme` | `"tokyo_night"` | 颜色主题名称 |
| `leader_key` | `" "`（空格） | 快捷键前缀键 |
| `auto_save_interval_ms` | `5000` | 自动保存间隔（毫秒） |
| `show_hints` | `true` | 显示快捷键提示栏 |
| `data_dir` | （未设置） | 自定义笔记存储路径（支持 `~` 展开） |
| `file_watch` | `true` | 启用/禁用文件系统监视 |
| `file_watch_debounce_ms` | `300` | 文件监视器防抖间隔 |
| `tab_width` | `4` | 按 Tab 键时插入的空格数 |

## 目标应用

在 `config.toml` 中配置内容分发目标：

```toml
[destinations.reminders]
app = "apple"          # 设为 "" 可跳过提醒事项
# list = "工作"

[destinations.calendar]
app = "apple"          # 设为 "" 可跳过日历事件
# calendar_name = "个人"

[destinations.notes]
app = "apple_notes"    # apple_notes, bear, obsidian；设为 "" 可跳过备忘录
# folder = "Kenotex"
# vault = "MyVault"
```

### 目标选项

**提醒事项** (`[destinations.reminders]`)：
- `app` — `"apple"`（默认）或 `""` 禁用
- `list` — 可选的提醒事项列表名称

**日历** (`[destinations.calendar]`)：
- `app` — `"apple"`（默认）或 `""` 禁用
- `calendar_name` — 可选的日历名称

**备忘录** (`[destinations.notes]`)：
- `app` — `"apple_notes"`（默认）、`"bear"`、`"obsidian"` 或 `""` 禁用
- `folder` — 可选的文件夹名称
- `vault` — 可选的保险库名称（仅 Obsidian）

## 主题

可用主题：
- **Tokyo Night** — `tokyo_night`
- **Gruvbox** — `gruvbox`
- **Nord** — `nord`
- **Catppuccin Mocha** — `catppuccin_mocha`
- **Catppuccin Macchiato** — `catppuccin_macchiato`
- **Catppuccin Frappe** — `catppuccin_frappe`
- **Catppuccin Latte** — `catppuccin_latte`

每个主题包含语法配色方案（comment、keyword、string、type、function、constant），用于围栏代码块高亮和编辑器语法着色。

在 Normal 模式下按 `T` 可循环切换主题。

## 快捷键

所有快捷键可通过 `[keyboard]` 部分重新映射。常用选项：

| 选项 | 默认值 | 说明 |
|--------|---------|-------------|
| `leader_organize` | `"o"` | 整理复选框（未勾选上移，已勾选下移） |

请参阅完整的[快捷键参考](keybindings_cn.md)和 [default.toml](default.toml) 了解所有可用选项。

## 时间模式

配置文件位置：`~/.config/kenotex/time_patterns.toml`（首次运行时自动创建）

此文件控制 `:::td` 和 `:::cal` 块中 `@time` 表达式的识别和解析方式。包含三个部分：

### 时间段

将时段关键词映射为默认的 `"HH:MM"` 值：

```toml
[periods]
早上 = "09:00"
morning = "09:00"
afternoon = "14:00"
```

例如 `@明天早上`（解析为 09:00）或 `@tomorrow morning`。

### 日期偏移

将相对日期关键词映射为距今天的天数：

```toml
[offsets]
明天 = 1
tomorrow = 1
后天 = 2
下周 = 7
```

### 星期别名

将星期别名映射为标准英文星期名：

```toml
[weekdays]
周一 = "monday"
星期一 = "monday"
```

### 自定义示例

将早上默认时间改为 8:00：
```toml
[periods]
morning = "08:00"
早上 = "08:00"
```

添加自定义关键词：
```toml
[offsets]
next_week = 7
```

**注意**：如果覆盖某个部分（如 `[periods]`），则只有你指定的键会生效 — 该部分的默认值会被完全替换。未指定的部分将保留默认值。

完整默认值参考请查看 [default_time_patterns.toml](default_time_patterns.toml)。
