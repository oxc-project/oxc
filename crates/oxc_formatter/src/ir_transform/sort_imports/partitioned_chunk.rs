use crate::formatter::format_element::FormatElement;

use super::{
    import_unit::{ImportUnits, SortableImport},
    source_line::SourceLine,
};

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

    #[must_use]
    pub fn into_import_units(self, _elements: &[FormatElement]) -> (ImportUnits, Vec<SourceLine>) {
        let Self::Imports(lines) = self else {
            unreachable!(
                "`into_import_units()` must be called on `PartitionedChunk::Imports` only."
            );
        };

        let mut units = vec![];

        let mut current_leading_lines = vec![];
        for line in lines {
            match line {
                SourceLine::Import(..) => {
                    units.push(SortableImport::new(
                        std::mem::take(&mut current_leading_lines),
                        line,
                    ));
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

        (ImportUnits(units), trailing_lines)
    }
}
