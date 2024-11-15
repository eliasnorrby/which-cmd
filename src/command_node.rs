use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct CommandNode {
    pub key: String,
    pub name: String,
    pub value: String,
    pub reset: bool,
    pub is_loop: bool,
    pub keys: Vec<CommandNode>,
}

// Implement custom deserialization for CommandNode
impl<'de> Deserialize<'de> for CommandNode {
    fn deserialize<D>(deserializer: D) -> Result<CommandNode, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a helper struct with optional name
        #[derive(Deserialize)]
        struct CommandNodeHelper {
            key: String,
            name: Option<String>,
            value: Option<String>,
            #[serde(default)]
            reset: bool,
            #[serde(default)]
            r#loop: bool,
            #[serde(default)]
            keys: Vec<CommandNode>,
        }

        let helper = CommandNodeHelper::deserialize(deserializer)?;
        let value = helper.value.unwrap_or_else(|| "".to_string());
        let name = helper.name.unwrap_or_else(|| value.clone());

        Ok(CommandNode {
            key: helper.key,
            name,
            value,
            reset: helper.reset,
            is_loop: helper.r#loop,
            keys: helper.keys,
        })
    }
}

impl CommandNode {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty()
    }
}
