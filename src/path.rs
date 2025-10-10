use crate::node::Node;

pub fn pop_to_first_non_is_fleeting(path: &mut Vec<Node>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

#[must_use]
pub fn compose_command(path: &[Node]) -> String {
    // Start building the command from the last anchor point
    let mut command_parts = Vec::new();
    let mut start_index = 0;
    for (i, node) in path.iter().enumerate() {
        if node.is_anchor {
            start_index = i;
        }
    }
    for node in &path[start_index..] {
        command_parts.push(node.value.as_str());
    }
    command_parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;

    #[test]
    fn test_compose_command_no_anchor() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "s".into(),
            key: "s".into(),
            name: "status".into(),
            value: "status".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let path = vec![node1, node2];
        let command = compose_command(&path);
        assert_eq!(command, "git status");
    }

    #[test]
    fn test_compose_command_with_anchor() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "h".into(),
            key: "h".into(),
            name: "GitHub".into(),
            value: "gh".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: true,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node3 = Node {
            id: "p".into(),
            key: "p".into(),
            name: "pull request".into(),
            value: "pr".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let path = vec![node1, node2, node3];
        let command = compose_command(&path);
        assert_eq!(command, "gh pr");
    }

    #[test]
    fn test_compose_command_empty_path() {
        let path: Vec<Node> = vec![];
        let command = compose_command(&path);
        assert_eq!(command, "");
    }

    #[test]
    fn test_compose_command_single_node() {
        let node = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let path = vec![node];
        let command = compose_command(&path);
        assert_eq!(command, "git");
    }

    #[test]
    fn test_compose_command_with_empty_values() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "s".into(),
            key: "s".into(),
            name: "status".into(),
            value: "".into(), // Empty value
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let path = vec![node1, node2];
        let command = compose_command(&path);
        assert_eq!(command, "git ");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_empty_path() {
        let mut path: Vec<Node> = vec![];
        pop_to_first_non_is_fleeting(&mut path);
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_no_fleeting() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "s".into(),
            key: "s".into(),
            name: "status".into(),
            value: "status".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };

        let mut path = vec![node1.clone(), node2.clone()];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop the last node (s) and put it back, so both nodes remain
        assert_eq!(path.len(), 2);
        assert_eq!(path[1].id, "s");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_with_fleeting() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "choice".into(),
            key: "c".into(),
            name: "choice".into(),
            value: "branch-name".into(),
            is_immediate: false,
            is_fleeting: true, // Fleeting node
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };

        let mut path = vec![node1.clone(), node2];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop through fleeting nodes and stop at the first non-fleeting
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].id, "g");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_multiple_fleeting() {
        let node1 = Node {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = Node {
            id: "f1".into(),
            key: "f1".into(),
            name: "fleeting1".into(),
            value: "fleeting1".into(),
            is_immediate: false,
            is_fleeting: true,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node3 = Node {
            id: "f2".into(),
            key: "f2".into(),
            name: "fleeting2".into(),
            value: "fleeting2".into(),
            is_immediate: false,
            is_fleeting: true,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };

        let mut path = vec![node1.clone(), node2, node3];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop through all fleeting nodes
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].id, "g");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_all_fleeting() {
        let node1 = Node {
            id: "f1".into(),
            key: "f1".into(),
            name: "fleeting1".into(),
            value: "fleeting1".into(),
            is_immediate: false,
            is_fleeting: true,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };

        let mut path = vec![node1];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop all fleeting nodes, leaving path empty
        assert_eq!(path.len(), 0);
    }
}
