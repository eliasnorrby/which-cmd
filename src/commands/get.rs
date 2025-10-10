use std::fs;

use crate::constants::*;
use crate::error::Result;

pub fn get_command() -> Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
    let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
    let contents = fs::read_to_string(&output_path)?;
    fs::remove_file(&output_path)?;
    println!("{}", contents);
    Ok(())
}
