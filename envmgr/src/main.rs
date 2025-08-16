use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

mod commands;
mod config;
mod dotfiles;
mod environment;
mod plugins;

use commands::*;
use config::EnvMgrConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "envmgr")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available environments
    List,
    /// Show current active environment
    Current,
    /// Use/activate an environment
    Use {
        /// Name of the environment to activate
        name: String,
    },
    /// Install shell hooks to auto-apply current env on new shells
    Install {
        /// Target shell to install hooks for (defaults to $SHELL)
        #[arg(long)]
        shell: Option<ShellKind>,
    },
    /// Add a new environment
    Add {
        /// Name of the new environment
        name: String,
        /// Base environment to inherit from (optional)
        #[arg(long)]
        base: Option<String>,
    },
    /// Remove an environment
    Remove {
        /// Name of the environment to remove
        name: String,
    },
    /// Edit environment configuration
    Edit {
        /// Name of the environment to edit
        name: String,
    },
    /// Dotfiles management
    #[command(subcommand)]
    Dotfiles(DotfilesCommands),
    /// Plugin management
    #[command(subcommand)]
    Plugin(PluginCommands),
}

#[derive(Subcommand)]
enum DotfilesCommands {
    /// List managed dotfiles
    List,
    /// Re-link all dotfiles
    Link,
    /// Show differences between environments
    Diff {
        /// Environment name to compare
        env: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List available plugins
    List,
    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
    /// Configure a plugin for an environment
    Config {
        /// Plugin name
        plugin: String,
        /// Environment name
        env: String,
    },
    /// Install a plugin
    Install {
        /// Plugin name
        ///
        /// This can be a single name for first-party plugins,
        /// or a full identifier for third-party plugins (e.g., `user/repo`)
        identifier: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = EnvMgrConfig::load().context("Failed to load envmgr configuration")?;

    match cli.command {
        Commands::List => list_environments(&config).await,
        Commands::Current => show_current(&config).await,
        Commands::Use { name } => use_environment(&config, &name).await,
        Commands::Install { shell } => {
            let shell_str = shell.map(|s| {
                match s {
                    ShellKind::Bash => "bash",
                    ShellKind::Zsh => "zsh",
                    ShellKind::Fish => "fish",
                }
                .to_string()
            });
            install_shell_hooks(&config, shell_str.as_deref()).await
        }
        Commands::Add { name, base } => add_environment(&config, &name, base.as_deref()).await,
        Commands::Remove { name } => remove_environment(&config, &name).await,
        Commands::Edit { name } => edit_environment(&config, &name).await,
        Commands::Dotfiles(cmd) => match cmd {
            DotfilesCommands::List => list_dotfiles(&config).await,
            DotfilesCommands::Link => link_dotfiles(&config).await,
            DotfilesCommands::Diff { env } => diff_dotfiles(&config, &env).await,
        },
        Commands::Plugin(cmd) => match cmd {
            PluginCommands::List => list_plugins(&config).await,
            PluginCommands::Enable { name } => enable_plugin(&config, &name).await,
            PluginCommands::Disable { name } => disable_plugin(&config, &name).await,
            PluginCommands::Config { plugin, env } => {
                configure_plugin(&config, &plugin, &env).await
            }
            PluginCommands::Install { identifier: _ } => {
                anyhow::bail!("Plugin installation is not yet implemented");
                // install_plugin(&config, &identifier).await
            }
        },
    }
}
