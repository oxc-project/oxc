use crate::{formatter::format_element::FormatElement, options};

use super::{import_unit::SortableImport, source_line::SourceLine};

#[derive(Debug, Clone)]
pub enum PartitionedChunk {
    /// A chunk containing import statements,
    /// and possibly leading/trailing comments or empty lines.
    Imports(Vec<SourceLine>),
    /// A boundary chunk.
    /// Always contains `SourceLine::Others`,
    /// or optionally `SourceLine::Empty|CommentOnly` depending on partition options.
    Boundary(SourceLine),
}

impl Default for PartitionedChunk {
    fn default() -> Self {
        Self::Imports(vec![])
    }
}

impl PartitionedChunk {
    pub fn add_imports_line(&mut self, line: SourceLine) {
        debug_assert!(
            !matches!(line, SourceLine::Others(..)),
            "`line` must not be of type `SourceLine::Others`."
        );

        match self {
            Self::Imports(lines) => lines.push(line),
            Self::Boundary(_) => {
                unreachable!("Cannot add to a boundary chunk");
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Imports(lines) if lines.is_empty())
    }

    /// Convert this import chunk into a list of sortable import units and trailing lines.
    /// Returns a tuple of `(sortable_imports, trailing_lines)`.
    #[must_use]
    pub fn into_import_units<'a>(
        self,
        elements: &'a [FormatElement],
        options: &options::SortImports,
    ) -> (Vec<SortableImport<'a>>, Vec<SourceLine>) {
        let Self::Imports(lines) = self else {
            unreachable!(
                "`into_import_units()` must be called on `PartitionedChunk::Imports` only."
            );
        };

        let mut sortable_imports = vec![];
        let mut current_leading_lines = vec![];
        for line in lines {
            match line {
                SourceLine::Import(..) => {
                    sortable_imports.push(
                        SortableImport::new(std::mem::take(&mut current_leading_lines), line)
                            .collect_sort_keys(elements, options),
                    );
                }
                SourceLine::CommentOnly(..) | SourceLine::Empty => {
                    current_leading_lines.push(line);
                }
                SourceLine::Others(..) => {
                    unreachable!(
                        "`PartitionedChunk::Imports` must not contain `SourceLine::Others`."
                    );
                }
            }
        }

        // Any remaining comments/lines are trailing
        let trailing_lines = current_leading_lines;

        (sortable_imports, trailing_lines)
    }
}
