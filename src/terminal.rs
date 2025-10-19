use crate::error::{Result, WhichCmdError};
use crate::node::InputType;

use crossterm::{
    cursor::{self},
    event,
    style::Stylize,
    terminal::{self, ClearType},
    ExecutableCommand,
};

use std::io::Write;

pub struct Terminal<W: Write> {
    writer: W,
    start_row: u16,
    tui_height: u16,
    border: bool,
    terminal_width: u16,
}

impl<W: Write> Terminal<W> {
    pub fn new(writer: W) -> Self {
        Terminal {
            writer,
            start_row: 0,
            tui_height: 0,
            border: false,
            terminal_width: 0,
        }
    }

    pub fn set_border(&mut self, enabled: bool) {
        self.border = enabled;
    }

    pub fn setup(&mut self) -> Result<()> {
        // Save the current cursor position
        let pos = cursor::position()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to get cursor position: {}", e)))?;
        self.start_row = pos.1;

        // Calculate TUI height using the centralized function
        // If border is enabled, add 2 lines for top and bottom borders
        self.tui_height = crate::constants::calculate_tui_height() as u16 + if self.border { 2 } else { 0 };

        // Ensure we have enough space below the cursor
        // If not, move down to create space
        let (cols, rows) = terminal::size()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to get terminal size: {}", e)))?;

        self.terminal_width = cols;

        if self.start_row + self.tui_height > rows {
            // We need to scroll down to make room
            let lines_needed = self.start_row + self.tui_height - rows;
            for _ in 0..lines_needed {
                self.writer.write_all(b"\r\n")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write newline: {}", e)))?;
            }
            self.start_row = rows.saturating_sub(self.tui_height);
        }

        terminal::enable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to enable raw mode: {}", e)))?;
        self.writer
            .execute(cursor::Hide)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to hide cursor: {}", e)))?;
        self.clear_screen()?;
        Ok(())
    }

    pub fn teardown(&mut self) -> Result<()> {
        // Clear the TUI area
        self.clear_screen()?;

        // Position cursor at the start row (where the TUI was)
        self.writer
            .execute(cursor::MoveTo(0, self.start_row))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

        self.writer
            .execute(cursor::Show)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to show cursor: {}", e)))?;
        terminal::disable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to disable raw mode: {}", e)))?;
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        // Move cursor to start position
        self.writer
            .execute(cursor::MoveTo(0, self.start_row))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

        // Clear from cursor to end of screen (will clear our TUI area)
        self.writer
            .execute(terminal::Clear(ClearType::FromCursorDown))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to clear screen: {}", e)))?;

        // Move back to start position
        self.writer
            .execute(cursor::MoveTo(0, self.start_row))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

        // If border is enabled, draw the top border
        if self.border {
            self.draw_top_border()?;
        }

        Ok(())
    }

    fn draw_top_border(&mut self) -> Result<()> {
        let border_line = format!(
            "{}",
            format!(
                "{}{}{}",
                "╭",
                "─".repeat((self.terminal_width - 2) as usize),
                "╮"
            )
            .dark_grey()
        );
        self.writer
            .write_all(border_line.as_bytes())
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write top border: {}", e)))?;
        Ok(())
    }

    pub fn draw_bottom_border(&mut self) -> Result<()> {
        if self.border {
            self.writer
                .write_all(b"\r\n")
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write newline: {}", e)))?;
            let border_line = format!(
                "{}",
                format!(
                    "{}{}{}",
                    "╰",
                    "─".repeat((self.terminal_width - 2) as usize),
                    "╯"
                )
                .dark_grey()
            );
            self.writer
                .write_all(border_line.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write bottom border: {}", e)))?;
        }
        Ok(())
    }

    pub fn write(&mut self, content: &str) -> Result<()> {
        if self.border {
            let left_border = format!("{} ", "│".dark_grey());
            self.writer
                .write_all(left_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        } else {
            self.writer
                .write_all(b" ")
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        }
        self.writer
            .write_all(content.as_bytes())
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        Ok(())
    }

    pub fn write_line(&mut self, content: &str) -> Result<()> {
        self.write(content)?;
        if self.border {
            // Add right border before newline
            // We need to get the current cursor position to know how much content was written
            let pos = cursor::position().map_err(|e| {
                WhichCmdError::Terminal(format!("Failed to get cursor position: {}", e))
            })?;

            // Current column position (0-based)
            let current_col = pos.0;

            // Calculate how many spaces we need to reach the right border
            // Terminal width - 2 (for " │")
            let target_col = self.terminal_width.saturating_sub(2);
            let padding = target_col.saturating_sub(current_col);

            for _ in 0..padding {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            let right_border = format!(" {}", "│".dark_grey());
            self.writer
                .write_all(right_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write right border: {}", e)))?;
        }
        self.blank_line()?;
        Ok(())
    }

    pub fn blank_line(&mut self) -> Result<()> {
        self.writer
            .write_all(b"\r\n")
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write blank line: {}", e)))?;
        Ok(())
    }

    pub fn empty_border_line(&mut self) -> Result<()> {
        if self.border {
            // Draw empty line with borders
            let left_border = format!("{}", "│".dark_grey());
            self.writer
                .write_all(left_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write empty border line: {}", e)))?;
            let inner_width = self.terminal_width.saturating_sub(2) as usize;
            for _ in 0..inner_width {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write empty border line: {}", e)))?;
            }
            let right_border = format!("{}", "│".dark_grey());
            self.writer
                .write_all(right_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write empty border line: {}", e)))?;
            self.blank_line()?;
        } else {
            self.blank_line()?;
        }
        Ok(())
    }

    /// Writes a line of text centered horizontally on the current row.
    pub fn write_centered(&mut self, content: &str) -> Result<()> {
        if self.border {
            // With border, we need to write the full line with left border, centered content, and right border
            let left_border = format!("{} ", "│".dark_grey());
            self.writer
                .write_all(left_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write left border: {}", e)))?;

            // Calculate available width for content (terminal width - borders)
            let available_width = self.terminal_width.saturating_sub(4) as usize; // 4 for "│ " and " │"
            let content_length = console::measure_text_width(content);

            // Calculate padding
            let total_padding = available_width.saturating_sub(content_length);
            let left_padding = total_padding / 2;
            let right_padding = total_padding - left_padding;

            // Write left padding
            for _ in 0..left_padding {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            // Write content
            self.writer
                .write_all(content.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write content: {}", e)))?;

            // Write right padding
            for _ in 0..right_padding {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            // Write right border
            let right_border = format!(" {}", "│".dark_grey());
            self.writer
                .write_all(right_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write right border: {}", e)))?;
        } else {
            // Without border, use the original implementation
            let (cols, _) = terminal::size()
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to get terminal size: {}", e)))?;

            // Calculate starting column for center alignment
            let content_length = console::measure_text_width(content) as u16;
            let start_col = if content_length < cols {
                (cols - content_length) / 2
            } else {
                0
            };

            // Move cursor to the starting column of the current row
            let pos = cursor::position().map_err(|e| {
                WhichCmdError::Terminal(format!("Failed to get cursor position: {}", e))
            })?;
            self.writer
                .execute(cursor::MoveTo(start_col, pos.1))
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

            // Write the content
            self.writer
                .write_all(content.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to flush: {}", e)))?;
        Ok(())
    }

    pub fn prepare_for_input(&mut self, content: &str) -> Result<()> {
        self.clear_screen()?;
        self.write_line(content)?;
        self.blank_line()?;
        Ok(())
    }

    pub fn input(&mut self, input_type: &InputType, name: &str) -> Result<String> {
        // Display prompt
        let prompt = format!("Enter {}: ", name);
        self.write(&prompt.cyan().to_string())?;
        self.flush()?;

        // Enable cursor and collect input
        self.writer
            .execute(cursor::Show)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to show cursor: {}", e)))?;

        let mut input_str = String::new();

        loop {
            if let event::Event::Key(event::KeyEvent { code, .. }) = event::read()
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to read event: {}", e)))?
            {
                match code {
                    event::KeyCode::Enter => break,
                    event::KeyCode::Esc => {
                        self.writer.execute(cursor::Hide).map_err(|e| {
                            WhichCmdError::Terminal(format!("Failed to hide cursor: {}", e))
                        })?;
                        return Err(WhichCmdError::Terminal("Input cancelled".to_string()));
                    }
                    event::KeyCode::Char(c) => {
                        // Validate input based on type
                        match input_type {
                            InputType::Number => {
                                // Only allow digits and minus sign (at start)
                                if c.is_ascii_digit() || (c == '-' && input_str.is_empty()) {
                                    input_str.push(c);
                                    self.writer.write_all(&[c as u8]).map_err(|e| {
                                        WhichCmdError::Terminal(format!("Failed to write: {}", e))
                                    })?;
                                    self.flush()?;
                                }
                            }
                            InputType::Text => {
                                input_str.push(c);
                                self.writer.write_all(&[c as u8]).map_err(|e| {
                                    WhichCmdError::Terminal(format!("Failed to write: {}", e))
                                })?;
                                self.flush()?;
                            }
                        }
                    }
                    event::KeyCode::Backspace => {
                        if !input_str.is_empty() {
                            input_str.pop();
                            // Move cursor back, write space, move cursor back again
                            self.writer.execute(cursor::MoveLeft(1)).map_err(|e| {
                                WhichCmdError::Terminal(format!("Failed to move cursor: {}", e))
                            })?;
                            self.writer.write_all(b" ").map_err(|e| {
                                WhichCmdError::Terminal(format!("Failed to write: {}", e))
                            })?;
                            self.writer.execute(cursor::MoveLeft(1)).map_err(|e| {
                                WhichCmdError::Terminal(format!("Failed to move cursor: {}", e))
                            })?;
                            self.flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Hide cursor again
        self.writer
            .execute(cursor::Hide)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to hide cursor: {}", e)))?;

        // Validate number input
        if let InputType::Number = input_type {
            input_str
                .parse::<i32>()
                .map_err(|_| WhichCmdError::Terminal("Invalid number".to_string()))?;
        }

        Ok(input_str)
    }

    /// Replaces the last line with an error message on the left and centered help text.
    /// This is used to display error messages alongside the close/back labels.
    /// The help text stays in the same centered position regardless of the error message.
    pub fn replace_last_line(&mut self, error_msg: &str, help_text: &str) -> Result<()> {
        // Calculate the row position of the last line
        // If border is enabled: start_row + tui_height - 2 (one line before bottom border)
        // If no border: start_row + tui_height - 1
        let last_line_row = if self.border {
            self.start_row + self.tui_height - 2
        } else {
            self.start_row + self.tui_height - 1
        };

        // Move cursor to the start of the last line
        self.writer
            .execute(cursor::MoveTo(0, last_line_row))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

        // Clear the current line
        self.writer
            .execute(terminal::Clear(ClearType::CurrentLine))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to clear line: {}", e)))?;

        if self.border {
            // With border: left border + error + padding + centered help text + padding + right border
            let left_border = format!("{} ", "│".dark_grey());
            self.writer
                .write_all(left_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write left border: {}", e)))?;

            // Write error message
            self.writer
                .write_all(error_msg.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write error: {}", e)))?;

            // Calculate available width for content (terminal width - borders)
            let available_width = self.terminal_width.saturating_sub(4) as usize; // 4 for "│ " and " │"
            let error_length = console::measure_text_width(error_msg);
            let help_length = console::measure_text_width(help_text);

            // Calculate where help text should be centered
            let help_start_col = (available_width.saturating_sub(help_length)) / 2;

            // Calculate padding before help text (accounting for error message)
            let padding_before_help = help_start_col.saturating_sub(error_length);

            // Write padding before help text
            for _ in 0..padding_before_help {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            // Write help text
            self.writer
                .write_all(help_text.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write help text: {}", e)))?;

            // Calculate padding after help text
            let used_width = error_length + padding_before_help + help_length;
            let padding_after_help = available_width.saturating_sub(used_width);

            // Write padding after help text
            for _ in 0..padding_after_help {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            // Write right border
            let right_border = format!(" {}", "│".dark_grey());
            self.writer
                .write_all(right_border.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write right border: {}", e)))?;
        } else {
            // Without border: error + padding + centered help text + padding
            self.writer
                .write_all(b" ")
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write space: {}", e)))?;

            // Write error message
            self.writer
                .write_all(error_msg.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write error: {}", e)))?;

            let available_width = self.terminal_width as usize;
            let error_length = console::measure_text_width(error_msg) + 1; // +1 for the leading space
            let help_length = console::measure_text_width(help_text);

            // Calculate where help text should be centered
            let help_start_col = (available_width.saturating_sub(help_length)) / 2;

            // Calculate padding before help text (accounting for error message)
            let padding_before_help = help_start_col.saturating_sub(error_length);

            // Write padding before help text
            for _ in 0..padding_before_help {
                self.writer
                    .write_all(b" ")
                    .map_err(|e| WhichCmdError::Terminal(format!("Failed to write padding: {}", e)))?;
            }

            // Write help text
            self.writer
                .write_all(help_text.as_bytes())
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to write help text: {}", e)))?;
        }

        Ok(())
    }
}
