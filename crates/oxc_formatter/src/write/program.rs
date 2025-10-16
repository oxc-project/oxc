use std::ops::Deref;

use cow_utils::CowUtils;
use rustc_hash::{FxBuildHasher, FxHashSet};

use oxc_allocator::{Address, Vec as ArenaVec};
use oxc_ast::{Comment, ast::*, match_expression};
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::{ZWNBSP, is_line_terminator};

use crate::{
    Buffer, Format, FormatResult, FormatTrailingCommas, TrailingSeparator, format_args,
    formatter::{SourceText, VecBuffer, prelude::*, trivia::FormatTrailingComments},
    generated::ast_nodes::{AstNode, AstNodes},
    options::SortImports,
    utils::{
        call_expression::is_test_call_expression,
        is_long_curried_call,
        member_chain::simple_argument::SimpleArgument,
        string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
    },
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, Program<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_trailing_comments = format_once(|f| {
            let comments = f.context().comments().comments_before(self.span.end);
            write!(f, FormatTrailingComments::Comments(comments))
        });

        write!(
            f,
            [
                // BOM
                f.source_text().chars().next().is_some_and(|c| c == ZWNBSP).then_some("\u{feff}"),
                self.hashbang(),
                self.directives(),
                FormatProgramBody(self.body()),
                format_trailing_comments,
                hard_line_break()
            ]
        )
    }
}

struct FormatProgramBody<'a, 'b>(&'b AstNode<'a, ArenaVec<'a, Statement<'a>>>);

impl<'a> Deref for FormatProgramBody<'a, '_> {
    type Target = AstNode<'a, ArenaVec<'a, Statement<'a>>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatProgramBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Check if we need to sort imports
        let should_sort_imports = f.options().experimental_sort_imports.is_some();
        if should_sort_imports { self.fmt_with_sorted_imports(f) } else { self.fmt_default(f) }
    }
}

impl<'a> FormatProgramBody<'a, '_> {
    fn fmt_default(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut join = f.join_nodes_with_hardline();
        for stmt in
            self.iter().filter(|stmt| !matches!(stmt.as_ref(), Statement::EmptyStatement(_)))
        {
            let span = self.get_statement_span(stmt);
            join.entry(span, stmt);
        }
        join.finish()
    }

    fn fmt_with_sorted_imports(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let sort_options = f.options().experimental_sort_imports.unwrap();
        let statements: Vec<_> = self.iter().collect();

        // Detect all import blocks in the program
        let import_blocks = self.detect_import_blocks(&statements, sort_options, f)?;

        if import_blocks.is_empty() {
            return self.fmt_default(f);
        }

        // Output statements and sorted import blocks
        let mut current_idx = 0;
        for (block_idx, block) in import_blocks.iter().enumerate() {
            // Output statements before this import block
            if current_idx < block.start_idx {
                let mut join = f.join_nodes_with_hardline();
                for stmt in &statements[current_idx..block.start_idx] {
                    if matches!(stmt.as_ref(), Statement::EmptyStatement(_)) {
                        continue;
                    }
                    let span = self.get_statement_span(stmt);
                    join.entry(span, stmt);
                }
                join.finish()?;

                // Add line break before the import block
                write!(f, [hard_line_break()])?;
            }

            // Output this import block (already includes trailing line break)
            self.output_import_block(block, &statements, f)?;

            current_idx = block.end_idx;
        }

        // Output remaining statements
        if current_idx < statements.len() {
            // Check if we need to add empty lines before remaining statements
            if let Some(last_block) = import_blocks.last() {
                // Get the end position of the last import block
                let last_import_idx = last_block.end_idx - 1;
                if let Some(last_import_stmt) = statements.get(last_import_idx) {
                    let last_import_end = last_import_stmt.span().end;

                    // Get the start position of the first remaining statement
                    if let Some(first_remaining_stmt) = statements[current_idx..]
                        .iter()
                        .find(|s| !matches!(s.as_ref(), Statement::EmptyStatement(_)))
                    {
                        let source_text = f.source_text();

                        // Check for comments before the first remaining statement
                        let all_comments = f.comments().all_comments();
                        let leading_comments = self.find_adjacent_leading_comments(
                            all_comments,
                            &source_text,
                            first_remaining_stmt.span().start,
                        );

                        // Use the first leading comment's start if there are any, otherwise use statement start
                        let check_start = if let Some(first_comment) = leading_comments.last() {
                            first_comment.span.start
                        } else {
                            first_remaining_stmt.span().start
                        };

                        // Count newlines between last import and the check position
                        let newline_count =
                            self.count_newlines_between(&source_text, last_import_end, check_start);
                        if newline_count >= 2 {
                            write!(f, [empty_line()])?;
                        }
                    }
                }
            }

            let mut join = f.join_nodes_with_hardline();
            for stmt in &statements[current_idx..] {
                if matches!(stmt.as_ref(), Statement::EmptyStatement(_)) {
                    continue;
                }
                let span = self.get_statement_span(stmt);
                join.entry(span, stmt);
            }
            join.finish()?;
        }

        Ok(())
    }

    /// Common helper: Counts newlines (line terminators) between two positions
    #[inline]
    fn count_newlines_between(&self, source_text: &SourceText, start: u32, end: u32) -> usize {
        source_text.slice_range(start, end).chars().filter(|&c| is_line_terminator(c)).count()
    }

    /// Finds adjacent leading comments for an import statement
    fn find_adjacent_leading_comments(
        &self,
        all_comments: &[Comment],
        source_text: &SourceText,
        import_start: u32,
    ) -> Vec<Comment> {
        // Binary search to find comments before import_start
        let end_idx = all_comments
            .binary_search_by(|c| {
                if c.span.end <= import_start {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|idx| idx);

        // Work backwards to find adjacent own-line comments
        let mut result = vec![];
        let mut search_pos = import_start;

        for comment in all_comments[..end_idx].iter().rev() {
            if !self.is_own_line_comment(comment, source_text) {
                continue;
            }

            let text_between = source_text.slice_range(comment.span.end, search_pos);
            if text_between.chars().all(char::is_whitespace) {
                result.push(*comment);
                search_pos = comment.span.start;
            } else {
                break;
            }
        }

        result
    }

    /// Checks if a comment is on its own line (not inline)
    ///
    /// Note: This differs from `SourceText::is_own_line_comment()` in handling
    /// file-start comments. This version considers comments at the start of the file
    /// as own-line comments, while SourceText's version requires a preceding newline.
    #[inline]
    fn is_own_line_comment(&self, comment: &Comment, source_text: &SourceText) -> bool {
        if comment.span.start == 0 {
            return true;
        }

        // Check if there's only whitespace from line start to comment
        let text_before = source_text.slice_range(0, comment.span.start);
        #[expect(clippy::cast_possible_truncation)]
        if let Some(line_start) = text_before.rfind(is_line_terminator) {
            !source_text
                .slice_range(line_start as u32 + 1, comment.span.start)
                .chars()
                .any(|c| !c.is_whitespace())
        } else {
            // Comment is on the first line, check from start of file
            !source_text.slice_range(0, comment.span.start).chars().any(|c| !c.is_whitespace())
        }
    }

    /// Detect all import blocks in the program
    fn detect_import_blocks(
        &self,
        statements: &[&AstNode<'a, Statement<'a>>],
        sort_options: SortImports,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<Vec<ImportBlock<'a>>> {
        let mut blocks = vec![];
        let mut block_start: Option<usize> = None;
        let mut block_imports = vec![];

        for (idx, stmt) in statements.iter().enumerate() {
            match stmt.as_ref() {
                Statement::ImportDeclaration(_) => {
                    if block_start.is_none() {
                        block_start = Some(idx);
                    }
                    block_imports.push(idx);
                }
                Statement::EmptyStatement(_) => {
                    // Skip empty statements
                }
                _ => {
                    // Non-import statement found, finish current block if any
                    if let Some(start) = block_start {
                        let block = self.build_import_block(
                            start,
                            idx,
                            &block_imports,
                            statements,
                            sort_options,
                            f,
                        )?;
                        blocks.push(block);
                        block_start = None;
                        block_imports.clear();
                    }
                }
            }
        }

        // Don't forget the last block if program ends with imports
        if let Some(start) = block_start {
            let block = self.build_import_block(
                start,
                statements.len(),
                &block_imports,
                statements,
                sort_options,
                f,
            )?;
            blocks.push(block);
        }

        Ok(blocks)
    }

    /// Build an import block from import indices
    fn build_import_block(
        &self,
        start_idx: usize,
        end_idx: usize,
        import_indices: &[usize],
        statements: &[&AstNode<'a, Statement<'a>>],
        sort_options: SortImports,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<ImportBlock<'a>> {
        // Format imports to IR with all comments included
        let mut import_irs = Vec::with_capacity(import_indices.len());
        for &idx in import_indices {
            let stmt = statements[idx];
            if let Statement::ImportDeclaration(import_decl) = stmt.as_ref() {
                let mut buffer = VecBuffer::new(f.state_mut());
                write!(&mut buffer, [stmt])?;
                let ir_elements = buffer.into_vec();

                let source = import_decl.source.value.as_str().to_string();
                let is_side_effect = import_decl.specifiers.is_none()
                    || import_decl.specifiers.as_ref().unwrap().is_empty();

                // Detect leading comments in IR (don't split, just mark the boundary)
                let leading_comment_end_idx = self.find_leading_comment_boundary(&ir_elements);

                import_irs.push(ImportIR {
                    index: idx,
                    source,
                    ir_elements,
                    leading_comment_end_idx,
                    is_side_effect,
                });
            }
        }

        // Partition and sort based on IR structure
        let groups =
            self.partition_imports_by_ir_structure(import_irs, sort_options, statements, f);
        let sorted_groups = self.sort_import_groups(groups, sort_options);

        Ok(ImportBlock { start_idx, end_idx, groups: sorted_groups })
    }

    /// Find the boundary between leading comments and the actual import statement
    /// Returns the index where the actual import starts (after leading comments)
    fn find_leading_comment_boundary(&self, ir_elements: &[FormatElement<'a>]) -> usize {
        let mut tag_depth = 0;
        let mut last_comment_or_line_idx = 0;

        for (idx, element) in ir_elements.iter().enumerate() {
            match element {
                FormatElement::Tag(tag) if tag.is_start() => {
                    tag_depth += 1;
                }
                FormatElement::Tag(tag) if tag.is_end() => {
                    tag_depth -= 1;
                }
                FormatElement::Line(_) => {
                    if tag_depth == 0 {
                        last_comment_or_line_idx = idx + 1;
                    }
                }
                FormatElement::DynamicText { text } | FormatElement::StaticText { text } => {
                    // Check if this looks like a comment
                    let trimmed = text.trim_start();
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                        // This is a comment, continue
                        if tag_depth == 0 {
                            last_comment_or_line_idx = idx + 1;
                        }
                    } else if tag_depth == 0 {
                        // Non-comment text at depth 0 means the import body has started
                        return last_comment_or_line_idx;
                    }
                }
                FormatElement::LocatedTokenText { .. } => {
                    if tag_depth == 0 {
                        // Start of actual import statement
                        return last_comment_or_line_idx;
                    }
                }
                _ => {}
            }
        }

        last_comment_or_line_idx
    }

    /// Partition imports into groups based on IR structure (comments) and source text (empty lines)
    fn partition_imports_by_ir_structure(
        &self,
        mut import_irs: Vec<ImportIR<'a>>,
        sort_options: SortImports,
        statements: &[&AstNode<'a, Statement<'a>>],
        f: &Formatter<'_, 'a>,
    ) -> Vec<ImportGroupIR<'a>> {
        if import_irs.is_empty() {
            return vec![];
        }

        let source_text = f.source_text();
        let all_comments = f.comments().all_comments();
        let mut groups = vec![];
        let mut current_group = ImportGroupIR::default();
        let mut prev_import_end: Option<u32> = None;

        for mut import_ir in import_irs {
            let stmt = statements[import_ir.index];
            let import_span = stmt.span();

            // Get leading comment IR slice
            let leading_comment_end_idx = import_ir.leading_comment_end_idx;
            let leading_comment_slice = &import_ir.ir_elements[..leading_comment_end_idx];

            // Check for separator comments from IR
            let has_separator_comment = if sort_options.partition_by_comment {
                self.has_own_line_comment_in_ir(leading_comment_slice)
            } else {
                false
            };

            // Check for empty lines from source text
            let has_empty_line = if sort_options.partition_by_newline
                && let Some(prev_end) = prev_import_end
            {
                self.has_empty_line_between(&source_text, all_comments, prev_end, import_span.start)
            } else {
                false
            };

            let should_partition = has_separator_comment || has_empty_line;

            if should_partition && !current_group.imports.is_empty() {
                // Save the current group and start a new one
                groups.push(std::mem::take(&mut current_group));

                if has_separator_comment {
                    // New group starts with these separator comments
                    current_group.separator_comment_irs = leading_comment_slice.to_vec();

                    // Check for empty lines around separator comments from source text
                    if sort_options.partition_by_newline
                        && let Some(prev_end) = prev_import_end
                    {
                        let (has_nl_before, has_nl_after) = self
                            .check_newlines_around_separator_from_source(
                                &source_text,
                                all_comments,
                                prev_end,
                                import_span.start,
                            );
                        current_group.has_newline_before_comments = has_nl_before;
                        current_group.has_newline_after_comments = has_nl_after;
                    }
                    // Leading comments were used as separator - skip them in import output
                    // (keep leading_comment_end_idx as is)
                } else {
                    // Empty line partition without separator comment
                    // Leading comments are NOT separators, keep them with the import
                    import_ir.leading_comment_end_idx = 0;
                }
            } else {
                // Leading comments are NOT separators, keep them with the import
                import_ir.leading_comment_end_idx = 0;
            }

            // Add this import to current group
            current_group.imports.push(import_ir);
            prev_import_end = Some(import_span.end);
        }

        // Don't forget to push the last group
        if !current_group.imports.is_empty() {
            groups.push(current_group);
        }

        groups
    }

    /// Check for empty lines around separator comments from source text
    fn check_newlines_around_separator_from_source(
        &self,
        source_text: &SourceText,
        all_comments: &[Comment],
        prev_import_end: u32,
        next_import_start: u32,
    ) -> (bool, bool) {
        // Binary search to find the first comment that could be in range
        let start_idx = all_comments
            .binary_search_by(|c| {
                if c.span.end < prev_import_end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|idx| idx);

        // Find own-line comments between the two imports
        let mut separator_comments = all_comments[start_idx..]
            .iter()
            .take_while(|c| c.span.start < next_import_start)
            .filter(|c| c.span.end <= next_import_start && self.is_own_line_comment(c, source_text))
            .map(|c| c.span);

        let Some(first_comment) = separator_comments.next() else {
            return (false, false);
        };

        let last_comment = separator_comments.last().unwrap_or(first_comment);

        let newline_before =
            self.count_newlines_between(source_text, prev_import_end, first_comment.start) >= 2;
        let newline_after =
            self.count_newlines_between(source_text, last_comment.end, next_import_start) >= 2;

        (newline_before, newline_after)
    }

    /// Checks if there's an empty line between two positions
    fn has_empty_line_between(
        &self,
        source_text: &SourceText,
        all_comments: &[Comment],
        prev_end: u32,
        next_start: u32,
    ) -> bool {
        if !source_text.contains_newline_between(prev_end, next_start) {
            return false;
        }

        // Exclude leading comments of the next import from empty line detection
        let next_import_leading_comments =
            self.find_adjacent_leading_comments(all_comments, source_text, next_start);

        let check_end = if let Some(first_leading) = next_import_leading_comments.last() {
            first_leading.span.start
        } else {
            next_start
        };

        self.count_newlines_between(source_text, prev_end, check_end) >= 2
    }

    /// Check if IR contains own-line comments
    fn has_own_line_comment_in_ir(&self, ir_elements: &[FormatElement<'a>]) -> bool {
        // Look for patterns like: DynamicText/StaticText followed by Line
        for window in ir_elements.windows(2) {
            if let (
                FormatElement::DynamicText { text } | FormatElement::StaticText { text },
                FormatElement::Line(_),
            ) = (&window[0], &window[1])
            {
                // Check if this looks like a comment
                if text.trim_start().starts_with("//") || text.trim_start().starts_with("/*") {
                    return true;
                }
            }
        }
        false
    }

    /// Sort import groups
    fn sort_import_groups(
        &self,
        mut groups: Vec<ImportGroupIR<'a>>,
        sort_options: SortImports,
    ) -> Vec<ImportGroupIR<'a>> {
        for group in &mut groups {
            // Separate ignored (side-effect) imports from sortable imports
            let sortable_len = group.imports.len();
            let mut ignored = Vec::with_capacity(sortable_len);
            let mut to_sort = Vec::with_capacity(sortable_len);

            for (original_pos, import_ir) in
                std::mem::take(&mut group.imports).into_iter().enumerate()
            {
                let is_ignored = !sort_options.sort_side_effects && import_ir.is_side_effect;
                if is_ignored {
                    ignored.push((original_pos, import_ir));
                } else {
                    to_sort.push(import_ir);
                }
            }

            // Sort non-ignored imports
            to_sort.sort_by(|a, b| {
                let ord = if sort_options.ignore_case {
                    use cow_utils::CowUtils;
                    a.source.cow_to_lowercase().cmp(&b.source.cow_to_lowercase())
                } else {
                    a.source.cmp(&b.source)
                };
                if sort_options.order.is_desc() { ord.reverse() } else { ord }
            });

            // Merge back: place ignored imports at original positions
            let total_len = ignored.len() + to_sort.len();
            let mut result = Vec::with_capacity(total_len);
            let mut to_sort_iter = to_sort.into_iter();
            let mut ignored_iter = ignored.into_iter();
            let mut next_ignored = ignored_iter.next();

            for original_pos in 0..total_len {
                if let Some((pos, _)) = next_ignored
                    && pos == original_pos
                {
                    if let Some((_, import_ir)) = next_ignored.take() {
                        result.push(import_ir);
                        next_ignored = ignored_iter.next();
                    }
                    continue;
                }
                if let Some(import_ir) = to_sort_iter.next() {
                    result.push(import_ir);
                }
            }

            group.imports = result;
        }

        groups
    }

    /// Output a sorted import block
    fn output_import_block(
        &self,
        block: &ImportBlock<'a>,
        _statements: &[&AstNode<'a, Statement<'a>>],
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        let sort_options = f.options().experimental_sort_imports.unwrap();
        let mut is_first_import = true;

        for (group_idx, group) in block.groups.iter().enumerate() {
            // Output separator comments before this group
            if !group.separator_comment_irs.is_empty() {
                // Add empty line before separator comments if needed
                if group_idx > 0
                    && sort_options.partition_by_newline
                    && group.has_newline_before_comments
                {
                    write!(f, [empty_line()])?;
                }

                // Output separator comment IR elements
                if !is_first_import {
                    write!(f, [hard_line_break()])?;
                }
                crate::formatter::buffer::BufferExtensions::write_elements(
                    f,
                    group.separator_comment_irs.iter().cloned(),
                )?;
                is_first_import = false;

                // Add empty line after separator comments if needed
                if sort_options.partition_by_newline
                    && group.has_newline_after_comments
                    && !group.imports.is_empty()
                {
                    write!(f, [empty_line()])?;
                }
            } else if group_idx > 0 {
                // No separator comments, just add empty line between groups
                write!(f, [empty_line()])?;
            }

            // Output each import in this group (skip leading comments, they're in separator_comment_irs)
            for import_ir in &group.imports {
                if !is_first_import {
                    write!(f, [hard_line_break()])?;
                }
                is_first_import = false;

                // Output only the import body (skip leading comments)
                let import_body_slice = &import_ir.ir_elements[import_ir.leading_comment_end_idx..];
                crate::formatter::buffer::BufferExtensions::write_elements(
                    f,
                    import_body_slice.iter().cloned(),
                )?;
            }
        }

        // Add line break after the import block
        if !block.groups.is_empty() {
            write!(f, [hard_line_break()])?;
        }

        Ok(())
    }

    fn get_statement_span(&self, stmt: &AstNode<'a, Statement<'a>>) -> Span {
        match stmt.as_ref() {
            // `@decorator export class A {}`
            // Get the span of the decorator.
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::ClassDeclaration(decl)) = &export.declaration
                    && let Some(decorator) = decl.decorators.first()
                    && decorator.span().start < export.span.start
                {
                    decorator.span()
                } else {
                    export.span
                }
            }
            // `@decorator export default class A {}`
            // Get the span of the decorator.
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::ClassDeclaration(decl) = &export.declaration
                    && let Some(decorator) = decl.decorators.first()
                    && decorator.span().start < export.span.start
                {
                    decorator.span()
                } else {
                    export.span
                }
            }
            _ => stmt.span(),
        }
    }
}

/// A block of consecutive import statements
struct ImportBlock<'a> {
    /// Index of first statement in this block
    start_idx: usize,
    /// Index after last statement in this block
    end_idx: usize,
    /// Sorted import groups within this block
    groups: Vec<ImportGroupIR<'a>>,
}

/// Import formatted to IR
struct ImportIR<'a> {
    /// Original index in statement list
    index: usize,
    /// Import source for sorting
    source: String,
    /// Formatted IR elements (including leading comments)
    ir_elements: Vec<FormatElement<'a>>,
    /// Index where actual import starts (after leading comments)
    leading_comment_end_idx: usize,
    /// Whether this is a side-effect import
    is_side_effect: bool,
}

/// Group of imports within a block (separated by comments/newlines)
#[derive(Default)]
struct ImportGroupIR<'a> {
    /// Imports in this group
    imports: Vec<ImportIR<'a>>,
    /// Separator comment IR elements before this group
    separator_comment_irs: Vec<FormatElement<'a>>,
    /// Whether there's an empty line before separator comments
    has_newline_before_comments: bool,
    /// Whether there's an empty line after separator comments
    has_newline_after_comments: bool,
}

impl<'a> Format<'a> for AstNode<'a, ArenaVec<'a, Directive<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some(last_directive) = self.last() else {
            // No directives, no extra new line
            return Ok(());
        };

        f.join_nodes_with_hardline().entries(self).finish()?;

        // if next_sibling's first leading_trivia has more than one new_line, we should add an extra empty line at the end of
        // JsDirectiveList, for example:
        //```js
        // "use strict"; <- first leading new_line
        //  			 <- second leading new_line
        // function foo() {

        // }
        //```
        // so we should keep an extra empty line after JsDirectiveList

        let end = if let Some(last_printed_comment) = f.comments().printed_comments().last()
            && last_printed_comment.span.end > last_directive.span.end
        {
            last_printed_comment.span.end
        } else {
            last_directive.span.end
        };

        let need_extra_empty_line = f.source_text().lines_after(end) > 1;
        write!(f, if need_extra_empty_line { empty_line() } else { hard_line_break() })
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Directive<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                FormatLiteralStringToken::new(
                    f.source_text().text_for(self.expression()),
                    self.expression().span(),
                    /* jsx */
                    false,
                    StringLiteralParentKind::Directive,
                ),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Hashbang<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#!", dynamic_text(self.value().as_str().trim_end())])?;

        if f.source_text().lines_after(self.span.end) > 1 {
            write!(f, [empty_line()])
        } else {
            write!(f, [hard_line_break()])
        }
    }
}
