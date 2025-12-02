use crate::ir_transform::sort_imports::{
    compute_metadata::compute_import_metadata,
    group_config::GroupName,
    options::SortImportsOptions,
    sortable_imports::{SortSortableImports, SortableImport},
    source_line::SourceLine,
};

#[derive(Debug)]
pub enum PartitionedChunk<'a> {
    /// A chunk containing import statements,
    /// and possibly leading/trailing comments or empty lines.
    Imports(Vec<SourceLine<'a>>),
    /// A boundary chunk.
    /// Always contains `SourceLine::Others`,
    /// or optionally `SourceLine::Empty|CommentOnly` depending on partition options.
    Boundary(SourceLine<'a>),
}

impl Default for PartitionedChunk<'_> {
    fn default() -> Self {
        Self::Imports(vec![])
    }
}

impl<'a> PartitionedChunk<'a> {
    pub fn add_imports_line(&mut self, line: SourceLine<'a>) {
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
    pub fn into_sorted_import_units(
        self,
        groups: &[Vec<GroupName>],
        options: &SortImportsOptions,
    ) -> (Vec<SortableImport<'a>>, Vec<SourceLine<'a>>) {
        let Self::Imports(lines) = self else {
            unreachable!(
                "`into_import_units()` must be called on `PartitionedChunk::Imports` only."
            );
        };

        let mut sortable_imports = vec![];
        let mut current_leading_lines = vec![];
        for line in lines {
            match line {
                SourceLine::Import(_, ref metadata) => {
                    let is_side_effect = metadata.is_side_effect;
                    let (group_idx, normalized_source, is_ignored) =
                        compute_import_metadata(metadata, groups, options);
                    sortable_imports.push(SortableImport {
                        leading_lines: std::mem::take(&mut current_leading_lines),
                        import_line: line,
                        is_side_effect,
                        group_idx,
                        normalized_source,
                        is_ignored,
                    });
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

        // Let's sort this chunk!
        sortable_imports.sort(options);

        (sortable_imports, trailing_lines)
    }
}
