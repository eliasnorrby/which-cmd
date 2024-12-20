use crate::{node::Node, path::compose_command};

pub struct SearchNode {
    pub id: String,
    pub command: String,
}

pub fn format_search_options(nodes: &[SearchNode]) -> Vec<String> {
    let longest_command = nodes
        .iter()
        .map(|node| node.command.len())
        .max()
        .unwrap_or(0);
    let textoptions: Vec<String> = nodes
        .iter()
        .map(|n| format_single_search_option(n, longest_command))
        .collect();
    textoptions
}

pub fn format_single_search_option(node: &SearchNode, command_length: usize) -> String {
    format!(
        "{:<length$} {}",
        &node.command,
        node.id
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" > "),
        length = command_length
    )
}

pub fn get_search_options(nodes: &[Node]) -> Vec<SearchNode> {
    nodes
        .iter()
        .flat_map(|node| get_search_options_recursively(&node.keys, &[node.clone()]))
        .collect()
}

pub fn get_search_options_recursively(nodes: &[Node], path: &[Node]) -> Vec<SearchNode> {
    nodes
        .iter()
        .flat_map(|node| {
            let new_path: Vec<Node> = path
                .iter()
                .cloned()
                .chain(std::iter::once(node.clone()))
                .collect();
            let command = compose_command(&new_path);

            let mut search_nodes = vec![SearchNode {
                id: node.id.clone(),
                command,
            }];

            if !node.keys.is_empty() {
                search_nodes.extend(get_search_options_recursively(&node.keys, &new_path));
            }

            search_nodes
        })
        .collect()
}
