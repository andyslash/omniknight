# Omniknight

A keyboard-first TUI cockpit for orchestrating AI agents and terminal sessions within workspaces.

```
 [NORMAL]  Omniknight  |  dev-project  |  3 workspaces
┌─ ▸ Sessions ─────────┐┌─ Terminal ─────────────────────────┐
│ ▼ dev-project [2]     ││                                    │
│   ● claude-agent ⚡ ◉ ││  $ cargo test                      │
│   ○ zsh               ││  running 7 tests ...               │
│ ▶ research [1]        ││  test workspace::new ... ok         │
│ ▶ deploy [1]          ││  test session::spawn ... ok         │
│                       ││  test ipc::bus ... ok               │
│                       ││                                    │
│                       ││  7 passed; 0 failed                │
│                       ││                                    │
└───────────────────────┘└────────────────────────────────────┘
 j/k:select  Enter:activate  n:workspace  t:shell  a:agent  q:quit
```

## Features

- **Workspaces** — session groups with a collapsible tree view
- **Multiple sessions** — shells and AI agents side by side, switchable tabs
- **Terminal PTY** — full VT100 rendering (colors, bold, italic) via `portable-pty` + `vt100`
- **Vim-style** — `j/k` navigation, normal/insert modes, count prefixes (`5j`), leader key (`:`)
- **Command palette** — fuzzy search (nucleo) across all commands
- **Event bus** — decoupled IPC architecture over crossbeam channels
- **TOML config** — tick rate, agent limits, keybinds, theme

## Stack

- **Rust** (edition 2024)
- **ratatui** + **crossterm** for TUI rendering
- **portable-pty** for pseudo-terminals
- **vt100** for ANSI parsing
- **tokio** for async runtime
- **nucleo** for fuzzy matching
- **insta** for snapshot testing

## Getting started

```bash
cargo run
```

## Keybindings

| Mode   | Key          | Action                     |
|--------|-------------|----------------------------|
| Normal | `j` / `k`  | Navigate up/down           |
| Normal | `Enter`     | Activate workspace/session |
| Normal | `Space`     | Collapse/expand workspace  |
| Normal | `n`         | New workspace              |
| Normal | `t`         | New shell session          |
| Normal | `a`         | New agent session          |
| Normal | `i`         | Enter insert mode          |
| Normal | `[` / `]`   | Switch tabs                |
| Normal | `:`         | Command palette            |
| Normal | `q`         | Quit                       |
| Insert | `Esc`       | Back to normal mode        |

## Configuration

```toml
# config/default.toml
[general]
tick_rate_ms = 100
max_agents = 20

[agent.defaults]
command = "claude"
shell = "/bin/zsh"

[keybinds]
leader = ":"
```

## Contributing

Contributions are welcome! Here's how to get started:

1. **Fork** the repository
2. **Create a branch** for your feature or fix (`git checkout -b feat/my-feature`)
3. **Make your changes** — keep commits small and focused
4. **Run tests** before submitting: `cargo test`
5. **Open a pull request** with a clear description of what changed and why

### Guidelines

- Follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, etc.)
- Keep PRs focused — one feature or fix per PR
- Add tests for new functionality when possible
- Run `cargo clippy` and fix warnings before submitting

### Ideas for contribution

- Multi-pane terminal splits
- Agent protocol integrations (MCP, A2A)
- Session persistence / restore
- Custom themes
- Plugin system for custom commands

## License

This project is licensed under the [MIT License](LICENSE).
