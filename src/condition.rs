use std::collections::HashMap;
use std::path::Path;

use crate::node::Condition;

pub struct ConditionEvaluator {
    command_cache: HashMap<String, bool>,
}

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self {
            command_cache: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, condition: &Condition) -> bool {
        match condition {
            Condition::FileExists(path) => Path::new(path).exists(),
            Condition::CommandExists(cmd) => self.command_exists(cmd),
        }
    }

    fn command_exists(&mut self, cmd: &str) -> bool {
        if let Some(&cached) = self.command_cache.get(cmd) {
            return cached;
        }

        let exists = std::env::var_os("PATH")
            .map(|path_var| std::env::split_paths(&path_var).any(|dir| dir.join(cmd).exists()))
            .unwrap_or(false);

        self.command_cache.insert(cmd.to_string(), exists);
        exists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists_true() {
        let mut evaluator = ConditionEvaluator::new();
        let condition = Condition::FileExists("Cargo.toml".to_string());
        assert!(evaluator.evaluate(&condition));
    }

    #[test]
    fn test_file_exists_false() {
        let mut evaluator = ConditionEvaluator::new();
        let condition = Condition::FileExists("nonexistent_file_xyz.txt".to_string());
        assert!(!evaluator.evaluate(&condition));
    }

    #[test]
    fn test_command_exists_true() {
        let mut evaluator = ConditionEvaluator::new();
        // "sh" should exist on any Unix system
        let condition = Condition::CommandExists("sh".to_string());
        assert!(evaluator.evaluate(&condition));
    }

    #[test]
    fn test_command_exists_false() {
        let mut evaluator = ConditionEvaluator::new();
        let condition = Condition::CommandExists("nonexistent_cmd_xyz_123".to_string());
        assert!(!evaluator.evaluate(&condition));
    }

    #[test]
    fn test_command_exists_caching() {
        let mut evaluator = ConditionEvaluator::new();
        let condition = Condition::CommandExists("sh".to_string());

        let first = evaluator.evaluate(&condition);
        let second = evaluator.evaluate(&condition);
        assert_eq!(first, second);
        assert!(evaluator.command_cache.contains_key("sh"));
    }
}
