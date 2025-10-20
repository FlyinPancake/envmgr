use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_environment_config_serialization() {
    use envmgr::config::{EnvVarsConfig, EnvironmentConfig};

    let config = EnvironmentConfig {
        name: "Test Environment".to_string(),
        env_vars: vec![EnvVarsConfig {
            key: "TEST_VAR".to_string(),
            value: "test_value".to_string(),
        }],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let yaml_str = serde_json::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_json::from_str(&yaml_str).unwrap();

    assert_eq!(deserialized.name, "Test Environment");
    assert_eq!(deserialized.env_vars.len(), 1);
    assert_eq!(deserialized.env_vars[0].key, "TEST_VAR");
}

fn create_test_env_structure(base_path: &Path, env_key: &str) -> PathBuf {
    let env_dir = base_path.join("environments").join(env_key);
    fs::create_dir_all(&env_dir).unwrap();

    let config = r#"
name: "Test Environment"
env_vars:
  - key: "TEST_VAR1"
    value: "value1"
  - key: "TEST_VAR2"
    value: "value2"
"#;

    fs::write(env_dir.join("config.yaml"), config).unwrap();
    env_dir
}

#[test]
fn test_environment_files_discovery() {
    let temp_dir = std::env::temp_dir().join("envmgr_integration_test_files");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let env_dir = create_test_env_structure(&temp_dir, "testenv");
    let files_dir = env_dir.join("files");
    fs::create_dir_all(&files_dir).unwrap();

    fs::write(files_dir.join(".bashrc"), "export TEST=1").unwrap();
    fs::create_dir_all(files_dir.join(".config")).unwrap();
    fs::write(files_dir.join(".config").join("app.conf"), "config data").unwrap();

    let discovered_files: Vec<PathBuf> = fs::read_dir(&files_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    assert!(discovered_files.len() >= 2);

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_state_persistence() {
    use envmgr::state::State;

    let state = State {
        current_env_key: "test_env".to_string(),
        applied_env_vars: HashMap::from([
            ("VAR1".to_string(), "value1".to_string()),
            ("VAR2".to_string(), "value2".to_string()),
        ]),
        managed_files: vec![PathBuf::from("/tmp/file1"), PathBuf::from("/tmp/file2")],
    };

    let serialized = toml::to_string_pretty(&state).unwrap();
    let deserialized: State = toml::from_str(&serialized).unwrap();

    assert_eq!(deserialized.current_env_key, "test_env");
    assert_eq!(deserialized.applied_env_vars.len(), 2);
    assert_eq!(deserialized.managed_files.len(), 2);
}

#[test]
fn test_symlink_creation() {
    let temp_dir = std::env::temp_dir().join("envmgr_symlink_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let source = temp_dir.join("source.txt");
    let target = temp_dir.join("target_link");

    fs::write(&source, "test content").unwrap();
    std::os::unix::fs::symlink(&source, &target).unwrap();

    assert!(target.is_symlink());
    assert_eq!(fs::read_link(&target).unwrap(), source);
    assert_eq!(fs::read_to_string(&target).unwrap(), "test content");

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_symlink_update() {
    let temp_dir = std::env::temp_dir().join("envmgr_symlink_update_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let source1 = temp_dir.join("source1.txt");
    let source2 = temp_dir.join("source2.txt");
    let target = temp_dir.join("target_link");

    fs::write(&source1, "content1").unwrap();
    fs::write(&source2, "content2").unwrap();

    std::os::unix::fs::symlink(&source1, &target).unwrap();
    assert_eq!(fs::read_link(&target).unwrap(), source1);

    fs::remove_file(&target).unwrap();
    std::os::unix::fs::symlink(&source2, &target).unwrap();
    assert_eq!(fs::read_link(&target).unwrap(), source2);

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_env_vars_config() {
    use envmgr::config::EnvVarsConfig;

    let env_var = EnvVarsConfig {
        key: "DATABASE_URL".to_string(),
        value: "postgres://localhost/mydb".to_string(),
    };

    let json = serde_json::to_string(&env_var).unwrap();
    let deserialized: EnvVarsConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.key, "DATABASE_URL");
    assert_eq!(deserialized.value, "postgres://localhost/mydb");
}

#[test]
fn test_multiple_env_vars_merge() {
    let base_vars = vec![
        ("VAR1".to_string(), "base1".to_string()),
        ("VAR2".to_string(), "base2".to_string()),
    ];

    let env_vars = vec![
        ("VAR2".to_string(), "override2".to_string()),
        ("VAR3".to_string(), "new3".to_string()),
    ];

    let mut merged = HashMap::new();
    for (k, v) in base_vars {
        merged.insert(k, v);
    }
    for (k, v) in env_vars {
        merged.insert(k, v);
    }

    assert_eq!(merged.get("VAR1"), Some(&"base1".to_string()));
    assert_eq!(merged.get("VAR2"), Some(&"override2".to_string()));
    assert_eq!(merged.get("VAR3"), Some(&"new3".to_string()));
}

// ============= Add Command Integration Tests =============

#[test]
fn test_environment_config_with_gh_cli_integration() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::gh_cli::{GhCliConfig, GhCliHostUser};

    let config = EnvironmentConfig {
        name: "GitHub Test".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: Some(GhCliConfig {
            hosts: vec![GhCliHostUser {
                host: "github.com".to_string(),
                user: "testuser".to_string(),
            }],
        }),
        tailscale: None,
    };

    // Test serialization with serde_norway
    let yaml = serde_norway::to_string(&config).unwrap();
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("GitHub Test"));
    assert!(yaml.contains("gh_cli:"));
    assert!(yaml.contains("github.com"));
    assert!(yaml.contains("testuser"));

    // Test deserialization
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();
    assert_eq!(deserialized.name, "GitHub Test");
    assert!(deserialized.gh_cli.is_some());
    assert_eq!(deserialized.gh_cli.unwrap().hosts[0].user, "testuser");
}

#[test]
fn test_environment_config_with_one_password_ssh_integration() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::one_password_ssh_agent::{OnePasswordSSHAgentConfig, OnePasswordSSHKey};

    let config = EnvironmentConfig {
        name: "1Password Test".to_string(),
        env_vars: vec![],
        op_ssh: Some(OnePasswordSSHAgentConfig {
            keys: vec![
                OnePasswordSSHKey {
                    vault: Some("Work".to_string()),
                    item: Some("SSH Key".to_string()),
                    account: Some("user@example.com".to_string()),
                },
            ],
        }),
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    assert!(yaml.contains("op_ssh:"));
    assert!(yaml.contains("vault:"));
    assert!(yaml.contains("Work"));

    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();
    assert!(deserialized.op_ssh.is_some());
    assert_eq!(deserialized.op_ssh.unwrap().keys[0].vault, Some("Work".to_string()));
}

#[test]
fn test_environment_config_with_tailscale_integration() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::tailscale::TailscaleConfig;

    let config = EnvironmentConfig {
        name: "Tailscale Test".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: None,
        tailscale: Some(TailscaleConfig {
            tailnet: "company.example.com".to_string(),
        }),
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    assert!(yaml.contains("tailscale:"));
    assert!(yaml.contains("company.example.com"));

    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();
    assert!(deserialized.tailscale.is_some());
    assert_eq!(deserialized.tailscale.unwrap().tailnet, "company.example.com");
}

#[test]
fn test_environment_config_with_all_integrations_serialization() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::gh_cli::{GhCliConfig, GhCliHostUser};
    use envmgr::integrations::one_password_ssh_agent::{OnePasswordSSHAgentConfig, OnePasswordSSHKey};
    use envmgr::integrations::tailscale::TailscaleConfig;

    let config = EnvironmentConfig {
        name: "Full Integration Test".to_string(),
        env_vars: vec![],
        op_ssh: Some(OnePasswordSSHAgentConfig {
            keys: vec![OnePasswordSSHKey {
                vault: Some("Personal".to_string()),
                item: Some("Key".to_string()),
                account: None,
            }],
        }),
        gh_cli: Some(GhCliConfig {
            hosts: vec![GhCliHostUser {
                host: "github.com".to_string(),
                user: "user".to_string(),
            }],
        }),
        tailscale: Some(TailscaleConfig {
            tailnet: "example.com".to_string(),
        }),
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "Full Integration Test");
    assert!(deserialized.op_ssh.is_some());
    assert!(deserialized.gh_cli.is_some());
    assert!(deserialized.tailscale.is_some());
}

#[test]
fn test_environment_config_with_multiple_github_hosts() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::gh_cli::{GhCliConfig, GhCliHostUser};

    let config = EnvironmentConfig {
        name: "Multi-Host GitHub".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: Some(GhCliConfig {
            hosts: vec![
                GhCliHostUser {
                    host: "github.com".to_string(),
                    user: "personal-user".to_string(),
                },
                GhCliHostUser {
                    host: "github.enterprise.com".to_string(),
                    user: "work-user".to_string(),
                },
            ],
        }),
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert!(deserialized.gh_cli.is_some());
    let hosts = deserialized.gh_cli.unwrap().hosts;
    assert_eq!(hosts.len(), 2);
    assert_eq!(hosts[0].host, "github.com");
    assert_eq!(hosts[1].host, "github.enterprise.com");
}

#[test]
fn test_environment_config_with_multiple_ssh_keys() {
    use envmgr::config::EnvironmentConfig;
    use envmgr::integrations::one_password_ssh_agent::{OnePasswordSSHAgentConfig, OnePasswordSSHKey};

    let config = EnvironmentConfig {
        name: "Multi-Key SSH".to_string(),
        env_vars: vec![],
        op_ssh: Some(OnePasswordSSHAgentConfig {
            keys: vec![
                OnePasswordSSHKey {
                    vault: Some("Work".to_string()),
                    item: Some("Work Key".to_string()),
                    account: Some("work@company.com".to_string()),
                },
                OnePasswordSSHKey {
                    vault: Some("Personal".to_string()),
                    item: Some("Personal Key".to_string()),
                    account: None,
                },
                OnePasswordSSHKey {
                    vault: None,
                    item: Some("Default Key".to_string()),
                    account: None,
                },
            ],
        }),
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert!(deserialized.op_ssh.is_some());
    let keys = deserialized.op_ssh.unwrap().keys;
    assert_eq!(keys.len(), 3);
    assert_eq!(keys[0].vault, Some("Work".to_string()));
    assert_eq!(keys[1].account, None);
    assert_eq!(keys[2].vault, None);
}

#[test]
fn test_environment_directory_structure_creation() {
    let temp_dir = std::env::temp_dir().join("envmgr_add_test_structure");
    let _ = fs::remove_dir_all(&temp_dir);

    let env_key = "test-env";
    let env_dir = temp_dir.join("environments").join(env_key);
    fs::create_dir_all(&env_dir).unwrap();

    let files_dir = env_dir.join("files");
    fs::create_dir_all(&files_dir).unwrap();

    assert!(env_dir.exists());
    assert!(env_dir.is_dir());
    assert!(files_dir.exists());
    assert!(files_dir.is_dir());

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_environment_config_file_creation() {
    use envmgr::config::EnvironmentConfig;

    let temp_dir = std::env::temp_dir().join("envmgr_add_test_config");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let config = EnvironmentConfig {
        name: "Test Config Write".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let config_path = temp_dir.join("config.yaml");
    let yaml = serde_norway::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();

    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("name:"));
    assert!(content.contains("Test Config Write"));

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_environment_config_empty_integrations() {
    use envmgr::config::EnvironmentConfig;

    let config = EnvironmentConfig {
        name: "Minimal Config".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "Minimal Config");
    assert!(deserialized.op_ssh.is_none());
    assert!(deserialized.gh_cli.is_none());
    assert!(deserialized.tailscale.is_none());
}

#[test]
fn test_serde_norway_yaml_formatting() {
    use envmgr::config::{EnvVarsConfig, EnvironmentConfig};

    let config = EnvironmentConfig {
        name: "Format Test".to_string(),
        env_vars: vec![
            EnvVarsConfig {
                key: "VAR1".to_string(),
                value: "value1".to_string(),
            },
            EnvVarsConfig {
                key: "VAR2".to_string(),
                value: "value2".to_string(),
            },
        ],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    
    // Verify YAML structure
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("env_vars:"));
    assert!(yaml.contains("- key: VAR1") || yaml.contains("key: VAR1"));
    assert!(yaml.contains("value: value1") || yaml.contains("value: 'value1'"));
}

#[test]
fn test_environment_config_roundtrip_with_special_characters() {
    use envmgr::config::{EnvVarsConfig, EnvironmentConfig};

    let config = EnvironmentConfig {
        name: "Special: Chars & Test!".to_string(),
        env_vars: vec![EnvVarsConfig {
            key: "DATABASE_URL".to_string(),
            value: "postgresql://user:pass@localhost:5432/db?sslmode=require".to_string(),
        }],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "Special: Chars & Test!");
    assert_eq!(
        deserialized.env_vars[0].value,
        "postgresql://user:pass@localhost:5432/db?sslmode=require"
    );
}

#[test]
fn test_environment_config_with_empty_strings() {
    use envmgr::integrations::tailscale::TailscaleConfig;
    use envmgr::config::EnvironmentConfig;

    let config = EnvironmentConfig {
        name: "".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: None,
        tailscale: Some(TailscaleConfig {
            tailnet: "".to_string(),
        }),
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "");
    assert!(deserialized.tailscale.is_some());
    assert_eq!(deserialized.tailscale.unwrap().tailnet, "");
}

#[test]
fn test_environment_config_path_functions() {
    use envmgr::config::EnvironmentConfig;

    let base_dir = EnvironmentConfig::get_base_env_dir();
    assert!(base_dir.to_string_lossy().contains("base"));

    let env_dir = EnvironmentConfig::get_env_dir_by_key("work");
    assert!(env_dir.to_string_lossy().contains("environments"));
    assert!(env_dir.to_string_lossy().contains("work"));

    let all_envs_dir = EnvironmentConfig::get_all_envs_dir();
    assert!(all_envs_dir.to_string_lossy().contains("environments"));
}

#[test]
fn test_complex_real_world_environment_config() {
    use envmgr::config::{EnvVarsConfig, EnvironmentConfig};
    use envmgr::integrations::gh_cli::{GhCliConfig, GhCliHostUser};
    use envmgr::integrations::one_password_ssh_agent::{OnePasswordSSHAgentConfig, OnePasswordSSHKey};
    use envmgr::integrations::tailscale::TailscaleConfig;

    let config = EnvironmentConfig {
        name: "Work Environment - Client ABC".to_string(),
        env_vars: vec![
            EnvVarsConfig {
                key: "AWS_PROFILE".to_string(),
                value: "client-abc-prod".to_string(),
            },
            EnvVarsConfig {
                key: "KUBECONFIG".to_string(),
                value: "/home/user/.kube/client-abc".to_string(),
            },
            EnvVarsConfig {
                key: "JIRA_URL".to_string(),
                value: "https://client-abc.atlassian.net".to_string(),
            },
        ],
        op_ssh: Some(OnePasswordSSHAgentConfig {
            keys: vec![
                OnePasswordSSHKey {
                    vault: Some("Work - Client ABC".to_string()),
                    item: Some("GitHub Deploy Key".to_string()),
                    account: Some("team@client-abc.com".to_string()),
                },
                OnePasswordSSHKey {
                    vault: Some("Work - Client ABC".to_string()),
                    item: Some("AWS SSH Key".to_string()),
                    account: Some("team@client-abc.com".to_string()),
                },
            ],
        }),
        gh_cli: Some(GhCliConfig {
            hosts: vec![
                GhCliHostUser {
                    host: "github.com".to_string(),
                    user: "work-account".to_string(),
                },
                GhCliHostUser {
                    host: "github.client-abc.com".to_string(),
                    user: "internal-account".to_string(),
                },
            ],
        }),
        tailscale: Some(TailscaleConfig {
            tailnet: "client-abc.ts.net".to_string(),
        }),
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "Work Environment - Client ABC");
    assert_eq!(deserialized.env_vars.len(), 3);
    assert_eq!(deserialized.op_ssh.as_ref().unwrap().keys.len(), 2);
    assert_eq!(deserialized.gh_cli.as_ref().unwrap().hosts.len(), 2);
    assert_eq!(
        deserialized.tailscale.as_ref().unwrap().tailnet,
        "client-abc.ts.net"
    );
}

#[test]
fn test_environment_config_unicode_handling() {
    use envmgr::config::EnvironmentConfig;

    let config = EnvironmentConfig {
        name: "Тест Environment 🚀".to_string(),
        env_vars: vec![],
        op_ssh: None,
        gh_cli: None,
        tailscale: None,
    };

    let yaml = serde_norway::to_string(&config).unwrap();
    let deserialized: EnvironmentConfig = serde_norway::from_str(&yaml).unwrap();

    assert_eq!(deserialized.name, "Тест Environment 🚀");
}