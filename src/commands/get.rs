use std::fs;

use crate::constants::*;

pub fn get_command() -> Result<(), Box<dyn std::error::Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
    let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
    let contents = fs::read_to_string(output_path)?;
    println!("{}", contents);
    Ok(())
}
