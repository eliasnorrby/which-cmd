use crate::config::Config;
use crate::constants::NUMBER_OF_ROWS;
use crate::{
    config_node::{ConfigNode, InputType},
    options::Options,
};

use crossterm::{
    cursor::{self},
    event::{self, Event, KeyCode},
    style::Stylize,
    terminal::{self, ClearType},
    ExecutableCommand, Result as CrosstermResult,
};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use std::io::Write;

const IMMEDIATE_PREFIX: &str = "__IMMEDIATE__";

struct Terminal<W: Write> {
    writer: W,
}

impl<W: Write> Terminal<W> {
    pub fn new(writer: W) -> Self {
        Terminal { writer }
    }

    pub fn setup(&mut self) -> CrosstermResult<()> {
        terminal::enable_raw_mode()?;
        self.writer.execute(cursor::Hide)?;
        self.clear_screen()?;
        Ok(())
    }

    pub fn teardown(&mut self) -> CrosstermResult<()> {
        self.clear_screen()?;
        self.writer.execute(cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn clear_screen(&mut self) -> CrosstermResult<()> {
        self.writer.execute(terminal::Clear(ClearType::All))?;
        self.writer.execute(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn write(&mut self, content: &str) -> CrosstermResult<()> {
        self.writer.write_all(b" ")?;
        self.writer.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn write_line(&mut self, content: &str) -> std::io::Result<()> {
        self.write(content)?;
        self.blank_line()?;
        Ok(())
    }

    pub fn blank_line(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    /// Writes a line of text centered horizontally on the current row.
    pub fn write_centered(&mut self, content: &str) -> std::io::Result<()> {
        // Fetch terminal size
        let (cols, _) = terminal::size()?;

        // Calculate starting column for center alignment
        let content_length = console::measure_text_width(content) as u16;
        let start_col = if content_length < cols {
            (cols - content_length) / 2
        } else {
            0
        };

        // Move cursor to the starting column of the current row
        self.writer
            .execute(cursor::MoveTo(start_col, cursor::position()?.1))?;

        // Write the content
        self.write(content)
    }

    pub fn start_of_row(&mut self) -> CrosstermResult<()> {
        self.writer
            .execute(cursor::MoveTo(0, cursor::position()?.1))?;
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

fn pop_to_first_non_is_fleeting(path: &mut Vec<ConfigNode>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

fn format_node(node: &ConfigNode, opts: &Options) -> String {
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
            } else if highlighted == "" {
                part.green()
            } else {
                part.yellow()
            }
        ));
    }
    highlighted
}

pub fn run_tui(config: Config, opts: Options) -> Result<String, Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut terminal = Terminal::new(std::io::stdout());

    terminal.setup()?;

    let mut path: Vec<ConfigNode> = Vec::new();
    let mut loop_node_index: Option<usize> = None;

    loop {
        terminal.clear_screen()?;

        // Display the current path
        if !path.is_empty() {
            terminal.write_line(&format!(
                "{} {}",
                "Command:".grey(),
                highlight_command(&compose_command(&path))
            ))?;
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
            &path[l].keys.to_vec()
        } else if let Some(last_node) = path.last() {
            &last_node.keys.to_vec()
        } else {
            &config.keys.to_vec()
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
                            terminal.clear_screen()?;
                            terminal.teardown()?;
                            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                                .with_prompt("What do you choose?")
                                .items(&node.choices)
                                .interact()
                                .unwrap();
                            terminal.setup()?;
                            path.push(node.with_selection(selection));
                        } else if let Some(input_type) = &node.input_type {
                            terminal.clear_screen()?;
                            terminal.teardown()?;
                            match input_type {
                                InputType::Text => {
                                    let input = dialoguer::Input::<String>::new()
                                        .with_prompt(&format!("Enter {}", node.name))
                                        .interact()?;
                                    terminal.setup()?;
                                    path.push(node.with_input(&input));
                                }
                                InputType::Number => {
                                    let input = dialoguer::Input::<i32>::new()
                                        .with_prompt(&format!("Enter {}", node.name))
                                        .interact()?;
                                    terminal.setup()?;
                                    path.push(node.with_input(&input.to_string()));
                                }
                            }
                            terminal.setup()?;
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

fn compose_command(path: &[ConfigNode]) -> String {
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
    use crate::config_node::ConfigNode;

    #[test]
    fn test_compose_command_no_anchor() {
        let node1 = ConfigNode {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = ConfigNode {
            id: "s".into(),
            key: "s".into(),
            name: "status".into(),
            value: "status".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
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
        let node1 = ConfigNode {
            id: "g".into(),
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node2 = ConfigNode {
            id: "h".into(),
            key: "h".into(),
            name: "GitHub".into(),
            value: "gh".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: true,
            is_loop: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let node3 = ConfigNode {
            id: "p".into(),
            key: "p".into(),
            name: "pull request".into(),
            value: "pr".into(),
            is_immediate: false,
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
            choices: vec![],
            input_type: None,
        };
        let path = vec![node1, node2, node3];
        let command = compose_command(&path);
        assert_eq!(command, "gh pr");
    }
}
