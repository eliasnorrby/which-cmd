use crate::config_node::InputType;

use crossterm::{
    cursor::{self},
    terminal::{self, ClearType},
    ExecutableCommand, Result as CrosstermResult,
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

    pub fn prepare_for_input(&mut self, content: &str) -> CrosstermResult<()> {
        self.clear_screen()?;
        self.write_line(content)?;
        self.blank_line()?;
        Ok(())
    }

    pub fn select(&mut self, options: &[String]) -> CrosstermResult<Option<usize>> {
        // Disable raw mode because it breaks alignment of the options
        terminal::disable_raw_mode()?;
        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option:")
            .items(options)
            .interact_opt()
            .unwrap();
        terminal::enable_raw_mode()?;
        // FuzzySelect will show the cursor, so hide it again
        self.writer.execute(cursor::Hide)?;
        Ok(selection)
    }

    pub fn input(&mut self, input_type: &InputType, name: &str) -> CrosstermResult<String> {
        let input = match input_type {
            InputType::Text => dialoguer::Input::<String>::new()
                .with_prompt(format!(" Enter {}", name))
                .interact()
                .unwrap(),
            InputType::Number => dialoguer::Input::<i32>::new()
                .with_prompt(format!(" Enter {}", name))
                .interact()
                .unwrap()
                .to_string(),
        };
        Ok(input)
    }
}
