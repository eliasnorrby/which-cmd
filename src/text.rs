/// Delete the word before the cursor, vim-style (Ctrl+W).
/// Skips trailing whitespace, then deletes back to the previous word boundary.
pub fn delete_word_back(s: &mut String, cursor_pos: &mut usize) {
    if *cursor_pos == 0 {
        return;
    }

    let bytes = s.as_bytes();
    let mut pos = *cursor_pos;

    // Skip whitespace before cursor
    while pos > 0 && bytes[pos - 1] == b' ' {
        pos -= 1;
    }

    // Delete back to the previous whitespace
    while pos > 0 && bytes[pos - 1] != b' ' {
        pos -= 1;
    }

    s.drain(pos..*cursor_pos);
    *cursor_pos = pos;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_single_word() {
        let mut s = "hello".to_string();
        let mut pos = 5;
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "");
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_delete_last_word() {
        let mut s = "hello world".to_string();
        let mut pos = 11;
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "hello ");
        assert_eq!(pos, 6);
    }

    #[test]
    fn test_delete_with_trailing_spaces() {
        let mut s = "hello   ".to_string();
        let mut pos = 8;
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "");
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_delete_at_beginning() {
        let mut s = "hello".to_string();
        let mut pos = 0;
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "hello");
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_delete_middle_word() {
        let mut s = "one two three".to_string();
        let mut pos = 7; // cursor after "two"
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "one  three");
        assert_eq!(pos, 4);
    }

    #[test]
    fn test_empty_string() {
        let mut s = String::new();
        let mut pos = 0;
        delete_word_back(&mut s, &mut pos);
        assert_eq!(s, "");
        assert_eq!(pos, 0);
    }
}
