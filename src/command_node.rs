use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommandNode {
    pub key: String,
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub reset: bool,
    #[serde(default)]
    pub keys: Vec<CommandNode>,
}

impl CommandNode {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty()
    }
}
