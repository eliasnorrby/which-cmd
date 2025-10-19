use crate::config::Config;
use crate::constants::{ERROR_DISPLAY_DURATION_MS, NUMBER_OF_ROWS};
use crate::error::{Result, WhichCmdError};
use crate::node::Node;
use crate::options::Options;
use crate::path::{compose_command, pop_to_first_non_is_fleeting};
use crate::search::{format_search_options, get_search_options};
use crate::terminal::Terminal;

use crossterm::{
    event::{self, Event, KeyCode},
    style::Stylize,
};

const IMMEDIATE_PREFIX: &str = "__IMMEDIATE__";

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

fn highlight_command(command: &str) -> String {
    let mut highlighted: String = "".to_string();
    let parts = command.split(' ').collect::<Vec<&str>>();
    for part in parts.iter() {
        highlighted.push_str(&format!(
            "{} ",
            if part.starts_with('-') {
                part.cyan()
            } else if highlighted.is_empty() {
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
        highlight_command(&compose_command(path))
    )
}

pub fn run_tui(config: Config, opts: Options) -> Result<String> {
    // Initialize terminal
    let mut terminal = Terminal::new(std::io::stdout());

    terminal.set_border(opts.border);
    terminal.setup()?;

    let mut path: Vec<Node> = Vec::new();
    let mut loop_node_index: Option<usize> = None;

    loop {
        terminal.clear_screen()?;

        // Display the current path
        if !path.is_empty() {
            terminal.write_line(&command_indicator(&path))?;
            terminal.empty_border_line()?;
            let keys_pressed: Vec<&str> = path.iter().map(|node| node.key.as_str()).collect();
            terminal.write_line(&format!(
                "{} {}",
                "Keys pressed:".grey(),
                keys_pressed.join(&" > ".dark_grey().to_string())
            ))?;
            terminal.empty_border_line()?;
        } else {
            terminal.write_line(&format!("{}", "Press a key to select an option".grey()))?;
            terminal.empty_border_line()?;
            terminal.write_line(&format!("{}", "Available keys:".grey()))?;
            terminal.empty_border_line()?;
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

        terminal.empty_border_line()?;
        terminal.write_centered(&format!(
            "󱊷  {}  󰁮  {}",
            "close".dark_grey(),
            "back".dark_grey()
        ))?;
        terminal.draw_bottom_border()?;

        terminal.flush()?;

        // Wait for an event
        let key_event = event::read()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to read event: {}", e)))?;

        if let Event::Key(event) = key_event {
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
                            if loop_node_index.is_none() {
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
                            if let Some(selection_idx) = selection {
                                if let Some(selected_node) = node.with_selection(selection_idx) {
                                    path.push(selected_node);
                                } else {
                                    pop_to_first_non_is_fleeting(&mut path);
                                }
                            } else {
                                pop_to_first_non_is_fleeting(&mut path);
                            }
                        } else if let Some(input_type) = &node.input_type {
                            terminal.prepare_for_input(&command_indicator(&path))?;
                            let input = terminal.input(input_type, &node.name)?;
                            path.push(node.with_input(&input.to_string()));
                        }
                    } else if c == '/' {
                        // Search
                        terminal.prepare_for_input(&command_indicator(&path))?;

                        let options = if path.is_empty() {
                            get_search_options(&config.keys)
                        } else {
                            get_search_options(&path)
                        };

                        let text_options = format_search_options(&options);
                        if let Some(selection) = terminal.select(text_options.as_slice())? {
                            let selected_node = &options[selection];

                            // Rebuild path based on the selected node ID
                            path.clear();
                            let mut lookup = &config.keys;
                            for part in selected_node.id.split("").filter(|part| !part.is_empty()) {
                                if let Some(node) = lookup.iter().find(|n| n.key == part) {
                                    path.push(node.clone());
                                    lookup = &node.keys;
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

                        // Display error for configured duration, or until user presses a key
                        let _ = event::poll(std::time::Duration::from_millis(
                            ERROR_DISPLAY_DURATION_MS,
                        ));
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
                    if path.is_empty() {
                        // Can't execute an empty command
                        terminal.start_of_row()?;
                        terminal.write(&format!("{}", "No command to execute".red()))?;
                        terminal.flush()?;

                        // Display error for configured duration, or until user presses a key
                        let _ = event::poll(std::time::Duration::from_millis(
                            ERROR_DISPLAY_DURATION_MS,
                        ));
                    } else {
                        let command = compose_command(&path);
                        terminal.teardown()?;
                        // Safe to unwrap because we checked is_empty above
                        let last_node = path.last().unwrap();
                        return if opts.print_immediate_tag && last_node.is_immediate {
                            Ok(format!("{} {}", IMMEDIATE_PREFIX, command))
                        } else {
                            Ok(command)
                        };
                    }
                }
                _ => {}
            }
        }
    }
}
