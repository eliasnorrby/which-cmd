use serde::Deserialize;

// TODO: add a unique id built from keys
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigNode {
    pub key: String,
    pub name: String,
    pub value: String,
    pub is_immediate: bool,
    pub is_fleeting: bool,
    pub is_anchor: bool,
    pub is_loop: bool,
    pub keys: Vec<ConfigNode>,
    pub choices: Vec<ConfigNode>,
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
            choices: Vec<String>,
        }

        let helper = ConfigNodeHelper::deserialize(deserializer)?;
        let value = helper.value.unwrap_or_else(|| "".to_string());
        let name = helper.name.unwrap_or_else(|| value.clone());
        let choices = helper
            .choices
            .iter()
            .map(|choice| ConfigNode {
                key: "[choice]".to_string(),
                name: choice.clone(),
                value: choice.clone(),
                is_immediate: false,
                is_fleeting: false,
                is_anchor: false,
                is_loop: false,
                keys: vec![],
                choices: vec![],
            })
            .collect();

        if name.is_empty() {
            return Err(serde::de::Error::custom("name must not be empty"));
        }

        Ok(ConfigNode {
            key: helper.key,
            name,
            value,
            is_immediate: helper.immediate,
            is_fleeting: helper.fleeting,
            is_anchor: helper.anchor,
            is_loop: helper.r#loop,
            keys: helper.keys,
            choices,
        })
    }
}

impl ConfigNode {
    pub fn is_leaf(&self) -> bool {
        self.keys.is_empty() && !self.has_choices()
    }

    pub fn has_choices(&self) -> bool {
        !self.choices.is_empty()
    }
}
