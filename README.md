# envmgr

A dotfiles manager on steroids.

## Limitations
- Currently supports only Unix-like systems (Linux, macOS).
- Only fish shell is supported at the moment.

## Features
- Manage your dotfiles with ease.
- Support for multiple environments (e.g., work, personal).
- Easy setup and configuration.
- Backup and restore your dotfiles.

## Installation

```sh
cargo install --git https://github.com/flyinpancake/envmgr.git
```

## Fish shell integration

envmgr emits shell commands that need to be evaluated in the current shell session. For fish, you can wire this up with a small hook. The hook is direnv-like and can auto-apply your environment when you cd.

- One-off for the current session:

```fish
envmgr hook fish | source
```

- Persist it for all future fish sessions by adding the hook to your fish config:

```fish
# Create/append to ~/.config/fish/conf.d/10-envmgr.fish so fish auto-loads it
envmgr hook fish > ~/.config/fish/conf.d/10-envmgr.fish
```

Usage in fish after installing the hook:

- Apply your current environment (prints and evals fish commands). This also happens automatically at the prompt:

```fish
envmgr use
```

- List environments:

```fish
envmgr list
```

Notes:

- The hook defines a fish function named `envmgr` that forwards subcommands to the binary and, for `use` and `switch`, evals the emitted `set`/`set -e` commands so your session updates in-place.
- If you prefer not to install the function, you can still manually eval output when needed: `command envmgr use | source`.


## Roadmap

- [x] Basic dotfile management
- [x] Environment switching
- [x] Fish shell support
- [x] GitHub CLI integration
- [x] 1Password SSH Agent integration
- [x] Tailscale integration
- [ ] Init command with interactive setup
- [ ] Add/remove environments
- [ ] Import / export existing dotfiles
- [ ] Doctor command
- [ ] Plugin system
- [ ] Encrypted files (e.g., using age)
- [ ] Git wrapper for managing encrypted files
- [ ] More integrations (e.g., AWS, GCP, Azure)
- [ ] More shells (zsh, bash)
- [ ] More platforms (Windows / macOS)
- [ ] Performance tests and budgets