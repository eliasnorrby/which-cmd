use crate::{condition::ConditionEvaluator, node::Node, path::compose_command};
use std::rc::Rc;

pub struct SearchNode {
    pub id: String,
    pub command: String,
    pub name: String,
}

pub fn format_search_options(nodes: &[SearchNode]) -> Vec<String> {
    let longest_command = nodes
        .iter()
        .map(|node| node.command.len())
        .max()
        .unwrap_or(0);
    let longest_name = nodes.iter().map(|node| node.name.len()).max().unwrap_or(0);
    let textoptions: Vec<String> = nodes
        .iter()
        .map(|n| format_single_search_option(n, longest_command, longest_name))
        .collect();
    textoptions
}

pub fn format_single_search_option(
    node: &SearchNode,
    command_length: usize,
    name_length: usize,
) -> String {
    format!(
        "{:<cl$}   {:<nl$}   {}",
        &node.command,
        &node.name,
        node.id
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" > "),
        cl = command_length,
        nl = name_length
    )
}

pub fn get_search_options(
    nodes: &[Rc<Node>],
    evaluator: &mut ConditionEvaluator,
) -> Vec<SearchNode> {
    let visible: Vec<_> = nodes
        .iter()
        .filter(|node| match &node.when {
            Some(condition) => evaluator.evaluate(condition),
            None => true,
        })
        .cloned()
        .collect();

    visible
        .iter()
        .flat_map(|node| get_search_options_recursively(&node.keys, &[Rc::clone(node)], evaluator))
        .collect()
}

fn compose_name(path: &[Rc<Node>]) -> String {
    path.iter()
        .map(|n| n.name.as_str())
        .collect::<Vec<_>>()
        .join(" > ")
}

pub fn get_search_options_recursively(
    nodes: &[Rc<Node>],
    path: &[Rc<Node>],
    evaluator: &mut ConditionEvaluator,
) -> Vec<SearchNode> {
    let visible: Vec<_> = nodes
        .iter()
        .filter(|node| match &node.when {
            Some(condition) => evaluator.evaluate(condition),
            None => true,
        })
        .cloned()
        .collect();

    visible
        .iter()
        .flat_map(|node| {
            let new_path: Vec<Rc<Node>> = path
                .iter()
                .map(Rc::clone)
                .chain(std::iter::once(Rc::clone(node)))
                .collect();
            let command = compose_command(&new_path);
            let name = compose_name(&new_path);

            let mut search_nodes = vec![SearchNode {
                id: node.id.clone(),
                command,
                name,
            }];

            if !node.keys.is_empty() {
                search_nodes.extend(get_search_options_recursively(
                    &node.keys, &new_path, evaluator,
                ));
            }

            search_nodes
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, key: &str, value: &str, children: Vec<Rc<Node>>) -> Rc<Node> {
        Rc::new(Node {
            id: id.to_string(),
            key: key.to_string(),
            name: value.to_string(),
            value: value.to_string(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            is_repeatable: false,
            keys: children,
            choices: vec![],
            input_type: None,
            when: None,
        })
    }

    #[test]
    fn test_format_single_search_option() {
        let search_node = SearchNode {
            id: "gs".to_string(),
            command: "git status".to_string(),
            name: "git > status".to_string(),
        };

        let formatted = format_single_search_option(&search_node, 15, 15);
        assert!(formatted.contains("git status"));
        assert!(formatted.contains("git > status"));
        assert!(formatted.contains("g > s"));
    }

    #[test]
    fn test_format_search_options_with_padding() {
        let nodes = vec![
            SearchNode {
                id: "g".to_string(),
                command: "git".to_string(),
                name: "git".to_string(),
            },
            SearchNode {
                id: "gs".to_string(),
                command: "git status".to_string(),
                name: "git > status".to_string(),
            },
        ];

        let formatted = format_search_options(&nodes);
        assert_eq!(formatted.len(), 2);

        // All commands should be padded to the same length (longest = "git status" = 10)
        // So "git" should have extra spaces
        assert!(formatted[0].starts_with("git       ")); // "git" padded to 10 chars
        assert!(formatted[1].starts_with("git status"));
        // Name column should be present
        assert!(formatted[0].contains("git"));
        assert!(formatted[1].contains("git > status"));
    }

    #[test]
    fn test_format_search_options_empty() {
        let nodes: Vec<SearchNode> = vec![];
        let formatted = format_search_options(&nodes);
        assert_eq!(formatted.len(), 0);
    }

    #[test]
    fn test_get_search_options_single_node() {
        let mut evaluator = ConditionEvaluator::new();
        let node = create_test_node("g", "g", "git", vec![]);
        let search_nodes = get_search_options(&[node], &mut evaluator);

        // get_search_options only returns children, not the node itself
        // A node with no children returns an empty list
        assert_eq!(search_nodes.len(), 0);
    }

    #[test]
    fn test_get_search_options_nested() {
        let mut evaluator = ConditionEvaluator::new();
        let child = create_test_node("gs", "s", "status", vec![]);
        let parent = create_test_node("g", "g", "git", vec![child]);

        let search_nodes = get_search_options(&[parent], &mut evaluator);

        // Should only have the child (not the parent itself)
        assert_eq!(search_nodes.len(), 1);

        // Should be the child
        assert_eq!(search_nodes[0].id, "gs");
        assert_eq!(search_nodes[0].command, "git status");
        assert_eq!(search_nodes[0].name, "git > status");
    }

    #[test]
    fn test_get_search_options_multiple_children() {
        let mut evaluator = ConditionEvaluator::new();
        let child1 = create_test_node("gs", "s", "status", vec![]);
        let child2 = create_test_node("gc", "c", "commit", vec![]);
        let parent = create_test_node("g", "g", "git", vec![child1, child2]);

        let search_nodes = get_search_options(&[parent], &mut evaluator);

        // Should have 2 children (not the parent)
        assert_eq!(search_nodes.len(), 2);

        // Should include both children
        let ids: Vec<&str> = search_nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"gs"));
        assert!(ids.contains(&"gc"));
    }

    #[test]
    fn test_get_search_options_deeply_nested() {
        let mut evaluator = ConditionEvaluator::new();
        let grandchild = create_test_node("gca", "a", "--amend", vec![]);
        let child = create_test_node("gc", "c", "commit", vec![grandchild]);
        let parent = create_test_node("g", "g", "git", vec![child]);

        let search_nodes = get_search_options(&[parent], &mut evaluator);

        // Should have child and grandchild (2 total, not the root parent)
        assert_eq!(search_nodes.len(), 2);

        // Should include the deeply nested node
        let deepest = search_nodes.iter().find(|n| n.id == "gca");
        assert!(deepest.is_some());
        assert_eq!(deepest.unwrap().command, "git commit --amend");
    }

    #[test]
    fn test_get_search_options_recursively_builds_path() {
        let mut evaluator = ConditionEvaluator::new();
        let child = create_test_node("s", "s", "status", vec![]);
        let parent_node = create_test_node("g", "g", "git", vec![]);

        let search_nodes = get_search_options_recursively(&[child], &[parent_node], &mut evaluator);

        assert_eq!(search_nodes.len(), 1);
        assert_eq!(search_nodes[0].command, "git status");
    }
}
