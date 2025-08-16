# Example envmgr Configuration

This directory shows an example configuration structure for envmgr.

## Directory Structure

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
│   │   └── .gitconfig      # Override for work
│   └── plugins/
│       ├── gh.yaml
│       └── tailscale.yaml
├── personal/
│   ├── config.yaml
│   ├── dotfiles/
│   │   └── .gitconfig      # Override for personal
│   └── plugins/
│       └── gh.yaml
└── current                 # Contains name of current environment
```

## Example Environment Configuration

### work/config.yaml
```yaml
name: work
base: null
env_vars:
  GITHUB_TOKEN: ghp_xxxxxxxxxxxxxxxxxxxx
  AWS_PROFILE: work
  NODE_ENV: development
plugins:
  gh: {}
  tailscale: {}
```

### work/plugins/gh.yaml
```yaml
token: ghp_work_token_here
user: myworkusername
```

### work/plugins/tailscale.yaml
```yaml
authkey: tskey-auth-xxxxxxxxxxxx
tailnet: company.ts.net
```

## Example Dotfiles

### base/dotfiles/.gitconfig
```ini
[user]
    name = Your Name
    email = your.email@example.com

[core]
    editor = vim
    autocrlf = input

[push]
    default = simple
```

### work/dotfiles/.gitconfig
```ini
[user]
    name = Your Name
    email = your.name@company.com

[core]
    editor = vim
    autocrlf = input

[push]
    default = simple

[url "git@github.com:company/"]
    insteadOf = https://github.com/company/
```

## Usage Examples

```bash
# Create environments
envmgr add work
envmgr add personal

# Configure plugins
envmgr plugin config gh work
envmgr plugin config gh personal

# Switch environments
eval $(envmgr use work)      # Sets env vars and runs plugins
eval $(envmgr use personal)  # Switches dotfiles and plugins

# List environments
envmgr list

# Show current environment
envmgr current

# Manage dotfiles
envmgr dotfiles list
envmgr dotfiles diff work
```
