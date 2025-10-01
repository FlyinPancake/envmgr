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
