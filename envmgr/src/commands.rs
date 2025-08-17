use anyhow::{Context, Result};
use std::process::Command;

use crate::config::EnvMgrConfig;
use crate::environment::EnvironmentManager;
use crate::shell::detect_shell;

/// List all environments
pub async fn list_environments(config: &EnvMgrConfig) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.list_environments().await
}

/// Show current environment
pub async fn show_current(config: &EnvMgrConfig) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.show_current()
}

/// Use/activate an environment
pub async fn use_environment(config: &EnvMgrConfig, name: &str) -> Result<()> {
    let mut manager = EnvironmentManager::new(config.clone());
    manager.use_environment(name).await
}

/// Add a new environment
pub async fn add_environment(config: &EnvMgrConfig, name: &str, base: Option<&str>) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.create_environment(name, base).await
}

/// Remove an environment
pub async fn remove_environment(config: &EnvMgrConfig, name: &str) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.remove_environment(name).await
}

/// Edit environment configuration
pub async fn edit_environment(config: &EnvMgrConfig, name: &str) -> Result<()> {
    if !config.env_exists(name) {
        anyhow::bail!("Environment '{}' does not exist", name);
    }

    let config_file = config.env_dir(name).join("config.yaml");

    // Use the user's preferred editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    let status = Command::new(&editor).arg(&config_file).status()?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    println!("Environment '{}' configuration updated", name);
    Ok(())
}

/// List managed dotfiles
pub async fn list_dotfiles(config: &EnvMgrConfig) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.dotfile_manager.list_dotfiles().await
}

/// Re-link all dotfiles
pub async fn link_dotfiles(config: &EnvMgrConfig) -> Result<()> {
    let mut manager = EnvironmentManager::new(config.clone());
    manager.dotfile_manager.relink_dotfiles().await
}

/// Show dotfile differences
pub async fn diff_dotfiles(config: &EnvMgrConfig, env: &str) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.dotfile_manager.diff_environments(env).await
}

/// List available plugins
pub async fn list_plugins(config: &EnvMgrConfig) -> Result<()> {
    let manager = EnvironmentManager::new(config.clone());
    manager.plugin_manager.list_plugins().await
}

/// Enable a plugin
pub async fn enable_plugin(config: &EnvMgrConfig, name: &str) -> Result<()> {
    // First, check if we have a current environment
    match &config.current_env {
        Some(env_name) => {
            let manager = EnvironmentManager::new(config.clone());
            manager.plugin_manager.enable_plugin(name, env_name).await
        }
        None => {
            println!("No environment is currently active. Please specify an environment:");
            println!("  envmgr plugin config {} <environment>", name);
            Ok(())
        }
    }
}

/// Disable a plugin
pub async fn disable_plugin(config: &EnvMgrConfig, name: &str) -> Result<()> {
    // First, check if we have a current environment
    match &config.current_env {
        Some(env_name) => {
            let manager = EnvironmentManager::new(config.clone());
            manager.plugin_manager.disable_plugin(name, env_name).await
        }
        None => {
            println!("No environment is currently active. Please specify an environment:");
            println!("  envmgr plugin config {} <environment>", name);
            Ok(())
        }
    }
}

/// Configure a plugin for an environment
pub async fn configure_plugin(config: &EnvMgrConfig, plugin: &str, env: &str) -> Result<()> {
    if !config.env_exists(env) {
        anyhow::bail!("Environment '{}' does not exist", env);
    }

    let manager = EnvironmentManager::new(config.clone());

    // First enable the plugin if it's not already enabled
    manager.plugin_manager.enable_plugin(plugin, env).await?;

    // Open the plugin config file for editing
    let plugin_file = config
        .env_dir(env)
        .join("plugins")
        .join(format!("{}.yaml", plugin));

    // Create plugins directory if it doesn't exist
    if let Some(parent) = plugin_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create a default config file if it doesn't exist
    if !plugin_file.exists() {
        let default_config = "# Plugin configuration\n# Add your plugin settings here\n";
        std::fs::write(&plugin_file, default_config)?;
    }

    // Use the user's preferred editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    let status = Command::new(&editor).arg(&plugin_file).status()?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    println!(
        "Plugin '{}' configuration updated for environment '{}'",
        plugin, env
    );
    Ok(())
}

/// Install shell hooks to automatically apply the current environment in new shells
pub async fn install_shell_hooks(config: &EnvMgrConfig, shell: Option<&str>) -> Result<()> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    // Determine target shell
    let detected_shell = shell.map(|s| s.to_string().to_lowercase()).or_else(|| {
        // Try to detect from environment
        Some(
            detect_shell()
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
        )
    });

    // Determine rc file and snippet
    let (shell_name, rc_file, snippet) = match detected_shell.as_deref() {
        Some("fish") => {
            let rc = home.join(".config/fish/conf.d/envmgr.fish");
            let snip = format!(
                r#"
# >>> envmgr (auto-generated) start >>>
function envmgr --wraps envmgr
    if test (count $argv) -ge 1 -a "$argv[1]" = "use"
        command envmgr $argv | source -
    else
        command envmgr $argv
    end
end
if test -f {cfg}/current
    set -l cur (cat {cfg}/current)
    if test -n "$cur"
        command envmgr use $cur | source -
    end
end
# <<< envmgr (auto-generated) end <<<"#,
                cfg = config.config_dir.display()
            );
            ("fish", rc, snip)
        }
        Some("zsh") => {
            let rc = home.join(".zshrc");
            let snip = format!(
                r#"
# >>> envmgr (auto-generated) start >>>
function envmgr() {{
  if [ "$1" = "use" ]; then
    eval "$(command envmgr "$@")"
  else
    command envmgr "$@"
  fi
}}
if [ -f {cfg}/current ]; then cur="$(cat {cfg}/current)"; if [ -n "$cur" ]; then eval "$(command envmgr use "$cur")"; fi; fi
# <<< envmgr (auto-generated) end <<<\n"#,
                cfg = config.config_dir.display()
            );
            ("zsh", rc, snip)
        }
        Some("bash") => {
            let rc = home.join(".bashrc");
            let snip = format!(
                r#"
# >>> envmgr (auto-generated) start >>>
function envmgr() {{
  if [ "$1" = "use" ]; then
    eval "$(command envmgr "$@")"
  else
    command envmgr "$@"
  fi
}}
if [ -f {cfg}/current ]; then cur="$(cat {cfg}/current)"; if [ -n "$cur" ]; then eval "$(command envmgr use "$cur")"; fi; fi
# <<< envmgr (auto-generated) end <<<"#,
                cfg = config.config_dir.display()
            );
            ("bash", rc, snip)
        }
        _ => {
            anyhow::bail!(
                "Unsupported shell: {}",
                detected_shell.unwrap_or("unknown".to_string())
            );
        }
    };

    // Create parent dirs if needed and append snippet if not present
    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains("envmgr (auto-generated) start") {
        println!(
            "envmgr: {} hooks already installed in {}",
            shell_name,
            rc_file.display()
        );
        return Ok(());
    }

    std::fs::write(&rc_file, format!("{}{}", existing, snippet))
        .with_context(|| format!("Failed to write to {}", rc_file.display()))?;

    println!(
        "envmgr: Installed {} hooks into {}",
        shell_name,
        rc_file.display()
    );
    println!("Open a new shell or source the file to apply.");
    Ok(())
}
