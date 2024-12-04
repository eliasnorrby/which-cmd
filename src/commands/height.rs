use crate::constants::*;

pub fn height_command() {
    let header_rows = 4;
    let footer_rows = 2;

    let tui_height = NUMBER_OF_ROWS + header_rows + footer_rows;
    println!("{}", tui_height)
}
