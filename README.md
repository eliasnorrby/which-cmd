# which-cmd

A which-key inspired command builder for the terminal. Discover and compose shell commands interactively with a visual TUI interface.

[![Crates.io](https://img.shields.io/crates/v/which-cmd.svg)](https://crates.io/crates/which-cmd)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`which-cmd` provides a terminal UI for building commands interactively by displaying available key bindings and letting you discover commands by pressing keys sequentially. Similar to [which-key](https://github.com/folke/which-key.nvim) for Neovim or [which-key](https://github.com/justbur/emacs-which-key) for Emacs, but for your shell.

Instead of memorizing complex command flags or searching through man pages, define your commonly used commands in a configuration file and navigate them visually.

## Features

- **Interactive TUI**: Visual command builder that shows available options as you type
- **Fuzzy Search**: Quick search across all configured commands with `/`
- **Flexible Configuration**: YAML-based config for defining command trees
- **Shell Integration**: Works with bash, zsh, and fish
- **Advanced Patterns**:
  - **Loop nodes**: Accumulate multiple flags (e.g., `curl -H "..." -X POST`)
  - **Choice nodes**: Fuzzy-select from predefined options
  - **Input nodes**: Prompt for dynamic values
  - **Anchor nodes**: Reset command context mid-flow
  - **Fleeting nodes**: Group multi-part selections that pop together

## Installation

### From crates.io (requires Rust)

```bash
cargo install which-cmd
```

### From source

```bash
git clone https://github.com/eliasnorrby/which-cmd
cd which-cmd
cargo install --path .
```

### Binary releases

Download precompiled binaries from the [releases page](https://github.com/eliasnorrby/which-cmd/releases) (coming soon).

## Quick Start

1. **Install** using one of the methods above

2. **Create a configuration file** at `~/.config/which-cmd/commands.yml`:

```yaml
keys:
  - key: g
    value: git
    keys:
      - key: s
        value: status
      - key: d
        value: diff
      - key: c
        value: commit
        keys:
          - key: m
            value: -m
```

3. **Set up shell integration** (optional but recommended):

```bash
# For bash, add to ~/.bashrc:
eval "$(which-cmd integration bash)"

# For zsh, add to ~/.zshrc:
eval "$(which-cmd integration zsh)"

# For fish, add to ~/.config/fish/config.fish:
which-cmd integration fish | source
```

4. **Use it**:
   - With shell integration: Press your configured keybinding (default varies by shell)
   - Standalone: Run `which-cmd build`

## Usage

### Commands

```bash
# Build a command interactively (launches TUI)
which-cmd build

# Build with immediate execution (execute when complete)
which-cmd build --immediate

# Get the last built command
which-cmd get

# Generate shell integration code
which-cmd integration <bash|zsh|fish>

# Validate configuration and check setup
which-cmd doctor

# Get TUI height (used by shell integrations)
which-cmd height
```

### TUI Controls

- **Character keys**: Navigate deeper into the command tree
- **`/`**: Fuzzy search across all commands
- **Backspace**: Go back one level
- **Enter**: Execute/return the current command
- **Esc**: Exit without executing

## Configuration

Configuration files are located at `$XDG_CONFIG_HOME/which-cmd/commands.yml` (typically `~/.config/which-cmd/commands.yml`).

### Basic Example

```yaml
keys:
  - key: d
    value: docker
    keys:
      - key: p
        name: pull
        value: pull
      - key: r
        name: run
        value: run
      - key: i
        name: images
        value: image ls
      - key: c
        name: containers
        value: container ls
```

Pressing `d` then `i` produces: `docker image ls`

### Advanced Features

#### Loop Nodes

Allow repeated selections to accumulate flags:

```yaml
keys:
  - key: c
    value: curl
    loop: true
    keys:
      - key: H
        name: header
        value: -H
      - key: X
        name: method
        value: -X POST
      - key: d
        name: data
        value: -d
```

You can now build `curl -H "..." -X POST -d "..."` by pressing `c`, `H`, `X`, `d`, then Enter.

#### Anchor Nodes

Reset the command context to start fresh:

```yaml
keys:
  - key: g
    value: git
    keys:
      - key: h
        name: GitHub CLI
        value: gh
        anchor: true
        keys:
          - key: p
            value: pr
```

Pressing `g` then `h` then `p` gives you `gh pr` (not `git gh pr`). The anchor resets the command.

#### Fleeting Nodes

Nodes that group multi-part selections - when you backspace, the entire group is removed:

```yaml
keys:
  - key: f
    name: find
    value: find .
    loop: true
    keys:
      - key: t
        value: -type
        fleeting: true
        keys:
          - key: f
            name: file
            value: f
          - key: d
            name: directory
            value: d
```

After building `find . -type f`, pressing backspace removes both `f` and `-type` together since they're a logical unit.

#### Immediate Execution

Execute without waiting for Enter:

```yaml
keys:
  - key: g
    value: git
    keys:
      - key: s
        value: status
        immediate: true
```

Pressing `g` then `s` immediately executes `git status`.

### Schema

See [schema.yml](schema.yml) for the complete configuration schema.

## Examples

See [fixtures/commands.yml](fixtures/commands.yml) for a comprehensive example configuration with git, docker, cargo, and more.

## Shell Integration

The shell integration allows you to invoke `which-cmd` with a keybinding and have the built command inserted into your current command line.

Integration code handles:
- Launching the TUI
- Capturing the built command
- Inserting it at the cursor position
- Preserving the command line state

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Locally

```bash
cargo run -- build
```

## How It Works

`which-cmd` uses a tree-based configuration where each node represents a key binding. When you press a key:

1. The corresponding node is added to your current path
2. The TUI displays available child nodes
3. When you reach a leaf or press Enter, all node values are concatenated to form the final command

The architecture is described in detail in [CLAUDE.md](CLAUDE.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Inspiration

- [which-key.nvim](https://github.com/folke/which-key.nvim) - Neovim plugin
- [emacs-which-key](https://github.com/justbur/emacs-which-key) - Emacs package

## Changelog

See [GitHub releases](https://github.com/eliasnorrby/which-cmd/releases) for version history.
