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
    // NOTE: `Document` and its `FormatElement`s are already well-formatted.
    // It means that:
    // - There is no redundant spaces/lines.
    // - Last element is always `FormatElement::Line(Hard)`.
    pub fn transform<'a>(&self, document: &Document<'a>) -> Document<'a> {
        let prev_elements: &[FormatElement<'a>] = document;

        // Roughly speaking, sort-imports is a process of swapping lines.
        // Therefore, as a preprocessing, group IR elements into `Line`s first.
        // e.g.
        // ```
        // [Text, Space, Text, Line, Tag, Text, Text, Line, ...]
        // ```
        // ↓↓
        // ```
        // [ [Text, Space, Text], [Tag, Text, Text], [...] ]
        // ```
        //
        // Literally, a `Line` is a group of elements separated by `FormatElement::Line(_)`.
        //
        // This is also meaningful to explicitly handle comment line, empty line,
        // and other line with or without import statement.
        //
        // NOTE: `FormatElement::Line(_)` may not exactly correspond to an actual line break in the output.
        // e.g. `LineMode::SoftOrSpace` may be rendered as a space.
        //
        // And this implementation is based on the following assumptions:
        // - `Line(Hard|Empty)` is used for joining `Program.body` in the output
        // - `Line(Hard|Empty)` does not appear inside an `ImportDeclaration` formatting
        //   - In case of this, we will check `Tag::StartLabelled(JsLabels::ImportDeclaration)`
        let mut lines = vec![];
        let mut current_line_indices = vec![];
        for (idx, el) in prev_elements.iter().enumerate() {
            if let FormatElement::Line(mode) = el
                && matches!(mode, LineMode::Empty | LineMode::Hard)
            {
                // Flush current line
                lines.push(Line::from_element_indices(prev_elements, &current_line_indices, *mode));
                current_line_indices = vec![];

                // We need this explicitly
                if matches!(mode, LineMode::Empty) {
                    lines.push(Line::Empty(*mode));
                }

                continue;
            }

            current_line_indices.push(idx);
        }
        if !current_line_indices.is_empty() {
            unreachable!("`Document` must end with a `FormatElement::Line(Hard)`.");
        }

        // Next, partition `Line`s into `Chunk`s.
        //
        // Within each chunk, we will sort import lines.
        // e.g.
        // ```
        // import C from "c";
        // import B from "b";
        // const THIS_IS_BOUNDARY = true;
        // import Z from "z";
        // import A from "a";
        // ```
        // ↓↓
        // ```
        // import B from "b";
        // import C from "c";
        // const THIS_IS_BOUNDARY = true;
        // import A from "a";
        // import Z from "z";
        // ```
        let mut chunks = vec![];
        let mut current_chunk = Chunk::default();
        for line in lines {
            // `Line::Import` never be a boundary.
            if matches!(line, Line::Import(..)) {
                current_chunk.add(line);
                continue;
            }

            // `Line::Empty` and `Line::CommentOnly` can be boundaries depending on options.
            // Otherwise, they will be the leading/trailing lines of `Line::Import` in the chunk.
            if matches!(line, Line::Empty(..)) && !self.options.partition_by_newline {
                current_chunk.add(line);
                continue;
            }
            // TODO: Support more flexible comment handling?
            // e.g. Specific text by regex, only line comments, etc.
            if matches!(line, Line::CommentOnly(..)) && !self.options.partition_by_comment {
                current_chunk.add(line);
                continue;
            }

            // This line is a boundary!
            // Generally, `Line::Others` should always reach here.

            // Flush current chunk
            chunks.push(current_chunk);
            current_chunk = Chunk::default();

            // Flush the boundary line as a separate chunk
            let mut boundary_chunk = Chunk::default();
            boundary_chunk.add(line);
            chunks.push(boundary_chunk);
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // Finally, sort import lines within each chunk.
        // After sorting, flatten everything back to `FormatElement`s.
        let mut next_elements = vec![];
        for chunk in &mut chunks {
            // We can skip sorting if not needed.
            if !chunk.is_sort_needed() {
                for line in &chunk.lines {
                    line.write(prev_elements, &mut next_elements);
                }
                continue;
            }

            // Here, we need to sort imports in this chunk.
            // Before sorting, convert `Chunk` into `ImportUnit`s.
            //
            // `ImportUnit` is a logical unit of import statement + its N leading lines.
            // And there may be trailing lines after all import statements in the chunk.
            // e.g.
            // ```
            // const THIS_IS_BOUNDARY = true;
            // // comment for A
            // import A from "a";
            // import B from "b";
            //
            // // comment for C and empty line above + below
            //
            // // another comment for C
            // import C from "c";
            // // trailing comment and empty line below for this chunk
            //
            // const YET_ANOTHER_BOUNDARY = true;
            // ```
            //
            let (mut import_units, trailing_lines) = chunk.to_import_units(prev_elements);

            // TODO: Sort based on `options.groups`, `options.type`, etc...
            // TODO: Consider `options.ignore_case`, `special_characters`, removing `?raw`, etc...
            import_units.sort_by_key(|unit| unit.get_source(prev_elements));

            // Output sorted import units and trailing lines.
            for ImportUnit { leading_lines, import_line } in &import_units {
                for line in leading_lines {
                    line.write(prev_elements, &mut next_elements);
                }
                import_line.write(prev_elements, &mut next_elements);
            }
            for line in &trailing_lines {
                line.write(prev_elements, &mut next_elements);
            }
        }

        Document::from(next_elements)
    }
}

#[derive(Debug, Clone)]
enum Line {
    /// Line that contains an import statement.
    /// May have leading comments like `/* ... */ import ...`.
    /// And also may have trailing comments like `import ...; // ...`.
    ///
    /// The 3rd field is the index of the original `elements` for import `source`.
    /// The 4th field indicates whether this is a side-effect-only import.
    // TODO: Consider using struct
    Import(Vec<usize>, LineMode, usize, bool),
    /// Empty line.
    /// May be used as a boundary if `options.partition_by_newline` is true.
    Empty(LineMode),
    /// Line that contains only comments.
    /// May be used as a boundary if `options.partition_by_comment` is true.
    CommentOnly(Vec<usize>, LineMode),
    /// Other lines, always a boundary.
    Others(Vec<usize>, LineMode),
}

impl Line {
    fn from_element_indices(
        elements: &[FormatElement],
        element_indices: &[usize],
        line_mode: LineMode,
    ) -> Self {
        debug_assert!(
            !element_indices.is_empty(),
            "`element_indices` must not be empty, othereise use `Line::Empty` directly."
        );

        // Check if the line is comment-only.
        // e.g.
        // ```
        // // comment
        // /* comment */
        // /* comment */ // comment
        // /* comment */ /* comment */
        // ```
        let is_comment_only = element_indices.iter().all(|&idx| match &elements[idx] {
            FormatElement::DynamicText { text } => text.starts_with("//") || text.starts_with("/*"),
            FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace) | FormatElement::Space => {
                true
            }
            _ => false,
        });
        if is_comment_only {
            // TODO: Check it contains ignore comment?
            return Line::CommentOnly(element_indices.to_vec(), line_mode);
        }

        // Sometimes, there might be leading comments in the same line as the import statement,
        // so we need to check all elements in the line to find `ImportDeclaration`.
        // ```
        // /* THIS */ import ...
        // import ...
        // ```
        let mut has_import = false;
        let mut source_idx = None;
        let mut is_side_effect_import = true;
        for idx in element_indices {
            match &elements[*idx] {
                // Special marker for `ImportDeclaration`
                FormatElement::Tag(Tag::StartLabelled(id))
                    if *id == LabelId::of(JsLabels::ImportDeclaration) =>
                {
                    has_import = true;
                }
                // `ImportDeclaration.source: StringLiteral` is formatted as either:
                // - `LocatedTokenText` (when borrowed, quote unchanged)
                // - `DynamicText` (when owned, quote normalized)
                FormatElement::LocatedTokenText { .. } | FormatElement::DynamicText { .. } => {
                    if has_import && source_idx.is_none() {
                        source_idx = Some(*idx);
                    }
                }
                FormatElement::StaticText { text } => {
                    if has_import && *text == "from" {
                        is_side_effect_import = false;
                    }
                }
                _ => {}
            }
        }
        if has_import && let Some(source_idx) = source_idx {
            // TODO: Check line has trailing ignore comment?
            return Line::Import(
                element_indices.to_vec(),
                line_mode,
                source_idx,
                is_side_effect_import,
            );
        }

        // Otherwise, this line is neither of:
        // - Empty line
        // - Comment-only line
        // - Import line
        // So, it will be a boundary line.
        Line::Others(element_indices.to_vec(), line_mode)
    }

    fn write<'a>(
        &self,
        prev_elements: &[FormatElement<'a>],
        next_elements: &mut Vec<FormatElement<'a>>,
    ) {
        match self {
            Line::Empty(..) => { /* Nothing */ }
            Line::CommentOnly(indices, ..)
            | Line::Import(indices, ..)
            | Line::Others(indices, ..) => {
                for idx in indices {
                    next_elements.push(prev_elements[*idx].clone());
                }
            }
        }
        // NOTE: This may add empty lines.
        // May be better to omit them if `options.partition_by_newline: false`.
        next_elements.push(FormatElement::Line(match self {
            Line::Empty(mode)
            | Line::CommentOnly(_, mode)
            | Line::Import(_, mode, ..)
            | Line::Others(_, mode) => *mode,
        }));
    }
}

#[derive(Debug, Clone, Default)]
struct Chunk {
    lines: Vec<Line>,
}

impl Chunk {
    fn add(&mut self, line: Line) {
        self.lines.push(line);
    }

    /// If this chunk has 0 or 1 import statements, no need to sort.
    fn is_sort_needed(&self) -> bool {
        1 < self.lines.iter().filter(|line| matches!(line, Line::Import(..))).count()
    }

    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    #[must_use]
    fn to_import_units(&self, elements: &[FormatElement]) -> (Vec<ImportUnit>, Vec<Line>) {
        let mut units = vec![];
        let mut trailing_lines = vec![];

        let mut current_leading_lines = vec![];
        for line in &self.lines {
            match line {
                Line::Import(..) => {
                    units.push(ImportUnit::new(current_leading_lines, line.clone()));
                    current_leading_lines = vec![];
                }
                Line::CommentOnly(..) | Line::Empty(..) => {
                    current_leading_lines.push(line.clone());
                }
                Line::Others(..) => {
                    unreachable!(
                        "`Chunk::to_import_units()` must be called for `is_sort_needed() == true` chunks."
                    );
                }
            }
        }

        // Any remaining comments/lines are trailing
        trailing_lines.extend(current_leading_lines);

        (units, trailing_lines)
    }
}

#[derive(Debug, Clone)]
struct ImportUnit {
    leading_lines: Vec<Line>,
    import_line: Line,
}

impl ImportUnit {
    fn new(leading_lines: Vec<Line>, import_line: Line) -> Self {
        Self { leading_lines, import_line }
    }

    fn get_source<'a>(&self, elements: &'a [FormatElement]) -> &'a str {
        let Line::Import(_, _, source_idx, _) = self.import_line else {
            unreachable!("`import_line` must be of type `Line::Import`.");
        };
        match &elements[source_idx] {
            FormatElement::LocatedTokenText { slice, .. } => slice.as_ref(),
            FormatElement::DynamicText { text } => text,
            _ => unreachable!("`source_idx` must point to either `LocatedTokenText` or `DynamicText` in the `elements`."),
        }
    }
}
