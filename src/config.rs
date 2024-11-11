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
