use crate::node::Node;

pub fn pop_to_first_non_is_fleeting(path: &mut Vec<Node>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

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
}
