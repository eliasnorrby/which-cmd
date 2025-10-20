pub const PREFIX: &str = env!("CARGO_PKG_NAME");
pub const CONFIG_FILE_NAME: &str = "commands.yml";
pub const OUTPUT_FILE_NAME: &str = "out";
pub const CHOICE_KEY: &str = "[choice]";
pub const INPUT_KEY: &str = "[input]";
pub const IMMEDIATE_PREFIX: &str = "__IMMEDIATE__";

/// Default height of the TUI content area (excluding borders)
pub const DEFAULT_HEIGHT: usize = 10;

/// Duration to display error messages in the TUI (milliseconds)
pub const ERROR_DISPLAY_DURATION_MS: u64 = 750;

/// Help text displayed in the TUI footer
pub fn help_text() -> String {
    use crossterm::style::Stylize;
    format!("󱊷  {}  󰁮  {}", "close".dark_grey(), "back".dark_grey())
}
