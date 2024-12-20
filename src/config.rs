use serde::Deserialize;
use std::fs;

use crate::constants::*;
use crate::node::Node;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub keys: Vec<Node>,
}

impl Config {
    pub fn from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
        let config_path = match xdg_dirs.find_config_file(CONFIG_FILE_NAME) {
            Some(path) => path,
            None => {
                eprintln!(
                    "Configuration file not found at {}{}",
                    xdg_dirs.get_config_home().display(),
                    CONFIG_FILE_NAME
                );
                std::process::exit(1);
            }
        };

        let contents = fs::read_to_string(config_path)?;

        Config::from_contents(&contents)
    }

    fn from_contents(contents: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config: Config = serde_yaml::from_str(contents)?;

        // Recursively loop through the config and set the id of each node.
        // It should be a concatenation of the keys of all the parent nodes
        // and the key of the current node.
        fn set_id(node: &mut Node, parent_id: &str) {
            node.set_id_from_parent(parent_id);
            Config::ensure_unique(&node.id, node.keys.iter().map(|node| &node.key).collect());
            for child in node.keys.iter_mut() {
                set_id(child, &node.id);
            }
        }

        Config::ensure_unique(
            &"".to_string(),
            config.keys.iter().map(|node| &node.key).collect(),
        );

        for node in config.keys.iter_mut() {
            set_id(node, "");
        }

        Ok(config)
    }

    fn ensure_unique(parent_id: &String, keys: Vec<&String>) {
        let mut seen = std::collections::HashSet::new();
        for key in keys {
            if seen.contains(key) {
                eprintln!("Conflicting keys found: {}{}", parent_id, key);
                panic!("Conflicting keys found");
            }
            seen.insert(key);
        }
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
