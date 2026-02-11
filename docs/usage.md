# Usage Guide

## Smart Block Syntax

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

## List Handling

Within `:::td` blocks:
- Detects: `-`, `*`, `- [ ]`, `- []` at line start
- Creates a separate reminder for each list item
- Strips list prefix before creating the reminder

## Time Expressions

- `@time` syntax: `@tomorrow`, `@9pm`, `@Monday`, `@明天早上8点`, `@下周一`
- Works in both `:::td` and `:::cal` blocks
- Editor highlights `@time` in bold accent color for visual feedback

## Warnings

- Unclosed tags show a warning with line number
- Empty blocks (no content between tags) are ignored

## List Continuation

When pressing `o` (Normal mode) or `Enter` (Insert mode) on a list line, the list prefix is automatically continued on the new line:

- `- [ ] ` / `- [x] ` / `- [X] ` → new line with `- [ ] ` (always unchecked)
- `- ` → new line with `- `
- `* ` → new line with `* `
- `1. ` → new line with `2. ` (auto-incrementing)
- `1) ` → new line with `2) ` (auto-incrementing)

**Bullet.vim behavior:** If the current line contains only a list prefix with no text after it, pressing `o` or `Enter` removes the prefix and inserts a blank line instead.

Indentation (leading whitespace) is preserved.
