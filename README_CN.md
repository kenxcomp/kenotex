# Kenotex

一款 Vim 风格的终端笔记应用，能够智能地将内容分发到 Apple 提醒事项、日历和备忘录应用。

## 功能特性

- **Vim 风格模态编辑**：完整支持 Normal、Insert、Visual 和 Search 模式
- **智能块检测**：基于标签和模式自动识别内容类型
- **多应用分发**：将内容发送到 Apple 提醒事项、日历、备忘录、Bear 或 Obsidian
- **主题支持**：Tokyo Night、Gruvbox 和 Nord 主题
- **Markdown 存储**：所有笔记以 markdown 文件形式存储在 `~/.config/kenotex/drafts/`
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
| `o` | 在下方插入新行 |
| `O` | 在上方插入新行 |
| `v` | 进入 Visual 模式 |
| `h/j/k/l` | 导航（左/下/上/右） |
| `w/b` | 向前/向后移动一个单词 |
| `0/$` | 行首/行尾 |
| `g/G` | 文件开头/结尾 |
| `x` | 删除字符 |
| `d` | 删除行 |
| `T` | 切换主题 |
| `/` 或 `f` | 进入搜索模式 |
| `Ctrl+G` | 在外部编辑器中打开缓冲区（`$VISUAL` / `$EDITOR` / `vi`） |
| `Esc` | 返回 Normal 模式 |
| `Ctrl+C` 或 `Ctrl+Q` | 退出 |

### Leader 命令（空格 + 按键）

| 按键 | 操作 |
|-----|--------|
| `空格 + s` | 处理并分发块 |
| `空格 + l` | 打开草稿列表 |
| `空格 + n` | 创建新笔记 |
| `空格 + w` | 保存当前笔记 |
| `空格 + q` | 退出 |
| `空格 + th` | 切换快捷键提示栏 |

### 列表视图

| 按键 | 操作 |
|-----|--------|
| `j/k` | 上下导航 |
| `Enter/l/i` | 打开选中的笔记 |
| `a` | 归档笔记（草稿视图） |
| `r` | 恢复笔记（归档视图） |
| `d` | 删除笔记 |
| `n` | 创建新笔记 |
| `A` | 切换到归档视图 |
| `/` 或 `f` | 搜索笔记 |
| `空格` | 切换选择 |
| `Esc` | 返回编辑器 |

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
theme = "tokyo_night"  # tokyo_night, gruvbox, nord
leader_key = " "
auto_save_interval_ms = 5000
show_hints = true      # 显示快捷键提示栏

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
# Leader 命令
leader_process = "s"
leader_list = "l"
leader_new = "n"
leader_save = "w"
leader_quit = "q"

[destinations.reminders]
app = "apple"
# list = "工作"

[destinations.calendar]
app = "apple"
# calendar_name = "个人"

[destinations.notes]
app = "apple_notes"  # apple_notes, bear, obsidian
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
│   └── distribution/       # 块解析器、时间解析器
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
- **regex** - 模式匹配
- **uuid** - 笔记 ID

## 许可证

MIT
