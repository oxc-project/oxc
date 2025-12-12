use crate::{
    formatter::format_element::LineMode,
    ir_transform::sort_imports::{
        compute_metadata::compute_import_metadata,
        group_config::GroupName,
        options::SortImportsOptions,
        sortable_imports::{SortSortableImports, SortableImport},
        source_line::SourceLine,
    },
};

/// Orphan content (comments/empty lines separated by empty line from the next import).
/// These stay at their original slot position after sorting.
#[derive(Debug)]
pub struct OrphanContent<'a> {
    pub lines: Vec<SourceLine<'a>>,
    /// The slot position:
    /// - `None`: before the first import (leading orphan)
    /// - `Some(n)`: after the Nth import (0-indexed)
    pub after_slot: Option<usize>,
}

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

    /// Convert this import chunk into `SortableImport` units with `OrphanContent`.
    /// Returns a tuple of `(sortable_imports, orphan_contents, trailing_lines)`.
    ///
    /// - `sortable_imports`: Import statements with their attached leading lines.
    ///   - `leading_lines`: Comments directly before this import (no empty line between).
    /// - `orphan_contents`: Orphan comments (separated by empty line from next import) with their slot positions.
    ///   - `after_slot: None` = before first import, `Some(n)` = after slot n
    /// - `trailing_lines`: Lines at the end of the chunk after all imports.
    #[must_use]
    pub fn into_sorted_import_units(
        self,
        groups: &[Vec<GroupName>],
        options: &SortImportsOptions,
    ) -> (Vec<SortableImport<'a>>, Vec<OrphanContent<'a>>, Vec<SourceLine<'a>>) {
        let Self::Imports(lines) = self else {
            unreachable!(
                "`into_import_units()` must be called on `PartitionedChunk::Imports` only."
            );
        };

        let mut sortable_imports: Vec<SortableImport<'a>> = vec![];
        let mut orphan_contents: Vec<OrphanContent<'a>> = vec![];

        // Comments separated from the next import by empty line.
        // These stay at their slot position, not attached to any import.
        let mut orphan_pending: Vec<SourceLine<'a>> = vec![];
        // Comments directly before the next import (no empty line between).
        // These attach to the next import as leading lines.
        let mut current_pending: Vec<SourceLine<'a>> = vec![];

        for line in lines {
            match line {
                SourceLine::Import(_, ref metadata) => {
                    // Handle orphan content (separated by empty line from this import)
                    // These stay at their original slot position, not attached to any import.
                    if !orphan_pending.is_empty() {
                        // Slot position: None = before first import, Some(n) = after slot n
                        let after_slot = if sortable_imports.is_empty() {
                            None
                        } else {
                            Some(sortable_imports.len() - 1)
                        };
                        // For leading orphans (None), keep all lines including empty lines
                        // For other orphans, keep only comments
                        let lines: Vec<_> = if after_slot.is_none() {
                            std::mem::take(&mut orphan_pending)
                        } else {
                            std::mem::take(&mut orphan_pending)
                                .into_iter()
                                .filter_map(|orphan| {
                                    if let SourceLine::CommentOnly(range, _) = orphan {
                                        Some(SourceLine::CommentOnly(range, LineMode::Hard))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        };
                        if !lines.is_empty() {
                            orphan_contents.push(OrphanContent { lines, after_slot });
                        }
                    }

                    let is_side_effect = metadata.is_side_effect;
                    let (group_idx, normalized_source, is_ignored) =
                        compute_import_metadata(metadata, groups, options);

                    sortable_imports.push(SortableImport {
                        leading_lines: std::mem::take(&mut current_pending),
                        import_line: line,
                        is_side_effect,
                        group_idx,
                        normalized_source,
                        is_ignored,
                    });
                }
                SourceLine::Empty => {
                    // Empty line separates comments from the next import.
                    // Move `current_pending` to `orphan_pending`, then add empty line.
                    orphan_pending.append(&mut current_pending);
                    orphan_pending.push(line);
                }
                SourceLine::CommentOnly(..) => {
                    current_pending.push(line);
                }
                SourceLine::Others(..) => {
                    unreachable!(
                        "`PartitionedChunk::Imports` must not contain `SourceLine::Others`."
                    );
                }
            }
        }

        // Any remaining lines are trailing
        // Combine orphan and current pending as trailing lines
        orphan_pending.append(&mut current_pending);
        let trailing_lines = orphan_pending;

        // Let's sort this chunk!
        sortable_imports.sort(options);

        (sortable_imports, orphan_contents, trailing_lines)
    }
}
