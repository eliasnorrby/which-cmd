use serde::Deserialize;

use crate::constants::{CHOICE_KEY, INPUT_KEY};

#[derive(Debug, Clone)]
pub struct ConfigNode {
    pub id: String,
    pub key: String,
    pub name: String,
    pub value: String,
    pub is_immediate: bool,
    pub is_fleeting: bool,
    pub is_anchor: bool,
    pub is_loop: bool,
    pub is_repeatable: bool,
    pub keys: Vec<ConfigNode>,
    pub choices: Vec<String>,
    pub input_type: Option<InputType>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum InputType {
    Text,
    Number,
}

// Implement custom deserialization for ConfigNode
impl<'de> Deserialize<'de> for ConfigNode {
    fn deserialize<D>(deserializer: D) -> Result<ConfigNode, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a helper struct with optional name
        #[derive(Deserialize)]
        struct ConfigNodeHelper {
            key: String,
            name: Option<String>,
            value: Option<String>,
            #[serde(default)]
            immediate: bool,
            #[serde(default)]
            fleeting: bool,
            #[serde(default)]
            anchor: bool,
            #[serde(default)]
            r#loop: bool,
            #[serde(default)]
            keys: Vec<ConfigNode>,
            #[serde(default)]
            repeatable: bool,
            #[serde(default)]
            choices: Vec<String>,
            input: Option<InputType>,
        }

        let helper = ConfigNodeHelper::deserialize(deserializer)?;
        let value = helper.value.unwrap_or_else(|| "".to_string());
        let name = helper.name.unwrap_or_else(|| value.clone());

        if name.is_empty() {
            return Err(serde::de::Error::custom("name must not be empty"));
        }

        Ok(ConfigNode {
            // Initialize id with empty string. This will be set later by traversing the tree.
            id: "".to_string(),
            key: helper.key,
            name,
            value,
            is_immediate: helper.immediate,
            is_fleeting: helper.fleeting,
            is_anchor: helper.anchor,
            is_loop: helper.r#loop,
            is_repeatable: helper.repeatable,
            keys: helper.keys,
            choices: helper.choices,
            input_type: helper.input,
        })
    }
}

impl ConfigNode {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty() && !self.has_choices() && !self.input_type.is_some()
    }

    pub fn has_choices(&self) -> bool {
        !self.choices.is_empty()
    }

    pub fn set_id_from_parent(&mut self, parent_id: &str) {
        self.id = ConfigNode::id_from_parent(parent_id, &self.key);
    }

    pub fn id_from_parent(parent_id: &str, key: &str) -> String {
        if parent_id != "" {
            format!("{}{}", parent_id, key)
        } else {
            key.to_string()
        }
    }

    pub fn with_selection(&self, choice: usize) -> ConfigNode {
        let selection = self.choices.get(choice).unwrap();

        ConfigNode {
            id: ConfigNode::id_from_parent(&self.id, CHOICE_KEY),
            key: CHOICE_KEY.to_string(),
            name: selection.to_string(),
            value: selection.to_string(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        }
    }

    pub fn with_input(&self, input: &str) -> ConfigNode {
        ConfigNode {
            id: ConfigNode::id_from_parent(&self.id, input),
            key: INPUT_KEY.to_string(),
            name: input.to_string(),
            value: input.to_string(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        }
    }
}
