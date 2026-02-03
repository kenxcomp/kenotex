# Kenotex

一款 Vim 风格的终端笔记应用，能够智能地将内容分发到 Apple 提醒事项、日历和备忘录应用。

## 功能特性

- **Vim 风格模态编辑**：完整支持 Normal、Insert、Visual 和 Search 模式
- **智能块检测**：基于标签和模式自动识别内容类型
- **多应用分发**：将内容发送到 Apple 提醒事项、日历、备忘录、Bear 或 Obsidian，支持实际调度
- **目标跳过**：设置 `app = ""` 可禁用任何目标应用；跳过的块在处理覆盖层中显示 "-"
- **成功后注释**：成功分发的块会在编辑器缓冲区中用 `<!-- -->` 包裹
- **幂等分发**：已注释的块在重新分发时会自动跳过，防止重复发送
- **主题支持**：Tokyo Night、Gruvbox、Nord 和 Catppuccin（Mocha/Macchiato/Frappé/Latte）主题
- **Markdown 存储**：所有笔记以 markdown 文件形式存储在 `~/.config/kenotex/drafts/`
- **自定义数据目录**：通过 `data_dir` 配置选项将笔记存储在任意位置（支持 `~` 展开）
- **实时重载**：自动检测外部文件更改并重新加载笔记，支持冲突解决
- **软换行光标**：光标在软换行行上正确跟踪位置，支持 Normal、Insert 和 Visual 模式
- **删除确认**：在列表视图中删除笔记时显示居中确认对话框
- **自动保存**：可配置的自动保存间隔

## 安装

```bash
# 克隆并构建
git clone https://github.com/your-username/kenotex.git
cd kenotex/kenotex
cargo build --release

# 运行
./target/release/kenotex
```

## 快捷键

### Normal 模式

| 按键 | 操作 |
|-----|--------|
| `i` | 进入 Insert 模式 |
| `a` | 进入 Insert 模式（追加） |
| `o` | 在下方插入新行（自动续接列表前缀） |
| `O` | 在上方插入新行 |
| `v` | 进入 Visual 模式 |
| `h/j/k/l` | 导航（左/下/上/右） |
| `w/b` | 向前/向后移动一个单词 |
| `0/$` | 行首/行尾 |
| `g/G` | 文件开头/结尾 |
| `x` | 删除字符 |
| `dd` | 删除整行 |
| `dw/d$/d0/dG/dg/db` | 配合动作删除（单词/行尾/行首/文件尾/文件首/前一单词） |
| `yy` | 复制整行到剪贴板 |
| `yw/y$/y0/yG/yg/yb` | 配合动作复制 |
| `p` | 在光标后粘贴（行级操作时在下方粘贴） |
| `P` | 在光标前粘贴（行级操作时在上方粘贴） |
| `u` | 撤销 |
| `Ctrl+R` | 重做 |
| `T` | 切换主题 |
| `/` 或 `f` | 进入搜索模式 |
| `Ctrl+L` | 从磁盘重新加载文件（文件被外部修改时使用） |
| `Ctrl+G` | 在外部编辑器中打开缓冲区（`$VISUAL` / `$EDITOR` / `vi`） |
| `Esc` | 返回 Normal 模式 |
| `Ctrl+C` 或 `Ctrl+Q` | 退出 |

### Visual 模式

| 按键 | 操作 |
|-----|--------|
| `h/j/k/l` | 扩展选区 |
| `w/b` | 按单词扩展 |
| `0/$` | 扩展到行首/行尾 |
| `g/G` | 扩展到文件开头/结尾 |
| `d` | 删除选区（同时复制到剪贴板） |
| `y` | 复制选区到剪贴板 |
| `Esc` | 退出 Visual 模式 |

### Leader 命令（空格 + 按键）

| 按键 | 操作 |
|-----|--------|
| `空格 + s` | 处理并分发块 |
| `空格 + l` | 打开草稿列表 |
| `空格 + nn` | 创建新笔记 |
| `空格 + q` | 退出 |
| `空格 + h` | 切换快捷键提示栏 |
| `空格 + d` | 切换复选框状态（`- [ ]` ↔ `- [x]`） |
| `空格 + mc` | 在当前行插入复选框（`- [ ] `） |

### 列表视图

| 按键 | 操作 |
|-----|--------|
| `j/k` | 上下导航 |
| `Enter/l/i` | 打开选中的笔记 |
| `a` | 归档笔记（草稿视图） |
| `r` | 恢复笔记（归档视图） |
| `d` | 删除笔记（需确认） |
| `n` | 创建新笔记 |
| `A` | 切换到归档视图 |
| `/` 或 `f` | 搜索笔记 |
| `空格` | 切换选择 |
| `Esc` | 返回编辑器 |

## 列表续行

在列表行上按 `o`（Normal 模式）或 `Enter`（Insert 模式）时，列表前缀会自动续接到新行：

- `- [ ] ` / `- [x] ` / `- [X] ` → 新行为 `- [ ] `（始终为未选中）
- `- ` → 新行为 `- `
- `1. ` → 新行为 `2. `（自动递增）
- `1) ` → 新行为 `2) `（自动递增）

**Bullet.vim 行为**：如果当前行仅包含列表前缀而没有文本内容，按 `o` 或 `Enter` 会移除前缀并插入空白行。

缩进（前导空格）会被保留。

## 智能块语法

Kenotex 使用以下模式自动检测块类型：

### 显式标签（最高优先级）
- `:::td` - 强制发送到提醒事项
- `:::cal` - 强制发送到日历
- `:::note` - 强制发送到备忘录

### 自动检测
- `- [ ]` 复选框项目 -> 提醒事项
- 时间表达式（tomorrow、Monday、10am 等）-> 日历
- 中文时间（明天、下周等）-> 日历
- 其他内容 -> 备忘录

### 示例

```markdown
# 会议记录

:::cal 明天早上10点团队站会

- [ ] 准备演示文稿
- [ ] 审查 PR #123
- [ ] 更新文档

:::note 记得询问 Q2 路线图
```

## 配置

配置文件位置：`~/.config/kenotex/config.toml`

完整配置参考请查看 [docs/default.toml](docs/default.toml)（含中英文注释）。

```toml
[general]
theme = "tokyo_night"  # tokyo_night, gruvbox, nord, catppuccin_mocha, catppuccin_macchiato, catppuccin_frappe, catppuccin_latte
leader_key = " "
auto_save_interval_ms = 5000
show_hints = true      # 显示快捷键提示栏
# data_dir = "~/Documents/kenotex-notes"  # 自定义笔记存储路径
file_watch = true       # 检测外部文件更改
file_watch_debounce_ms = 300

[keyboard]
layout = "qwerty"
# 导航键
move_left = "h"
move_down = "j"
move_up = "k"
move_right = "l"
word_forward = "w"
word_backward = "b"
# 插入模式
insert = "i"
insert_append = "a"
# 编辑操作
delete_char = "x"
delete_line = "d"
undo = "u"
redo = "ctrl+r"
yank = "y"
paste_after = "p"
paste_before = "P"
# Leader 命令
leader_process = "s"
leader_list = "l"
leader_new = "nn"
leader_quit = "q"

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

## 架构

项目遵循分层原子架构：

```
src/
├── main.rs                 # L1 入口层
├── coordinator/            # L2 协调层
│   ├── app.rs              # 应用状态（TEA 模式）
│   └── event_dispatcher.rs # 事件路由
├── molecules/              # L3 分子层（业务逻辑）
│   ├── editor/             # Vim 模式、文本缓冲区
│   ├── list/               # 草稿/归档列表
│   ├── config/             # 主题、快捷键
│   └── distribution/       # 块解析器、时间解析器、分发器
├── atoms/                  # L4 原子层（最小单元）
│   ├── widgets/            # UI 组件
│   ├── storage/            # 文件 I/O
│   └── applescript/        # macOS 应用集成
└── types/                  # 数据类型
```

## 依赖

- **ratatui** - 终端 UI 框架
- **crossterm** - 终端处理
- **tokio** - 异步运行时
- **chrono** + **chrono-english** - 日期/时间解析
- **serde** + **toml** - 配置管理
- **notify** + **notify-debouncer-mini** - 文件系统监视（实时重载）
- **regex** - 模式匹配
- **uuid** - 笔记 ID

## 许可证

MIT
