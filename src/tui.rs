use crate::command_node::CommandNode;
use crate::config::Config;

use crossterm::{
    cursor::{self},
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    ExecutableCommand,
};

use std::io::{Stderr, Write};

struct Output {
    stderr: Stderr,
}

impl Output {
    fn new() -> Self {
        Output {
            stderr: std::io::stderr(),
        }
    }

    fn write_line(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.stderr.write_all(b" ")?;
        self.stderr.write_fmt(args)?;
        self.blank_line()?;
        Ok(())
    }

    fn blank_line(&mut self) -> std::io::Result<()> {
        self.stderr.write_all(b"\r\n")?;
        Ok(())
    }
}

macro_rules! output_write_line {
    ($output:expr, $($arg:tt)*) => {
        $output.write_line(format_args!($($arg)*))
    }
}

// TODO: Error handling
fn clear_screen(output: &mut Output) -> std::io::Result<()> {
    output.stderr.execute(terminal::Clear(ClearType::All))?;
    output.stderr.execute(cursor::MoveTo(0, 0))?;
    Ok(())
}

fn setup(output: &mut Output) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    output.stderr.execute(cursor::Hide)?;
    clear_screen(output)?;
    Ok(())
}

fn teardown(output: &mut Output) -> std::io::Result<()> {
    clear_screen(output)?;
    output.stderr.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn pop_to_first_non_is_fleeting(path: &mut Vec<&CommandNode>) {
    while let Some(node) = path.pop() {
        if !node.is_fleeting {
            path.push(node);
            break;
        }
    }
}

pub fn run_tui(config: Config) -> Result<String, Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut output = Output::new();

    setup(&mut output)?;

    let mut current_nodes = &config.keys;
    let mut path: Vec<&CommandNode> = Vec::new();
    let mut loop_node: Option<&CommandNode> = None;

    loop {
        clear_screen(&mut output)?;

        // Display the current path
        if !path.is_empty() {
            output_write_line!(output, "Command: {}", compose_command(&path))?;
            output.blank_line()?;
            let keys_pressed: Vec<&str> = path.iter().map(|node| node.key.as_str()).collect();
            output_write_line!(output, "Keys pressed: {}", keys_pressed.join(" > "))?;
            output.blank_line()?;
        } else {
            output_write_line!(
                output,
                "Press a key to select an option, 'backspace' to go back, or 'q' to quit."
            )?;
            output.blank_line()?;
            output_write_line!(output, "Available comands:")?;
            output.blank_line()?;
        }

        // Sort the nodes by key
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

        // Display the sorted options
        for node in &sorted_nodes {
            let sub_keys_count = node.keys.len();
            if sub_keys_count > 0 {
                output_write_line!(
                    output,
                    "{:<3} {:<15} +{}",
                    node.key,
                    node.name,
                    sub_keys_count
                )?;
            } else {
                output_write_line!(output, "{:<3} {:<15}", node.key, node.name)?;
            }
        }

        output.stderr.flush()?;

        // Wait for an event
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    teardown(&mut output)?;
                    return Ok("".into());
                }
                KeyCode::Char(c) => {
                    // Handle character input
                    if let Some(node) = current_nodes.iter().find(|n| n.key == c.to_string()) {
                        path.push(node);
                        if node.is_loop {
                            loop_node = Some(node);
                        }
                        if node.is_leaf() {
                            if let Some(l) = loop_node {
                                current_nodes = &l.keys;
                            } else {
                                // Build and return the command
                                let command = compose_command(&path);
                                teardown(&mut output)?;
                                return Ok(command);
                            }
                        } else {
                            current_nodes = &node.keys;
                        }
                    } else {
                        // Invalid key pressed
                        output_write_line!(output, "\nInvalid key: {}", c)?;
                        output.stderr.flush()?;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                KeyCode::Backspace => {
                    // Handle backspace
                    if let Some(_) = path.pop() {
                        pop_to_first_non_is_fleeting(&mut path);
                        if let Some(last_node) = path.last() {
                            current_nodes = &last_node.keys;
                        } else {
                            current_nodes = &config.keys;
                        }
                    }
                }
                KeyCode::Enter => {
                    let command = compose_command(&path);
                    teardown(&mut output)?;
                    return Ok(command);
                }
                _ => {}
            }
        }
    }
}

fn compose_command(path: &[&CommandNode]) -> String {
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
    use crate::command_node::CommandNode;

    #[test]
    fn test_compose_command_no_anchor() {
        let node1 = CommandNode {
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
        };
        let node2 = CommandNode {
            key: "s".into(),
            name: "status".into(),
            value: "status".into(),
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
        };
        let path = vec![&node1, &node2];
        let command = compose_command(&path);
        assert_eq!(command, "git status");
    }

    #[test]
    fn test_compose_command_with_anchor() {
        let node1 = CommandNode {
            key: "g".into(),
            name: "git".into(),
            value: "git".into(),
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
        };
        let node2 = CommandNode {
            key: "h".into(),
            name: "GitHub".into(),
            value: "gh".into(),
            is_fleeting: false,
            is_anchor: true,
            is_loop: false,
            keys: vec![],
        };
        let node3 = CommandNode {
            key: "p".into(),
            name: "pull request".into(),
            value: "pr".into(),
            is_fleeting: false,
            is_anchor: false,
            is_loop: false,
            keys: vec![],
        };
        let path = vec![&node1, &node2, &node3];
        let command = compose_command(&path);
        assert_eq!(command, "gh pr");
    }
}
