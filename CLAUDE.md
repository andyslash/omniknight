# Omniknight

TUI workspace multiplexer for parallel AI agents. Like Conductor.build / Superset.sh but as a terminal app.

## Concept
- **Workspace = git worktree + embedded terminal** (interactive PTY shell)
- User creates workspaces, each gets an isolated terminal
- User launches whatever CLI they want in the terminal (`claude`, `aider`, `sh`)
- No separate "agent" concept — an agent is just a process in the terminal

## Stack
ratatui 0.30 + crossterm + portable-pty + vt100 + crossbeam-channel

## Layout (2 panes)
```
┌─ Workspaces ──────┐┌─ Terminal (project-a) ───────────────────────┐
│ ● project-a  ◉    ││ $ claude --task "fix auth"                   │
│   project-b       ││ Reading src/auth/...                         │
│                   ││                                              │
└───────────────────┘└──────────────────────────────────────────────┘
```

## Navigation
- `j/k` — select workspace (left pane) or scroll terminal (right pane)
- `l` / `Enter` / `Tab` — focus terminal (auto-spawns shell on first use)
- `h` / `Esc` / `BackTab` — focus workspaces
- `i` — insert mode (type into terminal PTY)
- `Esc` — back to normal mode
- `n` — new workspace dialog
- `:` — command palette
- `gg/G` — top/bottom, `Ctrl+d/u` — page, `5j` — vim counts

## Testing
```sh
cargo test --test snapshots           # TestBackend + insta (no TTY)
cargo test --test e2e -- --ignored    # PTY harness (spawns real binary)
```

## Terminal rendering
Uses `vt100` crate to parse ANSI escape sequences from PTY output before rendering in ratatui. Raw bytes → vt100::Parser → clean text lines.
