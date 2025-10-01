use std::path::PathBuf;

pub mod gh_cli;
pub mod one_password_ssh_agent;
pub mod tailscale;

#[expect(dead_code)]
pub struct OnUsePluginResult {
    env_vars: Vec<(String, String)>,
}

#[expect(dead_code)]
#[derive(Default)]
pub struct OnSwitchToPluginResult {
    files_to_link: Vec<(PathBuf, PathBuf)>,
}
