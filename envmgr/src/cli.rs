use clap::{Parser, ValueEnum};

/// Shells supported by envmgr hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Fish,
}

fn fish_quote(value: &str) -> String {
    // Basic single-quote escaping for fish: ' -> '\''
    // Also sanitize newlines and carriage returns (replace with spaces)
    if value.is_empty() {
        "''".to_string()
    } else {
        let sanitized = value.replace('\n', " ").replace('\r', " ");
        let escaped = sanitized.replace('\'', "'\\''");
        format!("'{}'", escaped)
    }
}

impl Shell {
    /// Generate a shell command to set an environment variable.
    pub fn set_env_var_cmd(&self, key: &str, value: &str) -> String {
        match self {
            Shell::Fish => {
                // Fish: export (-x) and make global (-g)
                format!("set -gx {} '{}'", key, fish_quote(value))
            }
        }
    }
    /// Generate a shell command to unset an environment variable.
    pub fn unset_env_var_cmd(&self, key: &str) -> String {
        match self {
            Shell::Fish => {
                // Fish: erase the global/exported variable if set
                format!("set -e -g {}", key)
            }
        }
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Initialize the environment manager
    Init {
        /// Force re-initialization if already initialized
        #[arg(short, long)]
        force: bool,
    },
    /// Output shell hook for integration
    ///
    /// For fish shell, run: `envmgr hook fish | source`
    /// Other shells not yet supported.
    Hook {
        /// Target shell to output hook for
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Add a new environment
    Add {
        /// Name of the new environment
        name: String,
    },
    /// List all environments
    List,
    /// Remove an environment
    Remove {
        /// Name of the environment to remove
        name: String,
    },
    /// Activate the current environment
    Use,
    /// Link files for the active environment
    Link,
    /// Switch to a different environment
    Switch {
        /// Name of the environment to switch to
        name: String,
    },
    /// Health check command
    Doctor,
    /// Generate shell completions
    Completions {
        /// Target shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
