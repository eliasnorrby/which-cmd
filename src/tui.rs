use crate::command_node::CommandNode;
use crate::config::Config;

use crossterm::{
    cursor::{self},
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    ExecutableCommand,
};

use std::io::Write;

struct Output {
    stdout: std::io::Stdout,
}

impl Output {
    fn new() -> Self {
        Output {
            stdout: std::io::stdout(),
        }
    }

    fn write_line(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.stdout.write_fmt(args)?;
        self.blank_line()?;
        Ok(())
    }

    fn blank_line(&mut self) -> std::io::Result<()> {
        self.stdout.write_all(b"\r\n")?;
        Ok(())
    }
}

macro_rules! output_write_line {
    ($output:expr, $($arg:tt)*) => {
        $output.write_line(format_args!($($arg)*))
    }
}

pub fn run_tui(config: Config) -> Result<String, Box<dyn std::error::Error>> {
    // Initialize terminal
    terminal::enable_raw_mode()?;
    let mut output = Output::new();

    let mut current_nodes = &config.keys;
    let mut path: Vec<&CommandNode> = Vec::new();

    loop {
        // Clear the screen
        output.stdout.execute(terminal::Clear(ClearType::All))?;
        output.stdout.execute(cursor::MoveTo(0, 0))?;

        output_write_line!(
            output,
            "Press a key to select an option, Backspace to go back, or 'q' to quit."
        )?;
        output.blank_line()?;

        // Display the current path
        if !path.is_empty() {
            let keys_pressed: Vec<&str> = path.iter().map(|node| node.key.as_str()).collect();
            output_write_line!(output, "Keys pressed: {}", keys_pressed.join(" > "))?;
            output.blank_line()?;
        } else {
            output_write_line!(output, "Available comands:")?;
            output.blank_line()?;
        }

        // Display the options
        for node in current_nodes {
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

        output.stdout.flush()?;

        // Wait for an event
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('q') => {
                    // Exit without outputting a command
                    terminal::disable_raw_mode()?;
                    return Err("User quit the application".into());
                }
                KeyCode::Char(c) => {
                    // Handle character input
                    if let Some(node) = current_nodes.iter().find(|n| n.key == c.to_string()) {
                        path.push(node);
                        if node.is_leaf() {
                            // Build and return the command
                            let command = compose_command(&path);
                            terminal::disable_raw_mode()?;
                            output.stdout.execute(terminal::Clear(ClearType::All))?;
                            output.stdout.execute(cursor::MoveTo(0, 0))?;
                            return Ok(command);
                        } else {
                            current_nodes = &node.keys;
                        }
                    } else {
                        // Invalid key pressed
                        output_write_line!(output, "\nInvalid key: {}", c)?;
                        output.stdout.flush()?;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                KeyCode::Backspace => {
                    // Handle backspace
                    if let Some(_) = path.pop() {
                        if let Some(last_node) = path.last() {
                            current_nodes = &last_node.keys;
                        } else {
                            current_nodes = &config.keys;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn compose_command(path: &[&CommandNode]) -> String {
    // Start building the command from the last reset point
    let mut command_parts = Vec::new();
    let mut start_index = 0;
    for (i, node) in path.iter().enumerate() {
        if node.reset {
            start_index = i;
        }
    }
    for node in &path[start_index..] {
        command_parts.push(node.value.as_str());
    }
    command_parts.join(" ")
}
