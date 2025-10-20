use std::fs;

use crate::config::Config;
use crate::error::Result;
use crate::options::Options;
use crate::tui;

use crate::constants::*;

pub fn build_command(immediate: bool, border: bool, height: usize) -> Result<()> {
    let opts = Options {
        print_immediate_tag: immediate,
        border,
        height,
    };

    let config = Config::from_file()?;
    let command = tui::run_tui(config, opts)?;

    let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
    let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
    fs::write(output_path, command)?;

    Ok(())
}
