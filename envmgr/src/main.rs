use clap::Parser;
use indoc::indoc;
use log::info;
use std::path::Path;

use crate::environment::EnvironmentManager;
use crate::error::EnvMgrResult;

mod cli;
mod config;
mod environment;
mod error;
mod integrations;
mod state;

fn make_fish_hook(bin_name: &str) -> String {
    // We output a direnv-like hook with two parts:
    // 1) A function named like the binary that intercepts `use` and `switch` and pipes output to `source`.
    // 2) Event-driven hooks that re-apply the environment on directory change or before command exec,
    //    controlled by $envmgr_fish_mode ("disable_arrow" to disable, "eval_after_arrow" to defer until after the prompt arrow).
    //
    // Users should run: envmgr hook fish | source
    // Or persist into ~/.config/fish/conf.d/envmgr.fish
    indoc! {r#"
    # envmgr fish hook

    # Re-apply env on prompt draw
    function __envmgr_export_eval --on-event fish_prompt
        command BIN_NAME use | source
    end"#}
    .replace("BIN_NAME", bin_name)
}

fn main() -> EnvMgrResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .format_source_path(false)
        .format_target(false)
        .init();
    let cli = cli::Args::parse();

    // Use only the executable basename as the fish function name (avoid paths like target/debug/envmgr)
    let bin_name = std::env::args()
        .next()
        .and_then(|p| {
            Path::new(&p)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .filter(|s: &String| !s.is_empty())
        .unwrap_or_else(|| "envmgr".to_string());

    match &cli.command {
        cli::Command::Init { force } => {
            info!("Initializing environment manager. Force: {}", force);
            todo!("Implement init functionality");
        }
        cli::Command::Hook { shell } => match shell {
            cli::Shell::Fish => {
                // Emit fish shell hook that defines a function to eval envmgr output
                // Users will run: envmgr hook fish | source
                println!("{}", make_fish_hook(&bin_name));
                Ok(())
            }
        },
        cli::Command::Add { name } => {
            info!("Adding a new environment. Name: {}", name);
            todo!("Implement add functionality");
        }
        cli::Command::List => {
            info!("Listing all environments.");
            let environments = EnvironmentManager::list_environments()?;
            for (current, env) in environments {
                eprintln!(
                    "{} {} - {}",
                    if current { "*" } else { " " },
                    env.key,
                    env.name
                );
            }
            Ok(())
        }
        cli::Command::Remove { name } => {
            info!("Removing environment: {}", name);
            todo!("Implement remove functionality");
        }
        cli::Command::Use => {
            let em = EnvironmentManager {
                shell: cli::Shell::Fish,
            };
            em.use_environment()
        }
        cli::Command::Link => EnvironmentManager::link_files(),
        cli::Command::Switch { name } => {
            if name == config::BASE_ENV_NAME {
                return EnvironmentManager::switch_base_environment();
            }
            EnvironmentManager::switch_environment_by_key(name)
        }
        cli::Command::Doctor => {
            info!("Running health check.");
            todo!("Implement doctor functionality");
        }
    }
}
