use std::ops::Range;

use cow_utils::CowUtils;

use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode,
        document::Document,
        tag::{LabelId, Tag},
    },
    options,
};

pub struct SortImportsTransform {
    options: options::SortImports,
}

impl SortImportsTransform {
    pub fn new(options: options::SortImports) -> Self {
        Self { options }
    }

    /// Transform the given `Document` by sorting import statements according to the specified options.
    ///
    // NOTE: `Document` and its `FormatElement`s are already well-formatted.
    // It means that:
    // - There is no redundant spaces, no consecutive line breaks, etc...
    // - Last element is always `FormatElement::Line(Hard)`.
    pub fn transform<'a>(&self, document: &Document<'a>) -> Document<'a> {
        // Early return for empty files
        if document.len() == 1 && matches!(document[0], FormatElement::Line(LineMode::Hard)) {
            return document.clone();
        }

        let prev_elements: &[FormatElement<'a>] = document;

        // Roughly speaking, sort-imports is a process of swapping lines.
        // Therefore, as a preprocessing, group IR elements into line first.
        // e.g.
        // ```
        // [Text, Space, Text, Line, StartTag, Text, Text, EndTag, Line, ...]
        // ```
        // ↓↓
        // ```
        // [ [Text, Space, Text], [StartTag, Text, Text, EndTag], [...] ]
        // ```
        //
        // This is also meaningful to explicitly handle comment line, empty line,
        // and other line with or without import statement.
        //
        // NOTE: `FormatElement::Line(_)` may not exactly correspond to an actual line break in the output.
        // e.g. `LineMode::SoftOrSpace` may be rendered as a space.
        //
        // And this implementation is based on the following assumptions:
        // - Only `Line(Hard|Empty)` is used for joining `Program.body` in the output
        // - `Line(Hard|Empty)` does not appear inside an `ImportDeclaration` formatting
        //   - In case of this, we should check `Tag::StartLabelled(JsLabels::ImportDeclaration)`
        let mut lines = vec![];
        let mut current_line_start = 0;
        for (idx, el) in prev_elements.iter().enumerate() {
            if let FormatElement::Line(mode) = el
                && matches!(mode, LineMode::Empty | LineMode::Hard)
            {
                // Flush current line
                if current_line_start < idx {
                    lines.push(SourceLine::from_element_range(
                        prev_elements,
                        current_line_start..idx,
                        *mode,
                    ));
                }
                current_line_start = idx + 1;

                // We need this explicitly to detect boundaries later.
                if matches!(mode, LineMode::Empty) {
                    lines.push(SourceLine::Empty);
                }
            }
        }
        if current_line_start < prev_elements.len() {
            unreachable!("`Document` must end with a `FormatElement::Line(Hard)`.");
        }

        // Next, partition `SourceLine`s into `PartitionedChunk`s.
        //
        // Within each chunk, we will sort import lines.
        // e.g.
        // ```
        // import C from "c"; // chunk1
        // import B from "b"; // chunk1
        // const THIS_IS_BOUNDARY = true;
        // import Z from "z"; // chunk2
        // import A from "a"; // chunk2
        // ```
        // ↓↓
        // ```
        // import B from "b"; // chunk1
        // import C from "c"; // chunk1
        // const THIS_IS_BOUNDARY = true;
        // import A from "a"; // chunk2
        // import Z from "z"; // chunk2
        // ```
        let mut chunks = vec![];
        let mut current_chunk = PartitionedChunk::default();
        for line in lines {
            match line {
                // `SourceLine::Import` never be a boundary.
                SourceLine::Import(..) => {
                    current_chunk.add_imports_line(line);
                }
                // `SourceLine::Empty` and `SourceLine::CommentOnly` can be boundaries depending on options.
                // Otherwise, they will be the leading/trailing lines of `PartitionedChunk::Imports`.
                SourceLine::Empty if !self.options.partition_by_newline => {
                    current_chunk.add_imports_line(line);
                }
                // TODO: Support more flexible comment handling?
                // e.g. Specific text by regex, only line comments, etc.
                SourceLine::CommentOnly(..) if !self.options.partition_by_comment => {
                    current_chunk.add_imports_line(line);
                }
                // This `SourceLine` is a boundary!
                // Generally, `SourceLine::Others` should always reach here.
                _ => {
                    // Flush current import chunk
                    if !current_chunk.is_empty() {
                        chunks.push(std::mem::take(&mut current_chunk));
                    }
                    // Add boundary chunk
                    chunks.push(PartitionedChunk::Boundary(line));
                }
            }
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // Finally, sort import lines within each chunk.
        // After sorting, flatten everything back to `FormatElement`s.
        let mut next_elements = vec![];

        let mut chunks_iter = chunks.into_iter().enumerate().peekable();
        while let Some((idx, chunk)) = chunks_iter.next() {
            match chunk {
                // Boundary chunks: Just output as-is
                PartitionedChunk::Boundary(line) => {
                    line.write(prev_elements, &mut next_elements, true);
                }
                // Import chunks: Sort and output
                PartitionedChunk::Imports(_) => {
                    // For ease of implementation, we will convert `ImportChunk` into multiple `SortableImport`s.
                    //
                    // `SortableImport` is a logical unit of 1 import statement + its N leading lines.
                    // And there may be trailing lines after all import statements in the chunk.
                    // e.g.
                    // ```
                    // const THIS_IS_BOUNDARY = true;
                    // // comment for A
                    // import A from "a"; // sortable1
                    // import B from "b"; // sortable2
                    //
                    // // comment for C and empty line above + below
                    //
                    // // another comment for C
                    // import C from "c"; // sortable3
                    // // trailing comment and empty line below for this chunk
                    //
                    // const YET_ANOTHER_BOUNDARY = true;
                    // ```
                    let (mut import_units, trailing_lines) = chunk.into_import_units(prev_elements);
                    import_units.sort_imports(prev_elements, self.options);

                    // Output sorted import units
                    let preserve_empty_line = self.options.partition_by_newline;
                    for SortableImport { leading_lines, import_line } in import_units {
                        for line in leading_lines {
                            line.write(prev_elements, &mut next_elements, preserve_empty_line);
                        }
                        import_line.write(prev_elements, &mut next_elements, false);
                    }
                    // And output trailing lines
                    //
                    // Special care is needed for the last empty line.
                    // We should preserve it only if the next chunk is a boundary.
                    // e.g.
                    // ```
                    // import A from "a"; // chunk1
                    // import B from "b"; // chunk1
                    // // This empty line should be preserved because the next chunk is a boundary.
                    //
                    // const BOUNDARY = true; // chunk2
                    // ```
                    // But in this case, we should not preserve it.
                    // ```
                    // import A from "a"; // chunk1
                    // import B from "b"; // chunk1
                    // // This empty line should NOT be preserved because the next chunk is NOT a boundary.
                    //
                    // import C from "c"; // chunk2
                    // ```
                    let next_chunk_is_boundary = chunks_iter
                        .peek()
                        .is_some_and(|(_, c)| matches!(c, PartitionedChunk::Boundary(_)));
                    for (idx, line) in trailing_lines.iter().enumerate() {
                        let is_last_empty_line =
                            idx == trailing_lines.len() - 1 && matches!(line, SourceLine::Empty);
                        let preserve_empty_line =
                            if is_last_empty_line { next_chunk_is_boundary } else { true };
                        line.write(prev_elements, &mut next_elements, preserve_empty_line);
                    }
                }
            }
        }

        Document::from(next_elements)
    }
}

#[derive(Debug, Clone)]
enum SourceLine {
    /// Line that contains an import statement.
    /// May have leading comments like `/* ... */ import ...`.
    /// And also may have trailing comments like `import ...; // ...`.
    ///
    /// The 2nd field is the index of the original `elements` for import `source`.
    /// The 3rd field indicates whether this is a side-effect-only import.
    ///
    /// Never be a boundary.
    // TODO: Consider using struct
    Import(Range<usize>, usize, bool),
    /// Empty line.
    /// May be used as a boundary if `options.partition_by_newline` is true.
    Empty,
    /// Line that contains only comment(s).
    /// May be used as a boundary if `options.partition_by_comment` is true.
    CommentOnly(Range<usize>, LineMode),
    /// Other lines, always a boundary.
    Others(Range<usize>, LineMode),
}

impl SourceLine {
    fn from_element_range(
        elements: &[FormatElement],
        range: Range<usize>,
        line_mode: LineMode,
    ) -> Self {
        debug_assert!(
            !range.is_empty(),
            "`range` must not be empty, otherwise use `SourceLine::Empty` directly."
        );

        // Check if the line is comment-only.
        // e.g.
        // ```
        // // comment
        // /* comment */
        // /* comment */ // comment
        // /* comment */ /* comment */
        // ```
        let is_comment_only = range.clone().all(|idx| match &elements[idx] {
            FormatElement::DynamicText { text } => text.starts_with("//") || text.starts_with("/*"),
            FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace) | FormatElement::Space => {
                true
            }
            _ => false,
        });
        if is_comment_only {
            // TODO: Check it contains ignore comment?
            return SourceLine::CommentOnly(range, line_mode);
        }

        // Check if the line contains an import statement.
        // Sometimes, there might be leading comments in the same line,
        // so we need to check all elements in the line to find an `ImportDeclaration`.
        // ```
        // /* THIS */ import ...
        // import ...
        // ```
        let mut has_import = false;
        let mut source_idx = None;
        let mut is_side_effect_import = true;
        for idx in range.clone() {
            match &elements[idx] {
                // Special marker for `ImportDeclaration`
                FormatElement::Tag(Tag::StartLabelled(id))
                    if *id == LabelId::of(JsLabels::ImportDeclaration) =>
                {
                    has_import = true;
                }
                FormatElement::StaticText { text } => {
                    if has_import && *text == "from" {
                        is_side_effect_import = false;
                        // Reset `source_idx` to ensure we get the text after "from".
                        // `ImportSpecifier` may appear before `source`.
                        source_idx = None;
                    }
                }
                // `ImportDeclaration.source: StringLiteral` is formatted as either:
                // - `LocatedTokenText` (when borrowed, quote unchanged)
                // - `DynamicText` (when owned, quote normalized)
                FormatElement::LocatedTokenText { .. } | FormatElement::DynamicText { .. } => {
                    if has_import && source_idx.is_none() {
                        source_idx = Some(idx);
                    }
                }
                _ => {}
            }
        }
        if has_import && let Some(source_idx) = source_idx {
            // TODO: Check line has trailing ignore comment?
            return SourceLine::Import(range, source_idx, is_side_effect_import);
        }

        // Otherwise, this line is neither of:
        // - Empty line
        // - Comment-only line
        // - Import line
        // So, it will be a boundary line.
        SourceLine::Others(range, line_mode)
    }

    fn write<'a>(
        &self,
        prev_elements: &[FormatElement<'a>],
        next_elements: &mut Vec<FormatElement<'a>>,
        preserve_empty_line: bool,
    ) {
        match self {
            SourceLine::Empty => {
                // Skip empty lines unless they should be preserved
                if preserve_empty_line {
                    next_elements.push(FormatElement::Line(LineMode::Empty));
                }
            }
            SourceLine::Import(range, ..) => {
                for idx in range.clone() {
                    next_elements.push(prev_elements[idx].clone());
                }
                // Always use hard line break after import statement.
                next_elements.push(FormatElement::Line(LineMode::Hard));
            }
            SourceLine::CommentOnly(range, mode) | SourceLine::Others(range, mode) => {
                for idx in range.clone() {
                    next_elements.push(prev_elements[idx].clone());
                }
                next_elements.push(FormatElement::Line(*mode));
            }
        }
    }
}

#[derive(Debug, Clone)]
enum PartitionedChunk {
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
    fn add_imports_line(&mut self, line: SourceLine) {
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

    fn is_empty(&self) -> bool {
        matches!(self, Self::Imports(lines) if lines.is_empty())
    }

    #[must_use]
    fn into_import_units(self, elements: &[FormatElement]) -> (ImportUnits, Vec<SourceLine>) {
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

#[derive(Debug)]
struct ImportUnits(Vec<SortableImport>);

impl IntoIterator for ImportUnits {
    type Item = SortableImport;
    type IntoIter = std::vec::IntoIter<SortableImport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ImportUnits {
    // TODO: Sort based on `options.groups`, `options.type`, etc...
    // TODO: Consider `special_characters`, removing `?raw`, etc...
    fn sort_imports(&mut self, elements: &[FormatElement], options: options::SortImports) {
        let imports_len = self.0.len();

        // Perform sorting only if needed
        if imports_len < 2 {
            return;
        }

        // Separate imports into:
        // - sortable: indices of imports that should be sorted
        // - fixed: indices of side-effect imports when `sort_side_effects: false`
        let mut sortable_indices = vec![];
        let mut fixed_indices = vec![];
        for (idx, si) in self.0.iter().enumerate() {
            if options.sort_side_effects || !si.is_side_effect_import() {
                sortable_indices.push(idx);
            } else {
                fixed_indices.push(idx);
            }
        }

        // Sort indices by comparing their corresponding import sources
        sortable_indices.sort_by(|&a, &b| {
            let source_a = self.0[a].get_source(elements);
            let source_b = self.0[b].get_source(elements);

            let ord = if options.ignore_case {
                source_a.cow_to_lowercase().cmp(&source_b.cow_to_lowercase())
            } else {
                source_a.cmp(source_b)
            };

            if options.order.is_desc() { ord.reverse() } else { ord }
        });

        // Create a permutation map
        let mut permutation = vec![0; imports_len];
        let mut sortable_iter = sortable_indices.into_iter();
        for (idx, perm) in permutation.iter_mut().enumerate() {
            // NOTE: This is O(n), but side-effect imports are usually few
            if fixed_indices.contains(&idx) {
                *perm = idx;
            } else if let Some(sorted_idx) = sortable_iter.next() {
                *perm = sorted_idx;
            }
        }
        debug_assert!(
            permutation.iter().copied().collect::<rustc_hash::FxHashSet<_>>().len() == imports_len,
            "`permutation` must be a valid permutation, all indices must be unique."
        );

        // Apply permutation in-place using cycle decomposition
        let mut visited = vec![false; imports_len];
        for idx in 0..imports_len {
            // Already visited or already in the correct position
            if visited[idx] || permutation[idx] == idx {
                continue;
            }
            // Follow the cycle
            let mut current = idx;
            loop {
                let next = permutation[current];
                visited[current] = true;
                if next == idx {
                    break;
                }
                self.0.swap(current, next);
                current = next;
            }
        }
        debug_assert!(self.0.len() == imports_len, "Length must remain the same after sorting.");
    }
}

#[derive(Debug, Clone)]
struct SortableImport {
    leading_lines: Vec<SourceLine>,
    import_line: SourceLine,
}

impl SortableImport {
    fn new(leading_lines: Vec<SourceLine>, import_line: SourceLine) -> Self {
        Self { leading_lines, import_line }
    }

    /// Get the import source string for sorting.
    /// e.g. `"./foo"`, `"react"`, etc...
    /// This includes quotes, but will not affect sorting.
    /// Since they are already normalized by the formatter.
    fn get_source<'a>(&self, elements: &'a [FormatElement]) -> &'a str {
        let SourceLine::Import(_, source_idx, _) = &self.import_line else {
            unreachable!("`import_line` must be of type `SourceLine::Import`.");
        };
        match &elements[*source_idx] {
            FormatElement::LocatedTokenText { slice, .. } => slice,
            FormatElement::DynamicText { text } => text,
            _ => unreachable!(
                "`source_idx` must point to either `LocatedTokenText` or `DynamicText` in the `elements`."
            ),
        }
    }

    /// Check if this import is a side-effect-only import.
    fn is_side_effect_import(&self) -> bool {
        match self.import_line {
            SourceLine::Import(_, _, is_side_effect) => is_side_effect,
            _ => unreachable!("`import_line` must be of type `SourceLine::Import`."),
        }
    }
}
