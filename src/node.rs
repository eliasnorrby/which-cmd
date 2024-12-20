use serde::Deserialize;

use crate::constants::{CHOICE_KEY, INPUT_KEY};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub key: String,
    pub name: String,
    pub value: String,
    pub is_immediate: bool,
    pub is_fleeting: bool,
    pub is_anchor: bool,
    pub is_loop: bool,
    pub is_repeatable: bool,
    pub keys: Vec<Node>,
    pub choices: Vec<String>,
    pub input_type: Option<InputType>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum InputType {
    Text,
    Number,
}

// Implement custom deserialization for Node
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Node, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a helper struct with optional name
        #[derive(Deserialize)]
        struct NodeHelper {
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
            keys: Vec<Node>,
            #[serde(default)]
            repeatable: bool,
            #[serde(default)]
            choices: Vec<String>,
            input: Option<InputType>,
        }

        let helper = NodeHelper::deserialize(deserializer)?;
        let value = helper.value.unwrap_or_else(|| "".to_string());
        let name = helper.name.unwrap_or_else(|| value.clone());

        if name.is_empty() {
            return Err(serde::de::Error::custom("name must not be empty"));
        }

        if [
            !helper.choices.is_empty(),
            helper.input.is_some(),
            !helper.keys.is_empty(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
            > 1
        {
            return Err(serde::de::Error::custom(format!(
                "node must have only one of choices, input, or keys: {}",
                name
            )));
        }

        Ok(Node {
            // Initialize id with empty string. This will be set later by traversing the tree.
            id: "".to_string(),
            key: helper.key,
            name,
            value,
            is_immediate: helper.immediate,
            is_fleeting: helper.fleeting || helper.input.is_some() || !helper.choices.is_empty(),
            is_anchor: helper.anchor,
            is_loop: helper.r#loop,
            is_repeatable: helper.repeatable,
            keys: helper.keys,
            choices: helper.choices,
            input_type: helper.input,
        })
    }
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty() && !self.has_choices() && self.input_type.is_none()
    }

    pub fn has_choices(&self) -> bool {
        !self.choices.is_empty()
    }

    pub fn set_id_from_parent(&mut self, parent_id: &str) {
        self.id = Node::id_from_parent(parent_id, &self.key);
    }

    pub fn id_from_parent(parent_id: &str, key: &str) -> String {
        if !parent_id.is_empty() {
            format!("{}{}", parent_id, key)
        } else {
            key.to_string()
        }
    }

    pub fn with_selection(&self, choice: usize) -> Node {
        let selection = self.choices.get(choice).unwrap();

        Node {
            id: Node::id_from_parent(&self.id, CHOICE_KEY),
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

    pub fn with_input(&self, input: &str) -> Node {
        Node {
            id: Node::id_from_parent(&self.id, input),
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
