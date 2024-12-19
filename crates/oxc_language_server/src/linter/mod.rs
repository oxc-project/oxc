use oxc_data_structures::rope::{get_line_column, Rope};
use tower_lsp::lsp_types::Position;

pub mod error_with_position;
mod isolated_lint_handler;
pub mod server_linter;

#[allow(clippy::cast_possible_truncation)]
pub fn offset_to_position(offset: usize, source_text: &str) -> Position {
    // TODO(perf): share a single instance of `Rope`
    let rope = Rope::from_str(source_text);
    let (line, column) = get_line_column(&rope, offset as u32, source_text);
    Position::new(line, column)
}
