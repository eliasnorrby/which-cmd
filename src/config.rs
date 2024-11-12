use serde::Deserialize;

use crate::command_node::CommandNode;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub keys: Vec<CommandNode>,
}

use std::fs;

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
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
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.keys.len(), 1);
        let git_node = &config.keys[0];
        assert_eq!(git_node.key, "g");
        assert_eq!(git_node.name, "git");
        assert_eq!(git_node.value, "git");
        assert_eq!(git_node.keys.len(), 1);
        let status_node = &git_node.keys[0];
        assert_eq!(status_node.key, "s");
        assert_eq!(status_node.name, "status");
        assert_eq!(status_node.value, "status");
    }
}
