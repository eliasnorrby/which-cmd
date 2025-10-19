use serde::Deserialize;
use std::fs;
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

        let contents = fs::read_to_string(config_path)?;

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
}
