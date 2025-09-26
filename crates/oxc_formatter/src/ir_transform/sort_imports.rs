use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode,
        document::Document,
        tag::{LabelId, Tag},
    },
    options::SortImports,
};

pub struct SortImportsTransform {
    options: SortImports,
}

impl SortImportsTransform {
    pub fn new(options: SortImports) -> Self {
        Self { options }
    }

    pub fn transform<'a>(&self, document: &Document<'a>) -> Document<'a> {
        let elements: &[FormatElement<'a>] = document;

        // FIXME: Avoid clone everywhere!

        // As a preprocessing, split elements into `Line`s.
        // Literally, a `Line` is a group of elements separated by `FormatElement::Line(_)`.
        //
        // Roughly speaking, sort-imports is a process of swapping lines.
        // It is meaningful to explicitly handle comments, empty lines,
        // and also lines with or without import statements.
        //
        // NOTE: `FormatElement::Line(_)` may not exactly correspond to an actual line break in the output.
        // e.g. `LineMode::SoftOrSpace` may be rendered as a space.
        // Also, in such cases, the start and end of a `Tag` are often split across different `Line`s.
        // If we restore them with wrong order, it will cause errors during printing.
        // But this never happens unless `Chunk`s are sorted.
        let mut lines = vec![];
        let mut current_line_indices = vec![];
        for (idx, el) in elements.iter().enumerate() {
            if let FormatElement::Line(mode) = el {
                // Flush current line
                lines.push(Line::new(
                    LineType::from_element_indices(&current_line_indices, elements),
                    current_line_indices,
                    *mode,
                ));
                current_line_indices = vec![];

                if matches!(mode, LineMode::Empty) {
                    lines.push(Line::new(LineType::Empty, vec![], LineMode::Empty));
                }

                continue;
            }

            current_line_indices.push(idx);
        }
        if !current_line_indices.is_empty() {
            unreachable!("`Document` must end with a `FormatElement::Line(_)`.");
        }

        // First, partition lines into `Chunk`s.
        // We will only sort import lines within each chunk.
        //
        // A `Chunk` is a group of lines separated by boundary like:
        // - Non-import lines: Always a boundary
        // - Empty lines: if `partition_by_newline: true`
        // - Comment lines: if `partition_by_comment: true`
        let mut chunks = vec![];
        let mut current_chunk = Chunk::new(vec![]);
        for line in lines {
            if matches!(line.r#type, LineType::Import(_)) {
                current_chunk.push(line, true);
                continue;
            }

            if line.r#type == LineType::Empty && !self.options.partition_by_newline {
                current_chunk.push(line, false);
                continue;
            }
            // TODO: Support more flexible comment handling?
            // e.g. Specific text by regex, only line comments, etc.
            if line.r#type == LineType::Comment && !self.options.partition_by_comment {
                current_chunk.push(line, false);
                continue;
            }

            // This line is a boundary!
            // Generally, `LineType::Other` should always reach here.
            chunks.push(current_chunk);
            current_chunk = Chunk::new(vec![line]);
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // Now, sort import lines within each chunk.
        // After sorting, flatten everything back to `FormatElement`s.
        let mut new_elements = vec![];
        for chunk in &mut chunks {
            // If the chunk has 0 or 1 import statements, no need to sort
            if chunk.import_count < 2 {
                for line in &chunk.lines {
                    for idx in &line.element_indices {
                        // TODO: Avoid clone if possible
                        new_elements.push(elements[*idx].clone());
                    }
                    new_elements.push(FormatElement::Line(line.line_mode));
                }
                continue;
            }

            // Otherwise, we need to sort imports in this chunk.
            // Convert lines into `ImportUnit`s (comments + import) for sorting.
            // and also collect trailing lines (empty lines, standalone comments, etc.) after all imports.
            // FIXME: This can be done like `chunks.push(current_chunk.finalize())`
            let (mut import_units, trailing_lines) = chunk.to_import_units(elements);
            // TODO: Sort by `options.groups`
            import_units.sort_by(|a, b| a.source.cmp(&b.source));

            // Finally, output sorted import units and trailing lines.
            for unit in &import_units {
                // Output leading comments first
                for line in &unit.comment_lines {
                    for idx in &line.element_indices {
                        new_elements.push(elements[*idx].clone());
                    }
                    new_elements.push(FormatElement::Line(line.line_mode));
                }
                // Then output the import line
                for idx in &unit.import_line.element_indices {
                    new_elements.push(elements[*idx].clone());
                }
                // Import line should end with a hard line break
                new_elements.push(FormatElement::Line(LineMode::Hard));
            }
            // Output trailing lines
            for line in &trailing_lines {
                for idx in &line.element_indices {
                    new_elements.push(elements[*idx].clone());
                }
                new_elements.push(FormatElement::Line(line.line_mode));
            }
        }

        Document::from(new_elements)
    }
}

#[derive(Debug, Clone)]
struct Line {
    r#type: LineType,
    /// Indices of elements in the original `Document` that belong to this line.
    element_indices: Vec<usize>,
    /// `LineMode` for this line.
    /// Keep it to restore later, especially for non-import lines.
    line_mode: LineMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LineType {
    Empty,
    Comment,
    Import(String), // The import source
    Other,
}

impl LineType {
    fn from_element_indices(element_indices: &[usize], elements: &[FormatElement]) -> LineType {
        if element_indices.is_empty() {
            return LineType::Empty;
        }

        // Sometimes, there might be leading comments in the same line,
        // so we need to check all elements in the line.
        // ```
        // /* THIS */ import ...
        // import ...
        // ```
        let mut in_import = false;
        let mut after_from = false;
        for idx in element_indices {
            match &elements[*idx] {
                FormatElement::Tag(Tag::StartLabelled(id))
                    if *id == LabelId::of(JsLabels::ImportDeclaration) =>
                {
                    in_import = true;
                }
                FormatElement::Tag(Tag::EndLabelled) if in_import => {
                    break;
                }
                FormatElement::StaticText { text } if in_import => {
                    if text.contains("from") {
                        after_from = true;
                    }
                }
                FormatElement::LocatedTokenText { slice, .. } if in_import && after_from => {
                    let text = &**slice;
                    if text.starts_with('"') || text.starts_with('\'') {
                        let source = text.trim_matches('"').trim_matches('\'').to_string();
                        return LineType::Import(source);
                    }
                }
                _ => {}
            }
        }

        // Comment line is a line that starts with a comment token
        if let Some(idx) = element_indices.first() {
            match &elements[*idx] {
                FormatElement::DynamicText { text }
                    if text.starts_with("//") || text.starts_with("/*") =>
                {
                    return LineType::Comment;
                }
                _ => {}
            }
        }

        LineType::Other
    }
}

impl Line {
    fn new(r#type: LineType, element_indices: Vec<usize>, line_mode: LineMode) -> Self {
        Self { r#type, element_indices, line_mode }
    }
}

#[derive(Debug, Clone)]
struct Chunk {
    lines: Vec<Line>,
    import_count: u16,
}

impl Chunk {
    fn new(lines: Vec<Line>) -> Self {
        Self { lines, import_count: 0 }
    }

    fn push(&mut self, line: Line, is_import: bool) {
        self.lines.push(line);

        if is_import {
            self.import_count += 1;
        }
    }

    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn to_import_units(&self, elements: &[FormatElement]) -> (Vec<ImportUnit>, Vec<Line>) {
        let mut units = vec![];
        let mut trailing_lines = vec![];

        let mut current_comment_lines = vec![];
        for line in &self.lines {
            if matches!(line.r#type, LineType::Import(_)) {
                units.push(ImportUnit::new(current_comment_lines, line.clone()));
                current_comment_lines = vec![];
            } else if line.r#type == LineType::Comment {
                // Comments are always potential leading comments for the next import
                current_comment_lines.push(line.clone());
            } else {
                // Empty line - breaks the comment chain
                // Include it with the current comments if they exist
                if current_comment_lines.is_empty() {
                    // Standalone empty line after all imports
                    trailing_lines.push(line.clone());
                } else {
                    current_comment_lines.push(line.clone());
                }
            }
        }

        // Any remaining comments/lines are trailing
        trailing_lines.extend(current_comment_lines);

        (units, trailing_lines)
    }
}

#[derive(Debug, Clone)]
struct ImportUnit {
    comment_lines: Vec<Line>,
    import_line: Line,
    source: String,
}

impl ImportUnit {
    fn new(comment_lines: Vec<Line>, import_line: Line) -> Self {
        let source = if let LineType::Import(source) = &import_line.r#type {
            source.clone()
        } else {
            unreachable!("`import_line` must be of type `LineType::Import`.");
        };
        Self { comment_lines, import_line, source }
    }
}
