use crate::node::Node;
use std::rc::Rc;

pub fn pop_to_first_non_is_fleeting(path: &mut Vec<Rc<Node>>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

#[must_use]
pub fn compose_command(path: &[Rc<Node>]) -> String {
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

    fn create_test_node(
        id: &str,
        key: &str,
        name: &str,
        value: &str,
        is_anchor: bool,
        is_fleeting: bool,
    ) -> Rc<Node> {
        Rc::new(Node {
            id: id.into(),
            key: key.into(),
            name: name.into(),
            value: value.into(),
            is_immediate: false,
            is_fleeting,
            is_anchor,
            is_loop: false,
            is_repeatable: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        })
    }

    #[test]
    fn test_compose_command_no_anchor() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = create_test_node("s", "s", "status", "status", false, false);
        let path = vec![node1, node2];
        let command = compose_command(&path);
        assert_eq!(command, "git status");
    }

    #[test]
    fn test_compose_command_with_anchor() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = create_test_node("h", "h", "GitHub", "gh", true, false);
        let node3 = create_test_node("p", "p", "pull request", "pr", false, false);
        let path = vec![node1, node2, node3];
        let command = compose_command(&path);
        assert_eq!(command, "gh pr");
    }

    #[test]
    fn test_compose_command_empty_path() {
        let path: Vec<Rc<Node>> = vec![];
        let command = compose_command(&path);
        assert_eq!(command, "");
    }

    #[test]
    fn test_compose_command_single_node() {
        let node = create_test_node("g", "g", "git", "git", false, false);
        let path = vec![node];
        let command = compose_command(&path);
        assert_eq!(command, "git");
    }

    #[test]
    fn test_compose_command_with_empty_values() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = Rc::new(Node {
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
        });
        let path = vec![node1, node2];
        let command = compose_command(&path);
        assert_eq!(command, "git ");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_empty_path() {
        let mut path: Vec<Rc<Node>> = vec![];
        pop_to_first_non_is_fleeting(&mut path);
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_no_fleeting() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = create_test_node("s", "s", "status", "status", false, false);

        let mut path = vec![Rc::clone(&node1), Rc::clone(&node2)];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop the last node (s) and put it back, so both nodes remain
        assert_eq!(path.len(), 2);
        assert_eq!(path[1].id, "s");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_with_fleeting() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = create_test_node("choice", "c", "choice", "branch-name", false, true);

        let mut path = vec![Rc::clone(&node1), node2];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop through fleeting nodes and stop at the first non-fleeting
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].id, "g");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_multiple_fleeting() {
        let node1 = create_test_node("g", "g", "git", "git", false, false);
        let node2 = create_test_node("f1", "f1", "fleeting1", "fleeting1", false, true);
        let node3 = create_test_node("f2", "f2", "fleeting2", "fleeting2", false, true);

        let mut path = vec![Rc::clone(&node1), node2, node3];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop through all fleeting nodes
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].id, "g");
    }

    #[test]
    fn test_pop_to_first_non_is_fleeting_all_fleeting() {
        let node1 = create_test_node("f1", "f1", "fleeting1", "fleeting1", false, true);

        let mut path = vec![node1];
        pop_to_first_non_is_fleeting(&mut path);

        // Should pop all fleeting nodes, leaving path empty
        assert_eq!(path.len(), 0);
    }
}
