# Omniknight

Cockpit TUI keyboard-first pour orchestrer des agents IA et des sessions terminal dans des workspaces.

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

## Fonctionnalites

- **Workspaces** — groupes de sessions avec arbre collapsible
- **Sessions multiples** — shells et agents IA cote a cote, onglets switchables
- **Terminal PTY** — rendu VT100 complet (couleurs, bold, italic) via `portable-pty` + `vt100`
- **Vim-style** — navigation `j/k`, mode normal/insert, compteurs (`5j`), leader key (`:`)
- **Command palette** — recherche fuzzy (nucleo) pour les commandes
- **Event bus** — architecture decouplée IPC par canaux crossbeam
- **Config TOML** — tick rate, limites agents, keybinds, theme

## Stack

- **Rust** (edition 2024)
- **ratatui** + **crossterm** pour le rendu TUI
- **portable-pty** pour les pseudo-terminaux
- **vt100** pour le parsing ANSI
- **tokio** pour l'async
- **nucleo** pour la recherche fuzzy
- **insta** pour les snapshot tests

## Lancer

```bash
cargo run
```

## Raccourcis

| Mode   | Touche       | Action                     |
|--------|-------------|----------------------------|
| Normal | `j` / `k`  | Naviguer haut/bas          |
| Normal | `Enter`     | Activer workspace/session  |
| Normal | `Space`     | Collapse/expand workspace  |
| Normal | `n`         | Nouveau workspace          |
| Normal | `t`         | Nouvelle session shell     |
| Normal | `a`         | Nouvelle session agent     |
| Normal | `i`         | Mode insert (terminal)     |
| Normal | `[` / `]`   | Changer d'onglet           |
| Normal | `:`         | Command palette            |
| Normal | `q`         | Quitter                    |
| Insert | `Esc`       | Retour mode normal         |

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

## Licence

MIT
