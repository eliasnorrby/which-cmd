use crate::error::{Result, WhichCmdError};
use crate::node::InputType;

use crossterm::{
    cursor::{self},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use std::io::Write;

pub struct Terminal<W: Write> {
    writer: W,
}

impl<W: Write> Terminal<W> {
    pub fn new(writer: W) -> Self {
        Terminal { writer }
    }

    pub fn setup(&mut self) -> Result<()> {
        terminal::enable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to enable raw mode: {}", e)))?;
        self.writer
            .execute(cursor::Hide)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to hide cursor: {}", e)))?;
        self.clear_screen()?;
        Ok(())
    }

    pub fn teardown(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.writer
            .execute(cursor::Show)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to show cursor: {}", e)))?;
        terminal::disable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to disable raw mode: {}", e)))?;
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        self.writer
            .execute(terminal::Clear(ClearType::All))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to clear screen: {}", e)))?;
        self.writer
            .execute(cursor::MoveTo(0, 0))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;
        Ok(())
    }

    pub fn write(&mut self, content: &str) -> Result<()> {
        self.writer
            .write_all(b" ")
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        self.writer
            .write_all(content.as_bytes())
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write: {}", e)))?;
        Ok(())
    }

    pub fn write_line(&mut self, content: &str) -> Result<()> {
        self.write(content)?;
        self.blank_line()?;
        Ok(())
    }

    pub fn blank_line(&mut self) -> Result<()> {
        self.writer
            .write_all(b"\r\n")
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to write blank line: {}", e)))?;
        Ok(())
    }

    /// Writes a line of text centered horizontally on the current row.
    pub fn write_centered(&mut self, content: &str) -> Result<()> {
        // Fetch terminal size
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
        let pos = cursor::position()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to get cursor position: {}", e)))?;
        self.writer
            .execute(cursor::MoveTo(start_col, pos.1))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;

        // Write the content
        self.write(content)
    }

    pub fn start_of_row(&mut self) -> Result<()> {
        let pos = cursor::position()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to get cursor position: {}", e)))?;
        self.writer
            .execute(cursor::MoveTo(0, pos.1))
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to move cursor: {}", e)))?;
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

    pub fn select(&mut self, options: &[String]) -> Result<Option<usize>> {
        // Disable raw mode because it breaks alignment of the options
        terminal::disable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to disable raw mode: {}", e)))?;

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option:")
            .items(options)
            .interact_opt()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to get selection: {}", e)))?;

        terminal::enable_raw_mode()
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to enable raw mode: {}", e)))?;

        // FuzzySelect will show the cursor, so hide it again
        self.writer
            .execute(cursor::Hide)
            .map_err(|e| WhichCmdError::Terminal(format!("Failed to hide cursor: {}", e)))?;

        Ok(selection)
    }

    pub fn input(&mut self, input_type: &InputType, name: &str) -> Result<String> {
        let input = match input_type {
            InputType::Text => dialoguer::Input::<String>::new()
                .with_prompt(format!(" Enter {}", name))
                .interact()
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to get text input: {}", e)))?,
            InputType::Number => dialoguer::Input::<i32>::new()
                .with_prompt(format!(" Enter {}", name))
                .interact()
                .map_err(|e| WhichCmdError::Terminal(format!("Failed to get number input: {}", e)))?
                .to_string(),
        };
        Ok(input)
    }
}
