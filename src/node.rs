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

    #[must_use]
    pub fn with_selection(&self, choice: usize) -> Option<Node> {
        let selection = self.choices.get(choice)?;

        Some(Node {
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
        })
    }

    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, key: &str, name: &str, value: &str) -> Node {
        Node {
            id: id.to_string(),
            key: key.to_string(),
            name: name.to_string(),
            value: value.to_string(),
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

    #[test]
    fn test_is_leaf_with_no_children() {
        let node = create_test_node("g", "g", "git", "git");
        assert!(node.is_leaf());
    }

    #[test]
    fn test_is_leaf_with_keys() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.keys
            .push(create_test_node("gs", "s", "status", "status"));
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_is_leaf_with_choices() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.choices = vec!["option1".to_string(), "option2".to_string()];
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_is_leaf_with_input() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.input_type = Some(InputType::Text);
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_has_choices_true() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.choices = vec!["option1".to_string()];
        assert!(node.has_choices());
    }

    #[test]
    fn test_has_choices_false() {
        let node = create_test_node("g", "g", "git", "git");
        assert!(!node.has_choices());
    }

    #[test]
    fn test_id_from_parent_with_parent() {
        let id = Node::id_from_parent("git", "s");
        assert_eq!(id, "gits");
    }

    #[test]
    fn test_id_from_parent_without_parent() {
        let id = Node::id_from_parent("", "g");
        assert_eq!(id, "g");
    }

    #[test]
    fn test_set_id_from_parent() {
        let mut node = create_test_node("", "s", "status", "status");
        node.set_id_from_parent("git");
        assert_eq!(node.id, "gits");
    }

    #[test]
    fn test_with_selection_valid_index() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.choices = vec!["branch".to_string(), "commit".to_string()];

        let selected = node.with_selection(0);
        assert!(selected.is_some());

        let selected_node = selected.unwrap();
        assert_eq!(selected_node.key, CHOICE_KEY);
        assert_eq!(selected_node.name, "branch");
        assert_eq!(selected_node.value, "branch");
        assert_eq!(selected_node.id, format!("g{}", CHOICE_KEY));
    }

    #[test]
    fn test_with_selection_invalid_index() {
        let mut node = create_test_node("g", "g", "git", "git");
        node.choices = vec!["branch".to_string()];

        let selected = node.with_selection(5);
        assert!(selected.is_none());
    }

    #[test]
    fn test_with_input() {
        let node = create_test_node("g", "g", "git", "git");
        let input_node = node.with_input("my-branch-name");

        assert_eq!(input_node.key, INPUT_KEY);
        assert_eq!(input_node.name, "my-branch-name");
        assert_eq!(input_node.value, "my-branch-name");
        assert_eq!(input_node.id, format!("gmy-branch-name"));
    }

    #[test]
    fn test_fleeting_flag_with_choices() {
        let yaml = r#"
key: g
value: git
choices:
  - branch
  - commit
"#;
        let node: Node = serde_yaml::from_str(yaml).unwrap();
        assert!(node.is_fleeting, "Nodes with choices should be fleeting");
    }

    #[test]
    fn test_fleeting_flag_with_input() {
        let yaml = r#"
key: b
value: branch
input: Text
"#;
        let node: Node = serde_yaml::from_str(yaml).unwrap();
        assert!(node.is_fleeting, "Nodes with input should be fleeting");
    }

    #[test]
    fn test_fleeting_flag_explicit() {
        let yaml = r#"
key: g
value: git
fleeting: true
"#;
        let node: Node = serde_yaml::from_str(yaml).unwrap();
        assert!(
            node.is_fleeting,
            "Explicitly fleeting nodes should be fleeting"
        );
    }
}
