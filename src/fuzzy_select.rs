use crate::error::Result;
use crate::terminal::Terminal;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    style::Stylize,
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::Write;

/// A fuzzy select interface that integrates with our bordered TUI
pub struct FuzzySelect<'a> {
    items: &'a [String],
    prompt: String,
}

struct MatchedItem {
    index: usize,
    score: i64,
    text: String,
}

impl<'a> FuzzySelect<'a> {
    pub fn new(items: &'a [String]) -> Self {
        FuzzySelect {
            items,
            prompt: " :".to_string(),
        }
    }

    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Run the fuzzy select interface and return the selected index, or None if cancelled
    pub fn interact<W: Write>(&mut self, terminal: &mut Terminal<W>) -> Result<Option<usize>> {
        let mut query = String::new();
        let mut cursor_pos = 0usize;
        let mut selected_index = 0usize;

        // Show cursor at the start
        terminal.show_cursor()?;

        loop {
            // Filter and sort items based on current query
            let matched_items = self.filter_items(&query);

            // Ensure selected_index is within bounds
            if selected_index >= matched_items.len() && !matched_items.is_empty() {
                selected_index = matched_items.len() - 1;
            }

            // Render the interface
            self.render(terminal, &query, &matched_items, selected_index)?;

            // Wait for input
            if let Event::Key(KeyEvent { code, .. }) = event::read().map_err(|e| {
                crate::error::WhichCmdError::Terminal(format!("Failed to read event: {}", e))
            })? {
                match code {
                    KeyCode::Esc => {
                        terminal.hide_cursor()?;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        terminal.hide_cursor()?;
                        if matched_items.is_empty() {
                            return Ok(None);
                        }
                        return Ok(Some(matched_items[selected_index].index));
                    }
                    KeyCode::Char(c) => {
                        query.insert(cursor_pos, c);
                        cursor_pos += 1;
                        selected_index = 0; // Reset selection when query changes
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            query.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                            selected_index = 0;
                        }
                    }
                    KeyCode::Up => {
                        selected_index = selected_index.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if !matched_items.is_empty() && selected_index < matched_items.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Filter items based on query using fuzzy matching
    fn filter_items(&self, query: &str) -> Vec<MatchedItem> {
        if query.is_empty() {
            // No query, return all items in original order
            return self
                .items
                .iter()
                .enumerate()
                .map(|(index, text)| MatchedItem {
                    index,
                    score: 0,
                    text: text.clone(),
                })
                .collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut matched: Vec<MatchedItem> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, text)| {
                matcher.fuzzy_match(text, query).map(|score| MatchedItem {
                    index,
                    score,
                    text: text.clone(),
                })
            })
            .collect();

        // Sort by score (highest first)
        matched.sort_by(|a, b| b.score.cmp(&a.score));

        matched
    }

    /// Render the fuzzy select interface
    /// Layout dynamically sized based on terminal rows setting:
    /// 1. Top border (if enabled)
    /// 2. Prompt line (Search: query)
    /// 3. Empty padding
    ///    4-N. Items (rows from terminal setting)
    ///    N+1. Empty padding
    ///    N+2. Footer (esc to cancel)
    ///    N+3. Bottom border (if enabled)
    fn render<W: Write>(
        &self,
        terminal: &mut Terminal<W>,
        query: &str,
        matched_items: &[MatchedItem],
        selected_index: usize,
    ) -> Result<()> {
        // Clear screen and draw top border
        terminal.clear_screen()?;

        // Line 2: Prompt and query
        terminal.write_line(&format!("{} {}", self.prompt.clone().yellow(), query))?;

        // Line 3: Empty padding
        terminal.empty_border_line()?;

        // Lines 4-N: Items
        // Fuzzy select layout:
        // - 1 line: prompt + query
        // - 1 line: empty padding before items
        // - N lines: items
        // - 1 line: empty padding after items
        // - 1 line: footer
        let content_rows = terminal.get_content_rows();
        let prompt_and_header_lines = 2; // prompt + padding
        let footer_lines = 2; // padding + help text
        let num_items = content_rows.saturating_sub(prompt_and_header_lines + footer_lines);

        for i in 0..num_items {
            if i < matched_items.len() {
                let item = &matched_items[i];
                let display = if i == selected_index {
                    format!("{} {}", ">".yellow(), item.text.clone())
                } else {
                    format!("  {}", item.text)
                };
                terminal.write_line(&display)?;
            } else {
                // Render empty line to maintain consistent height
                terminal.empty_border_line()?;
            }
        }

        // Empty padding
        terminal.empty_border_line()?;

        // Footer
        terminal.write_centered(&format!("󱊷  {}", "cancel".dark_grey()))?;

        // Bottom border
        terminal.draw_bottom_border()?;

        terminal.flush()?;

        // Position cursor after the query text
        // Row: start_row + 1 (accounting for top border if present)
        // Col: border (2 chars "│ ") + prompt length + space + query length
        let row = terminal.get_start_row() + if terminal.has_border() { 1 } else { 0 }; // Line 2 (0-indexed, so +1 from start)
        let prompt_len = console::measure_text_width(&self.prompt);
        let query_len = console::measure_text_width(query);
        let col = if terminal.has_border() { 2 } else { 1 }
            + prompt_len as u16
            + 1 // for the space after prompt
            + query_len as u16;
        terminal.move_cursor_to(col, row)?;

        Ok(())
    }
}
