use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectedShell {
    Bash,
    Zsh,
    Fish,
}

impl ToString for DetectedShell {
    fn to_string(&self) -> String {
        match self {
            DetectedShell::Bash => "bash".to_string(),
            DetectedShell::Zsh => "zsh".to_string(),
            DetectedShell::Fish => "fish".to_string(),
        }
    }
}

pub(crate) fn detect_shell() -> Result<DetectedShell> {
    let shell = whattheshell::Shell::infer();
    match shell {
        Ok(whattheshell::Shell::Fish) => Ok(DetectedShell::Fish),
        Ok(whattheshell::Shell::Zsh) => Ok(DetectedShell::Zsh),
        Ok(whattheshell::Shell::Bash) => Ok(DetectedShell::Bash),
        _ => {
            anyhow::bail!("Failed to detect shell.")
        }
    }
}

pub(crate) fn emit_set(sh: &DetectedShell, key: &str, value: &str) {
    match sh {
        DetectedShell::Fish => println!("set -gx {} {}", key, fish_escape(value)),
        DetectedShell::Bash | DetectedShell::Zsh => {
            println!("export {}={}", key, shell_escape(value))
        }
    }
}

/// Escape a string for shell usage
fn shell_escape(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}

fn fish_escape(s: &str) -> String {
    // Use single-quoted string, escape single quotes using the common POSIX trick
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}
