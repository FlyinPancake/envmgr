use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::config::{EnvMgrConfig, EnvironmentConfig};

struct SymlinkManager {
    symlinks: HashSet<PathBuf>,
}

impl SymlinkManager {
    pub fn new() -> Self {
        let mut sm = Self {
            symlinks: HashSet::new(),
        };
        sm.load_from_state().unwrap();
        sm
    }

    fn get_state_file() -> PathBuf {
        let home_dir = dirs::home_dir().expect("Could not determine home directory");
        dirs::state_dir()
            .unwrap_or_else(|| home_dir.join(".local/state"))
            .join("envmgr")
    }

    fn load_from_state(&mut self) -> Result<()> {
        let state_file = Self::get_state_file();
        if state_file.exists() {
            let contents = fs::read_to_string(state_file)?;
            for line in contents.lines() {
                let path = PathBuf::from(line);
                self.symlinks.insert(path);
            }
        }
        Ok(())
    }

    fn save_to_state(&self) -> Result<()> {
        let state_file = Self::get_state_file();
        let mut file = fs::File::create(state_file)?;
        for path in &self.symlinks {
            writeln!(file, "{}", path.display())?;
        }
        Ok(())
    }

    pub fn add_symlink(&mut self, path: PathBuf) {
        self.symlinks.insert(path);
        self.save_to_state()
            .expect("Failed to save symlinks to state");
    }

    pub fn remove_all(&mut self) -> Result<()> {
        for name in self.symlinks.iter() {
            if name.exists() && name.is_symlink() {
                fs::remove_file(name)?;
            }
        }
        self.symlinks.clear();
        self.save_to_state()?;
        Ok(())
    }
}

/// Manages dotfiles linking and unlinking
pub struct DotfileManager {
    config: EnvMgrConfig,
    symlink_manager: SymlinkManager,
}

impl DotfileManager {
    pub fn new(config: EnvMgrConfig) -> Self {
        Self {
            config,
            symlink_manager: SymlinkManager::new(),
        }
    }

    /// Apply dotfiles for a specific environment
    pub async fn apply_environment(&mut self, env_config: &EnvironmentConfig) -> Result<()> {
        // First, collect all dotfiles from base and environment
        let base_dotfiles = self.collect_base_dotfiles()?;
        let env_dotfiles = self.collect_env_dotfiles(env_config)?;

        // Remove existing symlinks that we manage
        self.remove_managed_symlinks(&base_dotfiles, &env_dotfiles)?;

        // Link base dotfiles first
        for (name, source) in &base_dotfiles {
            // Skip if environment has an override
            if !env_dotfiles.contains_key(name) {
                self.create_symlink(&source, name)?;
            }
        }

        // Link environment dotfiles (overrides)
        for (name, source) in &env_dotfiles {
            self.create_symlink(&source, name)?;
        }

        Ok(())
    }

    /// Collect base dotfiles
    fn collect_base_dotfiles(&self) -> Result<std::collections::HashMap<String, PathBuf>> {
        let base_dotfiles_dir = self.config.base_dir().join("dotfiles");
        self.collect_dotfiles_from_dir(&base_dotfiles_dir)
    }

    /// Collect environment-specific dotfiles
    fn collect_env_dotfiles(
        &self,
        env_config: &EnvironmentConfig,
    ) -> Result<std::collections::HashMap<String, PathBuf>> {
        let env_dotfiles_dir = env_config.dotfiles_dir(&self.config.config_dir);
        self.collect_dotfiles_from_dir(&env_dotfiles_dir)
    }

    /// Collect dotfiles from a directory
    fn collect_dotfiles_from_dir(
        &self,
        dir: &Path,
    ) -> Result<std::collections::HashMap<String, PathBuf>> {
        let mut dotfiles = std::collections::HashMap::new();

        if !dir.exists() {
            return Ok(dotfiles);
        }

        // Walk the directory recursively and collect files using paths relative to `dir`
        fn walk(
            base: &Path,
            current: &Path,
            acc: &mut std::collections::HashMap<String, PathBuf>,
        ) -> Result<()> {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Recurse into subdirectories
                    walk(base, &path, acc)?;
                } else if path.is_file() {
                    // Compute the key as a relative path from the base directory
                    let rel = path
                        .strip_prefix(base)
                        .context("Failed to compute relative path")?;
                    let key = rel.to_str().context("Invalid UTF-8 in path")?.to_string();
                    acc.insert(key, path);
                }
                // Ignore other kinds (e.g., sockets, FIFOs)
            }
            Ok(())
        }

        walk(dir, dir, &mut dotfiles)?;

        Ok(dotfiles)
    }

    /// Remove existing managed symlinks
    fn remove_managed_symlinks(
        &mut self,
        base_dotfiles: &std::collections::HashMap<String, PathBuf>,
        env_dotfiles: &std::collections::HashMap<String, PathBuf>,
    ) -> Result<()> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        // Collect all dotfile names we might manage
        let mut all_names = HashSet::new();
        all_names.extend(base_dotfiles.keys());
        all_names.extend(env_dotfiles.keys());

        for name in all_names {
            let target = home_dir.join(name);

            if target.is_symlink() {
                // Check if this symlink points to one of our managed files
                if let Ok(link_target) = fs::read_link(&target) {
                    let is_managed = base_dotfiles.values().any(|p| p == &link_target)
                        || env_dotfiles.values().any(|p| p == &link_target)
                        || link_target.starts_with(&self.config.config_dir);

                    if is_managed {
                        fs::remove_file(&target)?;
                    }
                }
            }
        }

        self.symlink_manager.remove_all()?;

        Ok(())
    }

    /// Create a symlink from source to target in home directory
    fn create_symlink(&mut self, source: &Path, filename: &str) -> Result<()> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;
        let target = home_dir.join(filename);

        // Create parent directories if needed
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        symlink(source, &target).with_context(|| {
            format!(
                "Failed to create symlink {} -> {}",
                target.display(),
                source.display()
            )
        })?;

        self.symlink_manager.add_symlink(home_dir.join(filename));

        Ok(())
    }

    /// List all managed dotfiles
    pub async fn list_dotfiles(&self) -> Result<()> {
        println!("Base dotfiles:");
        let base_dotfiles = self.collect_base_dotfiles()?;
        for (name, path) in &base_dotfiles {
            println!("  {} -> {}", name, path.display());
        }

        // List environment-specific dotfiles
        let envs = self.config.list_environments()?;
        for env_name in envs {
            let env_config = EnvironmentConfig::load(&self.config.config_dir, &env_name)?;
            let env_dotfiles = self.collect_env_dotfiles(&env_config)?;

            if !env_dotfiles.is_empty() {
                println!("\nEnvironment '{}' overrides:", env_name);
                for (name, path) in &env_dotfiles {
                    println!("  {} -> {}", name, path.display());
                }
            }
        }

        Ok(())
    }

    /// Relink all dotfiles for current environment
    pub async fn relink_dotfiles(&mut self) -> Result<()> {
        if let Some(current_env) = &self.config.current_env.clone() {
            let env_config = EnvironmentConfig::load(&self.config.config_dir, current_env)?;
            self.apply_environment(&env_config).await?;
            println!("Re-linked dotfiles for environment '{}'", current_env);
        } else {
            // Just link base dotfiles
            let base_dotfiles = self.collect_base_dotfiles()?;
            self.remove_managed_symlinks(&base_dotfiles, &std::collections::HashMap::new())?;

            for (name, source) in &base_dotfiles {
                self.create_symlink(&source, name)?;
            }

            println!("Linked base dotfiles");
        }

        Ok(())
    }

    /// Show differences between environments
    pub async fn diff_environments(&self, env_name: &str) -> Result<()> {
        if !self.config.env_exists(env_name) {
            anyhow::bail!("Environment '{}' does not exist", env_name);
        }

        let base_dotfiles = self.collect_base_dotfiles()?;
        let env_config = EnvironmentConfig::load(&self.config.config_dir, env_name)?;
        let env_dotfiles = self.collect_env_dotfiles(&env_config)?;

        println!("Differences for environment '{}':", env_name);

        // Show overrides
        for name in env_dotfiles.keys() {
            if base_dotfiles.contains_key(name) {
                println!("  Override: {}", name);
            } else {
                println!("  New: {}", name);
            }
        }

        // Show inherited files
        for name in base_dotfiles.keys() {
            if !env_dotfiles.contains_key(name) {
                println!("  Inherited: {}", name);
            }
        }

        Ok(())
    }
}
