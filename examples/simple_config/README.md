envmgr example configurations

This directory contains example configuration structures you can copy into your own ~/.config/envmgr directory. On Linux, envmgr stores config in ~/.config/envmgr by default.

Layout:
- base/: the base environment applied to all envs
- environments/<env_key>/: specific environments that overlay base

How to use:
1) Create your config directory: ~/.config/envmgr
2) Copy the base and any desired environments from these examples into ~/.config/envmgr
3) Review and adjust config.yaml values
4) In fish shell, load the hook once per session: envmgr hook fish | source
5) List envs: envmgr list
6) Switch envs: envmgr switch <env_key>
7) Apply variables to current shell: envmgr use (usually auto-run by the hook)

Notes:
- Files placed under base/files or environments/<key>/files are linked into $HOME preserving paths relative to the files directory. For example, base/files/.config/myapp/config.toml will be linked to ~/.config/myapp/config.toml.
- Only fish is currently supported for shell integration.
- Integrations like 1Password SSH Agent, GitHub CLI, and Tailscale are optional.
