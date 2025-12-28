use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::error::Result;
use crate::options::Options;
use crate::tui;

use crate::constants::*;

pub fn build_command(
    config_path: Option<&Path>,
    immediate: bool,
    border: bool,
    height: usize,
    exec: bool,
) -> Result<()> {
    let opts = Options {
        print_immediate_tag: immediate,
        border,
        height,
    };

    let config = match config_path {
        Some(path) => Config::from_path(path)?,
        None => Config::from_file()?,
    };
    let command = tui::run_tui(config, opts)?;

    if exec {
        if !command.is_empty() {
            let status = Command::new("sh").arg("-c").arg(&command).status()?;

            std::process::exit(status.code().unwrap_or(1));
        }
    } else {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
        let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
        fs::write(output_path, command)?;
    }

    Ok(())
}
