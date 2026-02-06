use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::rc::Rc;

use crate::constants::*;
use crate::error::{Result, WhichCmdError};
use crate::node::Node;

#[derive(Debug)]
pub struct Config {
    pub keys: Vec<Rc<Node>>,
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct ConfigHelper {
    keys: Vec<Node>,
}

impl Config {
    pub fn from_file() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
        let config_path = xdg_dirs.find_config_file(CONFIG_FILE_NAME).ok_or_else(|| {
            WhichCmdError::ConfigNotFound {
                path: format!(
                    "{}/{}",
                    xdg_dirs.get_config_home().display(),
                    CONFIG_FILE_NAME
                ),
            }
        })?;

        Config::from_path(&config_path)
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;

        Config::from_contents(&contents)
    }

    pub fn from_contents(contents: &str) -> Result<Self> {
        let helper: ConfigHelper = serde_yaml::from_str(contents)?;

        // Recursively loop through the config and set the id of each node.
        // It should be a concatenation of the keys of all the parent nodes
        // and the key of the current node.
        fn set_id(node: &mut Node, parent_id: &str) -> Result<()> {
            node.set_id_from_parent(parent_id);
            let keys: Vec<&str> = node.keys.iter().map(|n| n.key.as_str()).collect();
            Config::ensure_unique(&node.id, &keys)?;
            for child in node.keys.iter_mut() {
                // Get mutable reference to the node inside Rc
                let child_mut =
                    Rc::get_mut(child).expect("Should have exclusive access during initialization");
                set_id(child_mut, &node.id)?;
            }
            Ok(())
        }

        let keys: Vec<&str> = helper.keys.iter().map(|n| n.key.as_str()).collect();
        Config::ensure_unique("", &keys)?;

        let mut nodes = helper.keys;
        for node in nodes.iter_mut() {
            set_id(node, "")?;
        }

        Ok(Config {
            keys: nodes.into_iter().map(Rc::new).collect(),
        })
    }

    fn ensure_unique(parent_id: &str, keys: &[&str]) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for &key in keys {
            if seen.contains(key) {
                return Err(WhichCmdError::ConflictingKeys(format!(
                    "{}{}",
                    parent_id, key
                )));
            }
            seen.insert(key);
        }
        Ok(())
    }

    pub fn with_local_config(self) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        self.with_local_config_from_dir(&cwd)
    }

    fn with_local_config_from_dir(mut self, dir: &Path) -> Result<Self> {
        let local_path = LOCAL_CONFIG_FILE_NAMES
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.exists());

        let local_path = match local_path {
            Some(path) => path,
            None => return Ok(self),
        };

        // Require executable permission as a security measure
        let metadata = fs::metadata(&local_path)?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Ok(self);
        }

        let contents = fs::read_to_string(&local_path)?;
        let helper: ConfigHelper = serde_yaml::from_str(&contents)?;

        // Create a synthetic "." node to contain local config keys
        let mut local_node = Node::new_branch(
            LOCAL_CONFIG_KEY.to_string(),
            "local".to_string(),
            String::new(),
            helper.keys,
        );

        fn set_id(node: &mut Node, parent_id: &str) -> Result<()> {
            node.set_id_from_parent(parent_id);
            let keys: Vec<&str> = node.keys.iter().map(|n| n.key.as_str()).collect();
            Config::ensure_unique(&node.id, &keys)?;
            for child in node.keys.iter_mut() {
                let child_mut =
                    Rc::get_mut(child).expect("Should have exclusive access during initialization");
                set_id(child_mut, &node.id)?;
            }
            Ok(())
        }

        set_id(&mut local_node, "")?;

        // Check the "." key doesn't conflict with existing root keys
        let all_keys: Vec<&str> = self
            .keys
            .iter()
            .map(|n| n.key.as_str())
            .chain(std::iter::once(LOCAL_CONFIG_KEY))
            .collect();
        Config::ensure_unique("", &all_keys)?;

        self.keys.push(Rc::new(local_node));
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing_basic() {
        let yaml = r#"
keys:
  - key: g
    name: git
    value: git
    keys:
      - key: s
        name: status
        value: status
"#;
        let config = Config::from_contents(yaml).unwrap();
        assert_eq!(config.keys.len(), 1);
        let git_node = &config.keys[0];
        assert_eq!(git_node.id, "g");
        assert_eq!(git_node.key, "g");
        assert_eq!(git_node.name, "git");
        assert_eq!(git_node.value, "git");
        assert_eq!(git_node.keys.len(), 1);
        assert!(!git_node.is_loop);
        let status_node = &git_node.keys[0];
        assert_eq!(status_node.id, "gs");
        assert_eq!(status_node.key, "s");
        assert_eq!(status_node.name, "status");
        assert_eq!(status_node.value, "status");
    }

    #[test]
    fn test_config_parsing_no_name() {
        let yaml = r#"
keys:
  - key: g
    value: git
"#;
        let config = Config::from_contents(yaml).unwrap();
        assert_eq!(config.keys.len(), 1);
        let git_node = &config.keys[0];
        assert_eq!(git_node.key, "g");
        assert_eq!(git_node.name, "git");
        assert_eq!(git_node.value, "git");
    }

    #[test]
    fn test_config_parsing_no_value() {
        let yaml = r#"
keys:
  - key: g
    name: git commands
"#;
        let config = Config::from_contents(yaml).unwrap();
        assert_eq!(config.keys.len(), 1);
        let git_node = &config.keys[0];
        assert_eq!(git_node.key, "g");
        assert_eq!(git_node.name, "git commands");
        assert_eq!(git_node.value, "");
    }

    #[test]
    #[should_panic]
    fn test_config_parsing_neither_name_nor_value() {
        let yaml = r#"
keys:
  - key: g
    keys: []
"#;
        let _ = Config::from_contents(yaml).unwrap();
    }

    #[test]
    fn test_config_parsing_loop() {
        let yaml = r#"
keys:
  - key: g
    value: git
    loop: true
"#;
        let config = Config::from_contents(yaml).unwrap();
        assert_eq!(config.keys.len(), 1);
        let git_node = &config.keys[0];
        assert_eq!(git_node.key, "g");
        assert!(git_node.is_loop);
    }

    #[test]
    #[should_panic]
    fn test_config_parsing_duplicate_ids() {
        let yaml = r#"
keys:
  - key: g
    value: git
    keys:
      - key: s
        value: status
      - key: s
        value: stash
"#;
        let _ = Config::from_contents(yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_config_more_than_one_action() {
        let yaml = r#"
keys:
  - key: g
    value: git
    choices:
      - option1
      - option2
    keys:
      - key: s
        value: status
"#;
        let _ = Config::from_contents(yaml).unwrap();
    }

    mod local_config {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        fn create_temp_dir() -> tempfile::TempDir {
            tempfile::tempdir().unwrap()
        }

        fn write_local_config(dir: &Path, name: &str, contents: &str, executable: bool) {
            let path = dir.join(name);
            fs::write(&path, contents).unwrap();
            if executable {
                let mut perms = fs::metadata(&path).unwrap().permissions();
                perms.set_mode(perms.mode() | 0o111);
                fs::set_permissions(&path, perms).unwrap();
            }
        }

        fn base_config() -> Config {
            Config::from_contents(
                r#"
keys:
  - key: g
    value: git
"#,
            )
            .unwrap()
        }

        #[test]
        fn test_local_config_loads_executable_file() {
            let dir = create_temp_dir();
            write_local_config(
                dir.path(),
                ".wcmdrc.yml",
                "keys:\n  - key: t\n    value: test\n",
                true,
            );

            let config = base_config()
                .with_local_config_from_dir(dir.path())
                .unwrap();

            assert_eq!(config.keys.len(), 2);
            let local_node = config.keys.iter().find(|n| n.key == ".").unwrap();
            assert_eq!(local_node.name, "local");
            assert_eq!(local_node.keys.len(), 1);
            assert_eq!(local_node.keys[0].key, "t");
        }

        #[test]
        fn test_local_config_skips_non_executable() {
            let dir = create_temp_dir();
            write_local_config(
                dir.path(),
                ".wcmdrc.yml",
                "keys:\n  - key: t\n    value: test\n",
                false,
            );

            let config = base_config()
                .with_local_config_from_dir(dir.path())
                .unwrap();

            assert_eq!(config.keys.len(), 1);
        }

        #[test]
        fn test_local_config_skips_missing() {
            let dir = create_temp_dir();

            let config = base_config()
                .with_local_config_from_dir(dir.path())
                .unwrap();

            assert_eq!(config.keys.len(), 1);
        }

        #[test]
        fn test_local_config_tries_all_file_names() {
            for name in LOCAL_CONFIG_FILE_NAMES {
                let dir = create_temp_dir();
                write_local_config(
                    dir.path(),
                    name,
                    "keys:\n  - key: x\n    value: xval\n",
                    true,
                );

                let config = base_config()
                    .with_local_config_from_dir(dir.path())
                    .unwrap();

                assert_eq!(config.keys.len(), 2, "Failed for file name: {}", name);
            }
        }

        #[test]
        fn test_local_config_first_match_wins() {
            let dir = create_temp_dir();
            write_local_config(
                dir.path(),
                ".wcmdrc.yml",
                "keys:\n  - key: a\n    value: first\n",
                true,
            );
            write_local_config(
                dir.path(),
                ".wcmdrc.yaml",
                "keys:\n  - key: b\n    value: second\n",
                true,
            );

            let config = base_config()
                .with_local_config_from_dir(dir.path())
                .unwrap();

            let local_node = config.keys.iter().find(|n| n.key == ".").unwrap();
            assert_eq!(local_node.keys[0].key, "a");
        }

        #[test]
        fn test_local_config_sets_ids() {
            let dir = create_temp_dir();
            write_local_config(
                dir.path(),
                ".wcmdrc.yml",
                "keys:\n  - key: t\n    value: test\n    keys:\n      - key: r\n        value: run\n",
                true,
            );

            let config = base_config()
                .with_local_config_from_dir(dir.path())
                .unwrap();

            let local_node = config.keys.iter().find(|n| n.key == ".").unwrap();
            assert_eq!(local_node.id, ".");
            assert_eq!(local_node.keys[0].id, ".t");
            assert_eq!(local_node.keys[0].keys[0].id, ".tr");
        }
    }
}
