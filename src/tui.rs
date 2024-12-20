use crate::config::Config;
use crate::constants::NUMBER_OF_ROWS;
use crate::node::Node;
use crate::options::Options;
use crate::search::{format_search_options, SearchNode};
use crate::terminal::Terminal;

use crossterm::{
    event::{self, Event, KeyCode},
    style::Stylize,
};

const IMMEDIATE_PREFIX: &str = "__IMMEDIATE__";

fn pop_to_first_non_is_fleeting(path: &mut Vec<Node>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

fn format_node(node: &Node, opts: &Options) -> String {
    let sub_keys_count = node.keys.len();
    if sub_keys_count > 0 {
        format!(
            "{} {} {}",
            node.key.to_string().bold(),
            "•".dark_grey(),
            format!("{:<10} +{}", node.name, sub_keys_count).blue()
        )
    } else {
        let include_immediate_tag = opts.print_immediate_tag && node.is_immediate;
        format!(
            "{} {} {} {}",
            node.key.to_string().bold(),
            "•".dark_grey(),
            format!("{:<10}", node.name).yellow(),
            if include_immediate_tag { "↵" } else { "" }
        )
    }
}

pub fn format_all_nodes(nodes: &[Node]) -> Vec<SearchNode> {
    let mut formatted = vec![];
    for node in nodes {
        let path = vec![node.clone()];
        let sublist = format_nodes_recursive(&node.keys, &path);
        formatted.extend(sublist);
    }
    formatted
}

pub fn format_nodes_recursive(nodes: &[Node], path: &[Node]) -> Vec<SearchNode> {
    let mut list = vec![];
    for node in nodes {
        let newpath = path
            .iter()
            .chain(std::iter::once(node))
            .cloned()
            .collect::<Vec<Node>>();
        let command = compose_command(&newpath);
        list.push(SearchNode {
            id: node.id.clone(),
            command,
        });
        if node.keys.len() > 0 {
            let sublist = format_nodes_recursive(&node.keys, &newpath);
            list.extend(sublist);
        }
    }
    list
}

fn highlight_command(command: &str) -> String {
    let mut highlighted: String = "".to_string();
    let parts = command.split(' ').collect::<Vec<&str>>();
    for part in parts.iter() {
        highlighted.push_str(&format!(
            "{} ",
            if part.starts_with('-') {
                part.cyan()
            } else if highlighted == "" {
                part.green()
            } else {
                part.yellow()
            }
        ));
    }
    highlighted
}

fn command_indicator(path: &[Node]) -> String {
    format!(
        "{} {}",
        "Command:".grey(),
        highlight_command(&compose_command(&path))
    )
}

pub fn run_tui(config: Config, opts: Options) -> Result<String, Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut terminal = Terminal::new(std::io::stdout());

    terminal.setup()?;

    let mut path: Vec<Node> = Vec::new();
    let mut loop_node_index: Option<usize> = None;

    loop {
        terminal.clear_screen()?;

        // Display the current path
        if !path.is_empty() {
            terminal.write_line(&command_indicator(&path))?;
            terminal.blank_line()?;
            let keys_pressed: Vec<&str> = path.iter().map(|node| node.key.as_str()).collect();
            terminal.write_line(&format!(
                "{} {}",
                "Keys pressed:".grey(),
                keys_pressed.join(&" > ".dark_grey().to_string())
            ))?;
            terminal.blank_line()?;
        } else {
            terminal.write_line(&format!("{}", "Press a key to select an option".grey()))?;
            terminal.blank_line()?;
            terminal.write_line(&format!("{}", "Available commands:".grey()))?;
            terminal.blank_line()?;
        }

        // TODO: make configurable
        let num_rows = NUMBER_OF_ROWS;

        let current_nodes = if let Some(l) = loop_node_index {
            if let Some(last_node) = path.last() {
                if last_node.is_leaf() {
                    path[l]
                        .keys
                        .iter()
                        .filter(|n| n.is_repeatable || !path.iter().any(|p| p.id == n.id))
                        .cloned()
                        .collect()
                } else {
                    last_node.keys.clone()
                }
            } else {
                config.keys.clone()
            }
        } else if let Some(last_node) = path.last() {
            last_node.keys.clone()
        } else {
            config.keys.clone()
        };

        // Sort the current_nodes before displaying them
        let mut sorted_nodes = current_nodes.to_vec();
        sorted_nodes.sort_by(|a, b| {
            let a_key_lower = a.key.to_lowercase();
            let b_key_lower = b.key.to_lowercase();
            match a_key_lower.cmp(&b_key_lower) {
                std::cmp::Ordering::Equal => {
                    let a_is_lower = a.key.chars().next().unwrap().is_lowercase();
                    let b_is_lower = b.key.chars().next().unwrap().is_lowercase();
                    match (a_is_lower, b_is_lower) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.key.cmp(&b.key),
                    }
                }
                other => other,
            }
        });

        // Arrange the options into rows
        let mut rows: Vec<Vec<String>> = vec![Vec::new(); num_rows];

        for (i, node) in sorted_nodes.iter().enumerate() {
            let row_index = i % num_rows;
            let display_string = format_node(node, &opts);
            rows[row_index].push(display_string);
        }

        // Determine the maximum number of columns
        let num_columns = rows.iter().map(|row| row.len()).max().unwrap_or(0);

        // Initialize column widths
        let mut column_widths = vec![0; num_columns];

        // Calculate the maximum width for each column
        for row in &rows {
            for (col_index, display_string) in row.iter().enumerate() {
                let width = display_string.len();
                if width > column_widths[col_index] {
                    column_widths[col_index] = width;
                }
            }
        }

        // Display the options in table format
        for row in &rows {
            let mut line = String::new();
            for (col_index, display_string) in row.iter().enumerate() {
                let column_width = column_widths[col_index];
                // Pad the display string to the column width
                line.push_str(&format!(
                    "{:<width$}\t",
                    display_string,
                    width = column_width
                ));
            }
            terminal.write_line(&line)?;
        }

        terminal.blank_line()?;
        terminal.write_centered(&format!(
            "󱊷  {}  󰁮  {}",
            "close".dark_grey(),
            "back".dark_grey()
        ))?;

        terminal.flush()?;

        // Wait for an event
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Esc => {
                    terminal.teardown()?;
                    return Ok("".into());
                }
                KeyCode::Char(c) => {
                    // Handle character input
                    if let Some(node) = current_nodes.iter().find(|n| n.key == c.to_string()) {
                        path.push(node.clone());
                        if node.is_loop {
                            loop_node_index = Some(path.len() - 1);
                        }
                        if node.is_leaf() {
                            if !loop_node_index.is_some() {
                                // Build and return the command
                                let command = compose_command(&path);
                                terminal.teardown()?;
                                return if opts.print_immediate_tag && node.is_immediate {
                                    Ok(format!("{} {}", IMMEDIATE_PREFIX, command))
                                } else {
                                    Ok(command)
                                };
                            }
                        } else if node.has_choices() {
                            terminal.prepare_for_input(&command_indicator(&path))?;
                            let selection = terminal.select(&node.choices)?;
                            if let Some(selection) = selection {
                                path.push(node.with_selection(selection));
                            } else {
                                pop_to_first_non_is_fleeting(&mut path);
                            }
                        } else if let Some(input_type) = &node.input_type {
                            terminal.prepare_for_input(&command_indicator(&path))?;
                            let input = terminal.input(input_type, &node.name)?;
                            path.push(node.with_input(&input.to_string()));
                        }
                        // TODO: forbid binding /
                    } else if c == '/' {
                        // Search
                        terminal.prepare_for_input(&command_indicator(&path))?;
                        let options =
                            format_all_nodes(if path.len() > 0 { &path } else { &config.keys });
                        let textoptions = format_search_options(&options);
                        let selection = terminal.select(textoptions.as_slice())?;
                        if let Some(selection) = selection {
                            let selected_node = &options[selection];

                            path = vec![];
                            let mut lookup = config.keys.clone();
                            for part in selected_node.id.split("") {
                                if part != "" {
                                    if let Some(node) = lookup.iter().find(|n| n.key == part) {
                                        path.push(node.clone());
                                        lookup = node.keys.clone();
                                    }
                                }
                            }
                        } else {
                            pop_to_first_non_is_fleeting(&mut path);
                        }
                    } else {
                        // Invalid key pressed
                        terminal.start_of_row()?;
                        terminal.write(&format!("{} {}", "Invalid key:".red(), c))?;
                        terminal.flush()?;
                        std::thread::sleep(std::time::Duration::from_millis(750));
                    }
                }
                KeyCode::Backspace => {
                    if path.pop().is_some() {
                        pop_to_first_non_is_fleeting(&mut path);

                        // If loop_node is not contained in path, unset it
                        if loop_node_index.is_some_and(|l| path.len() <= l) {
                            loop_node_index = None;
                        }
                    }
                }
                KeyCode::Enter => {
                    let command = compose_command(&path);
                    terminal.teardown()?;
                    let last_node = path.last().unwrap();
                    return if opts.print_immediate_tag && last_node.is_immediate {
                        Ok(format!("{} {}", IMMEDIATE_PREFIX, command))
                    } else {
                        Ok(command)
                    };
                }
                _ => {}
            }
        }
    }
}

fn compose_command(path: &[Node]) -> String {
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
