use clap::{Parser, ValueEnum};

/// Shells supported by envmgr hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Fish,
}

/// Quote a string for safe use in fish shell commands.
fn fish_quote(value: &str) -> String {
    // Basic single-quote escaping for fish: ' -> '\''
    // Also sanitize newlines and carriage returns (replace with spaces)
    if value.is_empty() {
        "''".to_string()
    } else {
        let sanitized = value.replace(['\n', '\r'], " ");
        let escaped = sanitized.replace('\'', "\\'");
        format!("'{}'", escaped)
    }
}

impl Shell {
    /// Generate a shell command to set an environment variable.
    pub fn set_env_var_cmd(&self, key: &str, value: &str) -> String {
        match self {
            Shell::Fish => {
                // Fish: export (-x) and make global (-g)
                format!("set -gx {} {}", key, fish_quote(value))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fish_quote_empty() {
        assert_eq!(fish_quote(""), "''");
    }

    #[test]
    fn test_fish_quote_simple() {
        assert_eq!(fish_quote("hello"), "'hello'");
    }

    #[test]
    fn test_fish_quote_with_single_quotes() {
        assert_eq!(fish_quote("it's"), r#"'it\'s'"#);
    }

    #[test]
    fn test_fish_quote_with_newline() {
        assert_eq!(fish_quote("line1\nline2"), "'line1 line2'");
    }

    #[test]
    fn test_fish_quote_with_carriage_return() {
        assert_eq!(fish_quote("line1\rline2"), "'line1 line2'");
    }

    #[test]
    fn test_fish_quote_mixed_special_chars() {
        assert_eq!(fish_quote("hello\n'world'\r"), r#"'hello \'world\' '"#);
    }

    #[test]
    fn test_set_env_var_cmd_simple() {
        let shell = Shell::Fish;
        assert_eq!(
            shell.set_env_var_cmd("MY_VAR", "value"),
            "set -gx MY_VAR 'value'"
        );
    }

    #[test]
    fn test_set_env_var_cmd_with_special_chars() {
        let shell = Shell::Fish;
        assert_eq!(
            shell.set_env_var_cmd("PATH", "/usr/bin:/bin"),
            "set -gx PATH '/usr/bin:/bin'"
        );
    }

    #[test]
    fn test_set_env_var_cmd_with_quotes() {
        let shell = Shell::Fish;
        assert_eq!(
            shell.set_env_var_cmd("MSG", "it's working"),
            r#"set -gx MSG 'it\'s working'"#
        );
    }

    #[test]
    fn test_unset_env_var_cmd() {
        let shell = Shell::Fish;
        assert_eq!(shell.unset_env_var_cmd("MY_VAR"), "set -e -g MY_VAR");
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
