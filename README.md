# envmgr

`envmgr` is a simple and extensible environment manager for managing your command line environment with dotfiles, environment variables, and plugin support.

## Features

- **Environment Management**: Create, switch between, and manage multiple environments
- **Dotfile Management**: Easily manage your dotfiles with version control and environment-specific overrides
- **Environment Variables**: Set and manage environment variables across different projects
- **Plugin System**: Extensible plugin architecture for tool-specific integrations (external plugins)
- **Shell Integration**: Easy integration with Fish, Bash, and Zsh shells

## Built-in Plugins

There are no built-in plugins. Plugins will be discovered/loaded externally in a future iteration. For now, `envmgr plugin list` will show an empty set unless you register plugins programmatically.

## Installation

```bash
# Build from source
cargo build --release

# Install binary
cargo install --path .

# Install shell hooks (auto-apply current env in new shells)
# Detects your shell by default
envmgr install
# Or specify explicitly
envmgr install --shell fish   # fish
envmgr install --shell bash   # bash
envmgr install --shell zsh    # zsh

# Optional: source helper functions (wrapper for `envmgr use`, prompt, alias)
# Fish:
echo 'source /path/to/envmgr/scripts/envmgr.fish' >> ~/.config/fish/config.fish
# Bash/Zsh:
echo 'source /path/to/envmgr/scripts/envmgr.sh' >> ~/.bashrc  # or ~/.zshrc
```

## Quick Start

```bash
# Initialize base configuration
envmgr add base

# Create work environment
envmgr add work

# Configure environment variables (edit opens in $EDITOR)
envmgr edit work

# Add dotfiles to base
cp ~/.gitconfig "${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/base/dotfiles/"
cp ~/.zshrc "${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/base/dotfiles/"

# Create work-specific dotfile override
cp ~/.gitconfig "${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/work/dotfiles/.gitconfig"
# Edit work gitconfig with work email, etc.

# Switch to work environment in the current shell
# Emits shell-appropriate commands (export for bash/zsh, set -gx for fish)
# Bash/Zsh:
eval "$(envmgr use work)"
# Fish:
envmgr use work | source
# Or use the helper functions if sourced:
envmgr_use work

# List environments
envmgr list

# Show current environment
envmgr current
```

## Core Concepts

### Base Configuration
- Manages **dotfiles** (symlinks, templates, overrides)
- Provides a foundation that all environments inherit from
- Located in `${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/base/`

### Environments (Profiles)
- Extend the base config
- Add environment-specific **variables**, **dotfile overrides**, and **tool configs**
- Example: `work` environment overrides `.gitconfig` with work email, sets `GITHUB_TOKEN`

### Plugins
- Extend functionality for specific tools (e.g., `gh`, `tailscale`, `1password`)
- Handle tool-specific quirks (auth flows, config formats, etc.)
- Example: `gh` plugin manages `~/.config/gh/hosts.yml`

## Commands

### Environment Management
```bash
envmgr list                    # List all environments
envmgr current                 # Show current environment
envmgr use <env>              # Activate environment (prints shell commands)
envmgr add <env>              # Create new environment
envmgr add <env> --base <base> # Create environment inheriting from base
envmgr remove <env>           # Delete environment
envmgr edit <env>             # Edit environment config
```

### Shell Integration
```bash
envmgr install                # Install hooks into your shell rc to auto-apply current env
# After this, new shells will automatically apply the env recorded in `${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/current`.
```

### Dotfiles Management
```bash
envmgr dotfiles list          # Show managed dotfiles
envmgr dotfiles link          # Re-link all dotfiles
envmgr dotfiles diff <env>    # Show overrides vs base
```

### Plugin Management
```bash
envmgr plugin list            # List available plugins
envmgr plugin enable <plugin> # Enable plugin for current environment
envmgr plugin disable <plugin> # Disable plugin
envmgr plugin config <plugin> <env> # Configure plugin for environment
```

## Configuration

### Directory Structure
```
${XDG_CONFIG_HOME:-$HOME/.config}/envmgr/
├── base/
│   ├── config.yaml
│   └── dotfiles/
│       ├── .gitconfig
│       ├── .zshrc
│       └── .vimrc
├── work/
│   ├── config.yaml
│   ├── dotfiles/
│   │   └── .gitconfig      # Override
│   └── plugins/
│       ├── gh.yaml
│       └── tailscale.yaml
├── personal/
│   └── config.yaml
└── current                 # Current environment name
```

### Environment Config Example
```yaml
name: work
base: null
env_vars:
  GITHUB_TOKEN: ghp_xxxxxxxxxxxxxxxxxxxx
  AWS_PROFILE: work
plugins:
  gh: {}
  tailscale: {}
```

## Shell Integration Helpers

To get helper functions (wrapper for `envmgr use`, alias, prompt), source the scripts:
- Fish: add `source /path/to/envmgr/scripts/envmgr.fish` to `~/.config/fish/config.fish`
- Bash/Zsh: add `source /path/to/envmgr/scripts/envmgr.sh` to your `~/.bashrc` or `~/.zshrc`

## Examples

See the `examples/` directory for complete configuration examples and usage patterns.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.