pub const PREFIX: &str = env!("CARGO_PKG_NAME");
pub const CONFIG_FILE_NAME: &str = "commands.yml";
pub const OUTPUT_FILE_NAME: &str = "out";
pub const CHOICE_KEY: &str = "[choice]";
pub const INPUT_KEY: &str = "[input]";
pub const NUMBER_OF_ROWS: usize = 4;

/// Duration to display error messages in the TUI (milliseconds)
pub const ERROR_DISPLAY_DURATION_MS: u64 = 750;
