use serde::Deserialize;
use std::fs;

use crate::config_node::ConfigNode;
use crate::constants::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub keys: Vec<ConfigNode>,
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
        fn set_id(node: &mut ConfigNode, parent_id: &str) {
            node.id = format!("{}{}", parent_id, node.key);
            for child in node.keys.iter_mut() {
                set_id(child, &node.id);
            }
        }

        for node in config.keys.iter_mut() {
            set_id(node, "");
        }

        Ok(config)
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
        assert_eq!(git_node.is_loop, false);
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
        assert_eq!(git_node.is_loop, true);
    }
}
