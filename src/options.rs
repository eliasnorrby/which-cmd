use crate::constants::DEFAULT_HEIGHT;

#[derive(Debug)]
pub struct Options {
    pub print_immediate_tag: bool,
    pub border: bool,
    pub height: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            print_immediate_tag: false,
            border: false,
            height: DEFAULT_HEIGHT,
        }
    }
}
