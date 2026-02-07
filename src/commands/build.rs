use std::fs;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::error::Result;
use crate::options::Options;
use crate::terminal;
use crate::tui;

use crate::constants::*;

pub fn build_command(
    config_path: Option<&Path>,
    immediate: bool,
    border: bool,
    height: usize,
    exec: bool,
) -> Result<()> {
    terminal::enable_raw_mode()?;

    let result = build_command_inner(config_path, immediate, border, height, exec);

    if result.is_err() {
        let _ = crossterm::terminal::disable_raw_mode();
    }

    result
}

fn build_command_inner(
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
    }
    .with_local_config()?;
    let command = tui::run_tui(config, opts)?;

    if exec {
        if !command.is_empty() {
            // SAFETY: setsid() is async-signal-safe and appropriate in pre_exec.
            // It creates a new session, detaching from the parent's process group
            // so the child survives after the parent exits.
            unsafe {
                Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .pre_exec(|| {
                        libc::setsid();
                        Ok(())
                    })
                    .spawn()?;
            }
        }
    } else {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
        let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
        fs::write(output_path, command)?;
    }

    Ok(())
}
