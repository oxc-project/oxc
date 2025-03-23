use oxc_data_structures::rope::{Rope, get_line_column};
use tower_lsp::lsp_types::Position;

pub mod error_with_position;
mod isolated_lint_handler;
pub mod server_linter;

#[expect(clippy::cast_possible_truncation)]
pub fn offset_to_position(offset: usize, source_text: &str) -> Position {
    // TODO(perf): share a single instance of `Rope`
    let rope = Rope::from_str(source_text);
    let (line, column) = get_line_column(&rope, offset as u32, source_text);
    Position::new(line, column)
}

#[cfg(test)]
mod test {
    use crate::linter::offset_to_position;

    #[test]
    fn single_line() {
        let source = "foo.bar!;";
        assert_position(source, 0, (0, 0));
        assert_position(source, 4, (0, 4));
        assert_position(source, 9, (0, 9));
    }

    #[test]
    fn multi_line() {
        let source = "console.log(\n  foo.bar!\n);";
        assert_position(source, 0, (0, 0));
        assert_position(source, 12, (0, 12));
        assert_position(source, 13, (1, 0));
        assert_position(source, 23, (1, 10));
        assert_position(source, 24, (2, 0));
        assert_position(source, 26, (2, 2));
    }

    #[test]
    fn multi_byte() {
        let source = "let foo = \n  '👍';";
        assert_position(source, 10, (0, 10));
        assert_position(source, 11, (1, 0));
        assert_position(source, 14, (1, 3));
        assert_position(source, 18, (1, 5));
        assert_position(source, 19, (1, 6));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn out_of_bounds() {
        offset_to_position(100, "foo");
    }

    fn assert_position(source: &str, offset: usize, expected: (u32, u32)) {
        let position = offset_to_position(offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
