use std::path::Path;

use clap::{CommandFactory, Parser};
use envmgr::cli::{Args, Command, Shell};
use envmgr::config::BASE_ENV_NAME;
use envmgr::environment::EnvironmentManager;
use envmgr::error::EnvMgrResult;
use indoc::indoc;
use log::info;

fn make_fish_hook(bin_name: &str) -> String {
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
    let cli = Args::parse();

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
        Command::Init { force } => {
            info!("Initializing environment manager. Force: {}", force);
            todo!("Implement init functionality");
        }
        Command::Hook { shell } => match shell {
            Shell::Fish => {
                println!("{}", make_fish_hook(&bin_name));
                Ok(())
            }
        },
        Command::Add { name } => {
            info!("Adding a new environment. Name: {}", name);
            envmgr::commands::add::add_environment(name)
        }
        Command::List => {
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
        Command::Remove { name } => {
            info!("Removing environment: {}", name);
            todo!("Implement remove functionality");
        }
        Command::Use => {
            let em = EnvironmentManager { shell: Shell::Fish };
            em.use_environment()
        }
        Command::Link => EnvironmentManager::link_files(),
        Command::Switch { name } => {
            if name == BASE_ENV_NAME {
                return EnvironmentManager::switch_base_environment();
            }
            EnvironmentManager::switch_environment_by_key(name)
        }
        Command::Doctor => {
            info!("Running health check.");
            todo!("Implement doctor functionality");
        }
        Command::Completions { shell } => {
            let mut cmd = Args::command();
            clap_complete::generate(*shell, &mut cmd, &bin_name, &mut std::io::stdout());
            eprintln!(
                "Usage: {bin_name} completions fish > ~/.config/fish/completions/{bin_name}.fish"
            );
            Ok(())
        }
    }
}
