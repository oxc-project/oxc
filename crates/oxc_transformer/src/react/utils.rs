use ropey::Rope;

/// Get line and column from offset and source text
pub fn get_line_column(offset: u32, source_text: &str) -> (usize, usize) {
    let offset = offset as usize;
    let rope = Rope::from_str(source_text);
    let line = rope.byte_to_line(offset);
    let first_char_of_line = rope.line_to_char(line);
    // Original offset is byte, but Rope uses char offset
    let offset = rope.byte_to_char(offset);
    let column = offset - first_char_of_line;
    // line and column is zero-indexed, but we want 1-indexed
    (line + 1, column + 1)
}
