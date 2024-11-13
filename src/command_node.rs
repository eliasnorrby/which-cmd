use serde::Deserialize;

#[derive(Debug)]
pub struct CommandNode {
    pub key: String,
    pub name: String,
    pub value: String,
    pub reset: bool,
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
            value: String,
            #[serde(default)]
            reset: bool,
            #[serde(default)]
            keys: Vec<CommandNode>,
        }

        let helper = CommandNodeHelper::deserialize(deserializer)?;
        let name = helper.name.unwrap_or_else(|| helper.value.clone());

        Ok(CommandNode {
            key: helper.key,
            name,
            value: helper.value,
            reset: helper.reset,
            keys: helper.keys,
        })
    }
}

impl CommandNode {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty()
    }
}
