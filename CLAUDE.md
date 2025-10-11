# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`which-cmd` is a command-line tool that provides a which-key style interface for building commands interactively. It displays available key bindings in a TUI (Terminal User Interface), allowing users to discover and compose commands by pressing keys sequentially.

## Core Architecture

### Node-Based Command Tree

The entire command structure is represented as a tree of `Node` objects (src/node.rs):
- **Node**: The fundamental building block representing a key binding or command fragment
  - `id`: Unique identifier constructed by concatenating parent keys (e.g., "gs" for git status)
  - `key`: Single character that triggers this node
  - `value`: The command fragment to insert when this node is selected
  - `name`: Human-readable description shown in the TUI
  - Node types:
    - **Leaf nodes**: Terminal nodes that complete a command
    - **Branch nodes**: Have child `keys` for further selection
    - **Choice nodes**: Present a fuzzy-select menu via `choices` field
    - **Input nodes**: Prompt for text/number input via `input` field
  - Special flags:
    - `immediate`: Execute command immediately without waiting for Enter
    - `fleeting`: Groups multi-part selections that should pop together (e.g., `-type` and `f` in `find -type f`)
    - `anchor`: Marks a reset point in the command tree (resets the command prefix)
    - `loop`: Allows repeated selection of child keys (e.g., `curl -H ... -X POST`)
    - `repeatable`: Allows the same key to be selected multiple times in a loop

### Configuration Loading (src/config.rs)

- Configuration is loaded from YAML files stored in XDG config directories
- The config file contains a tree structure of keys/commands
- During parsing, the config system:
  1. Deserializes YAML into Node tree structure
  2. Assigns unique IDs to each node by traversing the tree
  3. Validates that no duplicate keys exist at the same level
  4. Enforces that nodes have only ONE action type (keys, choices, or input)

Example config structure (see fixtures/commands.yml for full example):
```yaml
keys:
  - key: g
    value: git
    keys:
      - key: s
        value: status
```

### TUI Interaction Loop (src/tui.rs)

The main TUI loop in `run_tui()`:
1. Displays current command being built at top
2. Shows available keys in a sorted, multi-column table layout
3. Waits for user input:
   - Character keys: Navigate deeper into the command tree
   - `/`: Trigger fuzzy search across all available commands
   - Backspace: Pop the last selection from the path
   - Enter: Execute the current command
   - Esc: Exit without executing
4. Maintains a `path` vector tracking selected nodes
5. Uses `loop_node_index` to track loop contexts for repeated selections
6. When a leaf node is reached (or Enter is pressed), calls `compose_command()` to build the final command string

### Command Composition (src/path.rs)

The `compose_command()` function builds the final command by:
- Concatenating the `value` fields from all nodes in the path
- Handling anchor nodes (which reset the command prefix)
- Joining with spaces to form the executable command

### Search Functionality (src/search.rs)

- `get_search_options()`: Recursively flattens the entire command tree
- Returns all possible command paths with their IDs
- Formatted for fuzzy-select using dialoguer
- When selected, rebuilds the path by parsing the node ID character by character

## Commands

### Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run clippy (linter)
cargo clippy

# Build and run
cargo run -- <subcommand>
```

### Application Subcommands

```bash
# Build a command interactively (main TUI mode)
which-cmd build

# Build with immediate execution flag
which-cmd build --immediate

# Retrieve the last built command
which-cmd get

# Generate shell integration code
which-cmd integration <shell>  # shell: bash, zsh, fish

# Troubleshoot configuration
which-cmd doctor

# Get TUI height (for shell integrations)
which-cmd height
```

### Configuration

- Config location: `$XDG_CONFIG_HOME/which-cmd/commands.yml` (typically `~/.config/which-cmd/commands.yml`)
- Schema defined in schema.yml
- Validation via `doctor` command

## Key Patterns

### Adding New Node Features

When adding new node properties:
1. Add field to `Node` struct in src/node.rs
2. Update custom `Deserialize` implementation in `NodeHelper`
3. Update tests in src/config.rs to cover new behavior
4. Handle the new property in src/tui.rs display/interaction logic

### Testing Configuration Parsing

All config parsing logic has comprehensive tests in src/config.rs. When modifying config behavior, add corresponding tests following the pattern of `test_config_parsing_*` functions.
