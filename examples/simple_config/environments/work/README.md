Work environment

- Directory: ~/.config/envmgr/environments/work
- Config:    ~/.config/envmgr/environments/work/config.yaml
- Files:     ~/.config/envmgr/environments/work/files

Overlays the base environment. Add work-specific variables, GitHub CLI user mapping, tailscale tailnet, and files.

Example:
- env var: AWS_PROFILE=work
- gh_cli: default user for github.com
- tailscale: switch to your work tailnet
