# which-cmd

Stop memorizing commands. `which-cmd` gives you a [which-key](https://github.com/folke/which-key.nvim)-style TUI for your terminal — press a key, see what's available, press another, and watch your command build itself. Define your own tree of key bindings in a simple YAML file and navigate it interactively. It's muscle memory without the memorization.

## How it works

You bind `wcmd` to a hotkey in your shell. When triggered, a panel appears showing your available keys. Press `g` and the panel updates to show git subcommands. Press `s` and `git status` lands on your command line. The deeper you go, the more specific the command gets — all without typing a single word.

```
┌─ git ──────────────────────────────────┐
│                                        │
│  b  branch     d  diff     R  reset    │
│  c  checkout   h  GitHub   r  rebase   │
│  s  status                             │
│                                        │
└────────────────────────────────────────┘
```

## Installation

Build from source (requires [Rust](https://rustup.rs/)):

```bash
cargo install --path .
```

Then add the shell integration to your shell config (see [Shell integration](#shell-integration)).

## Configuration

Commands are defined in `~/.config/which-cmd/commands.yml` as a tree of nodes. Each node maps a single key to a command fragment:

```yaml
keys:
  - key: g
    value: git
    keys:
      - key: s
        value: status
      - key: c
        value: checkout
      - key: d
        value: diff
      - key: r
        value: rebase
        keys:
          - key: i
            name: interactive
            value: -i
          - key: c
            name: continue
            value: --continue
          - key: a
            name: abort
            value: --abort
```

Pressing `g` `r` `i` produces `git rebase -i`.

The `name` field provides a human-readable label in the TUI. When omitted, it defaults to `value`.

Run `wcmd doctor` to validate your configuration.

### Project-local configuration

You can add a `.wcmdrc.yml` (or `.wcmdrc.yaml` / `.wcmdrc`) file in a project directory to define project-specific commands. These appear under a `.` prefix in the TUI.

## Features

### Loop nodes

Some commands accept multiple flags. Mark a node as `loop: true` and its children can be selected repeatedly:

```yaml
- key: l
  value: ls
  loop: true
  keys:
    - key: l
      name: long
      value: -l
    - key: a
      name: all
      value: -a
    - key: r
      name: recursive
      value: -R
```

Press `l` `l` `a` to get `ls -l -a`. Press Enter when done.

### Choices

Present a fuzzy-searchable list of static options:

```yaml
- key: e
  name: edit config
  value: vim
  choices:
    - ~/.bashrc
    - ~/.zshrc
    - ~/.config/which-cmd/commands.yml
```

### Dynamic choices

Generate options from a shell command at runtime:

```yaml
- key: b
  name: checkout branch
  value: git checkout
  choices_command: "git branch --format='%(refname:short)'"
```

### Input

Prompt for freeform text or numeric input:

```yaml
- key: m
  name: commit message
  value: git commit -m
  input: Text
```

### Fleeting nodes

Group related command fragments so they're removed together on backspace. Useful for multi-part flags:

```yaml
- key: h
  name: header
  value: -H
  fleeting: true
  keys:
    - key: c
      name: ContentType
      value: '"ContentType:'
      fleeting: true
      keys:
        - key: a
          name: application/json
          value: 'application/json"'
```

Pressing backspace once removes the entire `-H "ContentType: application/json"` group instead of just the last fragment.

### Anchor nodes

Switch the command prefix mid-navigation. An anchor discards everything before it:

```yaml
- key: g
  value: git
  keys:
    - key: h
      name: GitHub
      value: gh
      anchor: true
      keys:
        - key: b
          value: browse
```

Pressing `g` `h` `b` produces `gh browse`, not `git gh browse`.

### Conditional visibility

Show nodes only when a condition is met:

```yaml
- key: c
  value: cargo
  when:
    file_exists: Cargo.toml

- key: k
  value: kubectl
  when:
    command_exists: kubectl
```

### Immediate execution

Skip the Enter key — execute the command as soon as a leaf node is reached:

```yaml
- key: s
  value: status
  immediate: true
```

### Fuzzy search

Press `/` at any point to fuzzy-search across all available commands in the tree and jump directly to one.

## Shell integration

Generate shell integration code with `wcmd integration <shell>`.

**Zsh:**

```bash
eval "$(wcmd integration zsh)"
```

This binds `Ctrl+P` to open the TUI. The result is inserted into your current command line.

**Zsh + tmux** and **Bash + tmux** integrations are also available, opening `wcmd` in a tmux popup. These bind to the Space key when the command line is empty:

```bash
# zsh + tmux
eval "$(wcmd integration zsh-tmux)"

# bash + tmux
eval "$(wcmd integration bash-tmux)"
```

## Node reference

| Field | Type | Description |
|---|---|---|
| `key` | string | Single character trigger (required) |
| `value` | string | Command fragment to insert |
| `name` | string | Label shown in the TUI (defaults to `value`) |
| `keys` | list | Child nodes for deeper navigation |
| `choices` | list | Static fuzzy-select options |
| `choices_command` | string | Shell command to generate fuzzy-select options |
| `input` | `Text` \| `Number` | Prompt for user input |
| `loop` | bool | Allow repeated selection of children |
| `repeatable` | bool | Allow same key to be selected multiple times in a loop |
| `fleeting` | bool | Remove together with adjacent fleeting nodes on backspace |
| `anchor` | bool | Discard previous command fragments |
| `immediate` | bool | Execute without pressing Enter |
| `when` | object | Conditional visibility (`file_exists` or `command_exists`) |

A node must have at most one of `keys`, `choices`, `choices_command`, or `input`.

## What this tool is not

- **Not a shell replacement.** `which-cmd` builds commands and hands them to your shell. It doesn't execute anything itself (unless you use `immediate` mode).
- **Not an alias manager.** Aliases are static shortcuts. `which-cmd` is a composable, interactive tree — you discover and build commands as you go, including flags and arguments you use less often.
- **Not a command history tool.** It doesn't learn from your usage or suggest past commands. Your command tree is explicitly defined and predictable.
- **Not a scripting tool.** It's designed for interactive use at the terminal, not for automation or piping.

## Contributing

Contributions are welcome! The project is pre-1.0, so things may shift, but bug fixes, documentation improvements, and well-motivated features are appreciated.

```bash
cargo build        # build
cargo test         # run tests
cargo clippy       # lint
cargo fmt          # format (always run before committing)
```

If you're adding a new node property, the typical path is:

1. Add the field to the `Node` struct in `src/node.rs`
2. Update the deserialization logic in `NodeHelper`
3. Handle the new property in `src/tui.rs`
4. Add tests in `src/config.rs`

Please open an issue before starting work on larger changes so we can discuss the approach.

## License

[MIT](LICENSE)
