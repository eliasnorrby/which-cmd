pub struct SearchNode {
    pub id: String,
    pub command: String,
}

pub fn format_search_options(nodes: &Vec<SearchNode>) -> Vec<String> {
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
