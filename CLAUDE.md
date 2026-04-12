# Omniknight

TUI session multiplexer for AI agents. Like agent-deck / Conductor / Superset but as a Rust terminal app.

## Concept
- **Session** is the primary entity — each session = isolated PTY (shell or agent process)
- Sessions are grouped by **workspace** (collapsible tree in left pane)
- **Workspace** = a directory (future: git worktree) with N sessions
- User launches shells (`t`) or agents (`a`) as sessions
- Status detection: Running ●, Idle ○, Done ✓, Error ✕

## Stack
ratatui 0.30 + crossterm + portable-pty + vt100 + crossbeam-channel

## Layout
```
┌─ Sessions ────────────┐┌─ Active Session ──────────────────────────┐
│ ▼ project-a     [2]   ││ $ claude --task "fix auth"                │
│   ● claude       ◉    ││ Reading src/auth/...                      │
│   ○ shell             ││ Found issue at line 42...                 │
│ ▶ project-b     [1]   ││                                           │
└───────────────────────┘└──────────────────────────────────────────┘
```

## Navigation
- `j/k` — select in session tree or scroll terminal
- `Enter` / `l` — activate session (or spawn shell on empty workspace)
- `Space` — toggle workspace collapse
- `h` / `Esc` — back to session list
- `i` — insert mode (type into PTY)
- `t` — new shell session in workspace
- `a` — spawn agent (dialog: command + title)
- `n` — new workspace (dialog)
- `[` / `]` — cycle sessions within workspace
- `:` — command palette
- `gg/G` — top/bottom, `Ctrl+d/u` — page, `5j` — vim counts

## Testing
```sh
cargo test --test snapshots           # TestBackend + insta
cargo test --test e2e -- --ignored    # PTY harness
```

## Terminal rendering
`vt100` crate parses ANSI from raw PTY bytes → styled ratatui Lines with colors/bold/italic/underline.
