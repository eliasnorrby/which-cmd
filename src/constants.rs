pub const PREFIX: &str = env!("CARGO_PKG_NAME");
pub const CONFIG_FILE_NAME: &str = "commands.yml";
pub const OUTPUT_FILE_NAME: &str = "out";
pub const CHOICE_KEY: &str = "[choice]";
pub const INPUT_KEY: &str = "[input]";
pub const NUMBER_OF_ROWS: usize = 4;
pub const IMMEDIATE_PREFIX: &str = "__IMMEDIATE__";

/// Duration to display error messages in the TUI (milliseconds)
pub const ERROR_DISPLAY_DURATION_MS: u64 = 750;

/// Calculate the total height of the TUI in rows
///
/// This includes:
/// - Header rows (4): command display, keys pressed, blank lines
/// - Content rows (NUMBER_OF_ROWS): the key bindings table
/// - Footer rows (2): help text and blank line
pub fn calculate_tui_height() -> usize {
    let header_rows = 4;
    let footer_rows = 2;
    NUMBER_OF_ROWS + header_rows + footer_rows
}
