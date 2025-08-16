#!/bin/bash

# Example setup script for envmgr
# This demonstrates the complete workflow

echo "Setting up envmgr example..."

# Path to envmgr binary
ENVMGR="./target/release/envmgr"

# Get the config directory (XDG compliant)
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/envmgr"

# Create base dotfiles
mkdir -p "$CONFIG_DIR/base/dotfiles"

# Create a simple .gitconfig for base
cat > "$CONFIG_DIR/base/dotfiles/.gitconfig" << 'EOF'
[user]
    name = Your Name
    email = your.personal@example.com

[core]
    editor = vim
    autocrlf = input

[push]
    default = simple

[color]
    ui = auto
EOF

# Create a simple .zshrc for base
cat > "$CONFIG_DIR/base/dotfiles/.zshrc" << 'EOF'
# Base .zshrc configuration
export EDITOR=vim
export LANG=en_US.UTF-8

# Basic aliases
alias ll='ls -la'
alias la='ls -A'
alias l='ls -CF'

# Show current envmgr environment in prompt
if [ -n "$ENVMGR_CURRENT_ENV" ]; then
    PS1="($ENVMGR_CURRENT_ENV) $PS1"
fi

echo "Base environment loaded"
EOF

# Create work environment if it doesn't exist
$ENVMGR add work 2>/dev/null || true

# Create work-specific .gitconfig
mkdir -p "$CONFIG_DIR/work/dotfiles"
cat > "$CONFIG_DIR/work/dotfiles/.gitconfig" << 'EOF'
[user]
    name = Your Name
    email = your.name@company.com

[core]
    editor = vim
    autocrlf = input

[push]
    default = simple

[color]
    ui = auto

[url "git@github.com:company/"]
    insteadOf = https://github.com/company/

[credential]
    helper = store
EOF

# Update work environment config with environment variables
cat > "$CONFIG_DIR/work/config.yaml" << 'EOF'
name: work
base: null
env_vars:
  GITHUB_TOKEN: ghp_work_token_placeholder
  AWS_PROFILE: work
  NODE_ENV: development
  COMPANY_API_URL: https://api.company.com
plugins:
  gh: {}
EOF

# Create personal environment if it doesn't exist
$ENVMGR add personal 2>/dev/null || true

# Update personal environment config
cat > "$CONFIG_DIR/personal/config.yaml" << 'EOF'
name: personal
base: null
env_vars:
  GITHUB_TOKEN: ghp_personal_token_placeholder
  AWS_PROFILE: personal
plugins:
  gh: {}
EOF

echo ""
echo "Example setup complete! Try these commands:"
echo ""
echo "# List environments:"
echo "$ENVMGR list"
echo ""
echo "# Switch to work environment:"
echo "eval \$($ENVMGR use work)"
echo ""
echo "# Switch to personal environment:"
echo "eval \$($ENVMGR use personal)"
echo ""
echo "# Check current environment:"
echo "$ENVMGR current"
echo ""
echo "# List dotfiles:"
echo "$ENVMGR dotfiles list"
echo ""
echo "# Show differences:"
echo "$ENVMGR dotfiles diff work"
echo ""
echo "# List plugins:"
echo "$ENVMGR plugin list"
echo ""
echo "Configuration is stored in: $CONFIG_DIR"
echo "Note: The dotfiles will be symlinked to your home directory when you use an environment."
echo "Be careful as this will overwrite existing files!"
