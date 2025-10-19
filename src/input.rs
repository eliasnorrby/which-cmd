use crate::error::{Result, WhichCmdError};
use crate::node::InputType;
use crate::terminal::Terminal;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    style::Stylize,
};
use std::io::Write;

/// An input component that integrates with our bordered TUI
pub struct Input<'a> {
    input_type: &'a InputType,
    prompt: String,
}

impl<'a> Input<'a> {
    pub fn new(input_type: &'a InputType, name: &str) -> Self {
        Input {
            input_type,
            prompt: format!("Enter {}: ", name),
        }
    }

    /// Run the input interface and return the entered value, or None if cancelled
    pub fn interact<W: Write>(&self, terminal: &mut Terminal<W>) -> Result<Option<String>> {
        let mut input_str = String::new();

        // Render initial state
        self.render(terminal, &input_str)?;

        // Enable cursor
        terminal.show_cursor()?;

        loop {
            // Wait for input
            if let Event::Key(KeyEvent { code, .. }) = event::read()
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to read event: {}", e)))?
            {
                match code {
                    KeyCode::Enter => {
                        terminal.hide_cursor()?;
                        // Validate number input
                        if let InputType::Number = self.input_type {
                            if input_str.is_empty() {
                                return Ok(None);
                            }
                            input_str.parse::<i32>().map_err(|_| {
                                WhichCmdError::Terminal("Invalid number".to_string())
                            })?;
                        }
                        return Ok(Some(input_str));
                    }
                    KeyCode::Esc => {
                        terminal.hide_cursor()?;
                        return Ok(None);
                    }
                    KeyCode::Char(c) => {
                        // Validate input based on type
                        match self.input_type {
                            InputType::Number => {
                                // Only allow digits and minus sign (at start)
                                if c.is_ascii_digit() || (c == '-' && input_str.is_empty()) {
                                    input_str.push(c);
                                    self.render(terminal, &input_str)?;
                                }
                            }
                            InputType::Text => {
                                input_str.push(c);
                                self.render(terminal, &input_str)?;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if !input_str.is_empty() {
                            input_str.pop();
                            self.render(terminal, &input_str)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Render the input interface
    fn render<W: Write>(&self, terminal: &mut Terminal<W>, input: &str) -> Result<()> {
        terminal.clear_screen()?;

        // Display prompt and input
        terminal.write_line(&format!("{}{}", self.prompt.clone().cyan(), input))?;

        // Fill remaining space
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;
        terminal.empty_border_line()?;

        // Footer
        terminal.empty_border_line()?;
        terminal.write_centered(&format!("󱊷  {}", "cancel".dark_grey()))?;

        terminal.draw_bottom_border()?;
        terminal.flush()?;

        Ok(())
    }
}
