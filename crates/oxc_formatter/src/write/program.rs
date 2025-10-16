use std::ops::Deref;

use cow_utils::CowUtils;
use rustc_hash::{FxBuildHasher, FxHashSet};

use oxc_allocator::{Address, Vec};
use oxc_ast::{Comment, ast::*, match_expression};
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::{ZWNBSP, is_line_terminator};

use crate::{
    Buffer, Format, FormatResult, FormatTrailingCommas, TrailingSeparator, format_args,
    formatter::{SourceText, prelude::*, trivia::FormatTrailingComments},
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

struct FormatProgramBody<'a, 'b>(&'b AstNode<'a, Vec<'a, Statement<'a>>>);

impl<'a> Deref for FormatProgramBody<'a, '_> {
    type Target = AstNode<'a, Vec<'a, Statement<'a>>>;
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
        let source_text = f.source_text();
        let all_comments = f.comments().all_comments();

        // Collect import statements and their indices
        let mut import_groups: std::vec::Vec<ImportGroup> = std::vec::Vec::new();
        let mut current_group = ImportGroup::new();
        let mut last_import_end: Option<u32> = None;

        // Track separator comments to exclude them from leading comments
        let mut separator_comment_spans: CommentSpanSet = CommentSpanSet::default();
        // Track all import-related comments (will grow dynamically)
        let mut all_import_comment_spans: CommentSpanSet = CommentSpanSet::default();

        for (idx, stmt) in self.iter().enumerate() {
            match stmt.as_ref() {
                Statement::ImportDeclaration(import_decl) => {
                    // Check if we should start a new group
                    let partition_info = if let Some(prev_end) = last_import_end {
                        self.check_partition(
                            sort_options,
                            &source_text,
                            all_comments,
                            prev_end,
                            import_decl.span.start,
                        )
                    } else {
                        PartitionInfo::no_partition()
                    };

                    if partition_info.should_partition && !current_group.imports.is_empty() {
                        import_groups.push(std::mem::take(&mut current_group));

                        // Store separator comments and newline info in the new group
                        // Move instead of clone since PartitionInfo is temporary
                        current_group.separator_comment_spans = partition_info.separator_comments;
                        current_group.has_newline_before_comments =
                            partition_info.newline_before_comments;
                        current_group.has_newline_after_comments =
                            partition_info.newline_after_comments;

                        // Track separator comment spans
                        for span in &current_group.separator_comment_spans {
                            separator_comment_spans.insert(*span);
                        }
                    }

                    current_group.imports.push(idx);
                    last_import_end = Some(import_decl.span.end);
                }
                Statement::EmptyStatement(_) => {
                    // Skip empty statements
                }
                _ => {
                    // Non-import statement - finalize current group
                    if !current_group.imports.is_empty() {
                        import_groups.push(std::mem::take(&mut current_group));
                    }
                    break;
                }
            }
        }
        // Don't forget the last group
        if !current_group.imports.is_empty() {
            import_groups.push(current_group);
        }

        // If no imports to sort, use default formatting
        if import_groups.is_empty() {
            return self.fmt_default(f);
        }

        // Sort each group and collect sorted imports with their comments
        let mut sorted_groups: std::vec::Vec<std::vec::Vec<ImportWithComments>> =
            std::vec::Vec::with_capacity(import_groups.len());
        for group in &mut import_groups {
            let sorted = group.sort_with_comments(
                self,
                sort_options,
                &source_text,
                all_comments,
                &separator_comment_spans,
            );
            sorted_groups.push(sorted);
        }

        // Track all import-related comments for view_limit and printed_count
        for group in &sorted_groups {
            for import_with_comments in group {
                // Track leading comments
                for span in &import_with_comments.leading_comment_spans {
                    all_import_comment_spans.insert(*span);
                }
                // Track trailing comments
                if let Some(span) = import_with_comments.trailing_comment_span {
                    all_import_comment_spans.insert(span);
                }
            }
        }
        // Also track separator comments
        for group in &import_groups {
            for span in &group.separator_comment_spans {
                all_import_comment_spans.insert(*span);
            }
        }

        // Release the borrow on f
        let _ = all_comments;

        // Find the span range covering all imports (for view_limit)
        let first_import_idx = import_groups[0].imports[0];
        let last_group = &import_groups[import_groups.len() - 1];
        let last_import_idx = last_group.imports[last_group.imports.len() - 1];

        let first_import_start = if let Some(stmt) = self.iter().nth(first_import_idx) {
            if let Statement::ImportDeclaration(import) = stmt.as_ref() {
                // Find the earliest comment or import span
                let mut start = import.span.start;
                for comment_span in &all_import_comment_spans {
                    if comment_span.start < start {
                        start = comment_span.start;
                    }
                }
                start
            } else {
                0
            }
        } else {
            0
        };

        let last_import_end = if let Some(stmt) = self.iter().nth(last_import_idx) {
            let import_end = stmt.span().end;
            // Check if the last import has a trailing comment
            let mut end = import_end;
            for import_with_comments in sorted_groups.iter().flat_map(|g| g.iter()) {
                if import_with_comments.index == last_import_idx {
                    if let Some(trailing_span) = import_with_comments.trailing_comment_span {
                        end = trailing_span.end;
                    }
                    break;
                }
            }
            end
        } else {
            u32::MAX
        };

        // Find the index in the comments array where our import-related comments end
        // This will be used to set view_limit
        let mut last_import_comment_idx = None;
        for (idx, comment) in f.comments().all_comments().iter().enumerate() {
            if all_import_comment_spans.contains(&comment.span) {
                last_import_comment_idx = Some(idx);
            }
        }

        // Set view_limit to hide all import-related comments from automatic processing
        let original_limit = if last_import_comment_idx.is_some() {
            let current_printed = f.comments().printed_count();
            f.context_mut().comments_mut().view_limit.replace(current_printed)
        } else {
            f.context_mut().comments_mut().view_limit
        };

        // Track which comments we've manually output to skip them later (preallocate)
        let mut manually_output_comments =
            CommentSpanSet::with_capacity_and_hasher(all_import_comment_spans.len(), FxBuildHasher);

        // Build import_idx -> group_idx mapping for O(1) lookup
        // Use Vec since import indices are sequential (0, 1, 2, ...)
        let max_import_idx =
            import_groups.iter().flat_map(|g| g.imports.iter()).max().copied().unwrap_or(0);
        let mut import_to_group: std::vec::Vec<Option<usize>> = vec![None; max_import_idx + 1];
        for (g_idx, group) in import_groups.iter().enumerate() {
            for &import_idx in &group.imports {
                import_to_group[import_idx] = Some(g_idx);
            }
        }

        // Track which import group we've already output (use Vec for small sizes)
        let mut output_import_groups = std::vec::Vec::with_capacity(import_groups.len());
        let mut is_first_statement = true;

        // Single pass: collect statements and find first non-import in one iteration
        let mut all_stmts = std::vec::Vec::new();
        let mut first_non_import_idx = None;

        for (stmt_idx, stmt) in self.iter().enumerate() {
            if matches!(stmt.as_ref(), Statement::EmptyStatement(_)) {
                continue;
            }

            let is_import = stmt_idx < import_to_group.len() && import_to_group[stmt_idx].is_some();
            let current_idx = all_stmts.len();

            if !is_import && first_non_import_idx.is_none() {
                first_non_import_idx = Some(current_idx);
            }

            all_stmts.push((stmt_idx, stmt));
        }

        // Output imports
        for (i, (stmt_idx, stmt)) in all_stmts.iter().enumerate() {
            // Find which group this import belongs to (O(1) lookup)
            let is_import_in_group =
                if *stmt_idx < import_to_group.len() { import_to_group[*stmt_idx] } else { None };

            if let Some(g_idx) = is_import_in_group {
                // If this is the first import from this group, output all sorted imports
                if !output_import_groups.contains(&g_idx) {
                    output_import_groups.push(g_idx);
                    // Add spacing/separator before this group
                    if !import_groups[g_idx].separator_comment_spans.is_empty() {
                        // This group has separator comments
                        if g_idx == 0 && !is_first_statement {
                            write!(f, [hard_line_break()])?;
                        }

                        // Add empty line before comments if needed
                        if import_groups[g_idx].has_newline_before_comments {
                            write!(f, [empty_line()])?;
                        }

                        // Output separator comments (read from source_text) and mark as manually output
                        for separator_span in &import_groups[g_idx].separator_comment_spans {
                            let comment_text = source_text.text_for(separator_span);
                            write!(f, [dynamic_text(comment_text), hard_line_break()])?;
                            manually_output_comments.insert(*separator_span);
                        }

                        // Add empty line after comments if needed
                        if import_groups[g_idx].has_newline_after_comments {
                            write!(f, [empty_line()])?;
                        }
                    } else if g_idx > 0 {
                        // This group was separated by newlines only (no comments)
                        write!(f, [empty_line()])?;
                    } else if !is_first_statement {
                        write!(f, [hard_line_break()])?;
                    }

                    // Output all imports in this group in sorted order with their comments
                    for (imp_idx, import_with_comments) in sorted_groups[g_idx].iter().enumerate() {
                        let import_idx = import_with_comments.index;
                        let import_stmt = self.iter().nth(import_idx).unwrap();

                        // Output leading comments (read from source_text) and mark as manually output
                        for comment_span in &import_with_comments.leading_comment_spans {
                            let comment_text = source_text.text_for(comment_span);
                            write!(f, [dynamic_text(comment_text), hard_line_break()])?;
                            manually_output_comments.insert(*comment_span);
                        }

                        // Output the import statement itself (without formatter processing comments)
                        write!(f, [import_stmt])?;

                        // Output trailing inline comment if exists and mark as manually output
                        if let Some(trailing_span) = import_with_comments.trailing_comment_span {
                            let comment_text = source_text.text_for(&trailing_span);
                            write!(f, [dynamic_text(" "), dynamic_text(comment_text)])?;
                            manually_output_comments.insert(trailing_span);
                        }

                        write!(f, [hard_line_break()])?;

                        is_first_statement = false;
                    }
                }
                // Skip this import as it's already been output
            } else if first_non_import_idx.is_some() && i == first_non_import_idx.unwrap() {
                // CRITICAL: Restore view_limit BEFORE processing non-import statements
                // This prevents invariant violation (printed_count > view_limit) when
                // formatting non-import statements that may call unprinted_comments()

                // Mark all manually output comments as printed
                let mut last_import_region_comment_idx = None;
                for (idx, comment) in f.comments().all_comments().iter().enumerate() {
                    if comment.span.start < last_import_end
                        && manually_output_comments.contains(&comment.span)
                    {
                        last_import_region_comment_idx = Some(idx);
                    } else if comment.span.start >= last_import_end {
                        break;
                    }
                }

                if let Some(last_idx) = last_import_region_comment_idx {
                    let target_printed_count = last_idx + 1;
                    let current_printed_count = f.comments().printed_count();
                    if target_printed_count > current_printed_count {
                        // Temporarily remove view_limit to maintain invariant
                        f.context_mut().comments_mut().view_limit = None;
                        f.context_mut().comments_mut().increase_printed_count_by(
                            target_printed_count - current_printed_count,
                        );
                    }
                }

                // Restore view_limit with invariant check
                let current_printed = f.comments().printed_count();
                match original_limit {
                    Some(limit) if limit >= current_printed => {
                        f.context_mut().comments_mut().view_limit = Some(limit);
                    }
                    _ => {
                        f.context_mut().comments_mut().view_limit = None;
                    }
                }

                // This is the first non-import statement
                // Add spacing after imports
                if !is_first_statement {
                    let lines_after_imports = source_text.lines_after(last_import_end);
                    if lines_after_imports > 1 {
                        write!(f, [empty_line()])?;
                    } else {
                        write!(f, [hard_line_break()])?;
                    }
                }

                // Now use join_nodes_with_hardline for all remaining non-import statements
                let mut join = f.join_nodes_with_hardline();
                for (j, (_, non_import_stmt)) in all_stmts[i..].iter().enumerate() {
                    let span = self.get_statement_span(non_import_stmt);
                    join.entry(span, non_import_stmt);
                }
                join.finish()?;
                break;
            }
        }

        Ok(())
    }

    /// Checks if we should partition import groups based on the content between two imports
    fn check_partition(
        &self,
        sort_options: SortImports,
        source_text: &SourceText,
        all_comments: &[Comment],
        prev_end: u32,
        next_start: u32,
    ) -> PartitionInfo {
        // Step 1: Check for partition_by_comment (takes precedence)
        if sort_options.partition_by_comment {
            let separator_comments =
                self.find_separator_comments(all_comments, source_text, prev_end, next_start);

            if !separator_comments.is_empty() {
                // Found separator comments - also check for newlines around them
                let (nl_before, nl_after) = if sort_options.partition_by_newline {
                    self.check_newlines_around_comments(
                        source_text,
                        &separator_comments,
                        prev_end,
                        next_start,
                    )
                } else {
                    (false, false)
                };

                return PartitionInfo::with_separator_comments(
                    separator_comments,
                    nl_before,
                    nl_after,
                );
            }
        }

        // Step 2: Check for partition_by_newline (if no separator comments found)
        if sort_options.partition_by_newline
            && self.has_empty_line_between(source_text, all_comments, prev_end, next_start)
        {
            return PartitionInfo::with_newline();
        }

        PartitionInfo::no_partition()
    }

    /// Finds separator comments between two positions
    fn find_separator_comments(
        &self,
        all_comments: &[Comment],
        source_text: &SourceText,
        start: u32,
        end: u32,
    ) -> std::vec::Vec<Span> {
        // Binary search to find the first comment that could be in range
        let start_idx = all_comments
            .binary_search_by(|c| {
                if c.span.end < start {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|idx| idx);

        // Collect comments in range
        all_comments[start_idx..]
            .iter()
            .take_while(|c| c.span.start < end)
            .filter(|c| c.span.end <= end && self.is_own_line_comment(c, source_text))
            .map(|c| c.span)
            .collect()
    }

    /// Common helper: Counts newlines (line terminators) between two positions
    #[inline]
    fn count_newlines_between(&self, source_text: &SourceText, start: u32, end: u32) -> usize {
        source_text.slice_range(start, end).chars().filter(|&c| is_line_terminator(c)).count()
    }

    /// Checks for newlines before and after separator comments
    fn check_newlines_around_comments(
        &self,
        source_text: &SourceText,
        separator_comments: &[Span],
        prev_end: u32,
        next_start: u32,
    ) -> (bool, bool) {
        let first_comment_start = separator_comments[0].start;
        let last_comment_end = separator_comments.last().unwrap().end;

        let newline_before =
            self.count_newlines_between(source_text, prev_end, first_comment_start) >= 2;
        let newline_after =
            self.count_newlines_between(source_text, last_comment_end, next_start) >= 2;

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

    /// Common helper: Finds adjacent leading comments by working backwards from a position.
    /// Returns comments that are only separated by whitespace, in reverse order (most recent first).
    #[inline]
    fn find_adjacent_comments_backwards<'c>(
        &self,
        candidates: &[&'c Comment],
        source_text: &SourceText,
        start_pos: u32,
    ) -> std::vec::Vec<&'c Comment> {
        let mut leading = std::vec::Vec::new();
        let mut search_pos = start_pos;

        // Iterate in reverse without collecting
        for &comment in candidates.iter().rev() {
            let text_between = source_text.slice_range(comment.span.end, search_pos);
            if text_between.chars().all(char::is_whitespace) {
                leading.push(comment);
                search_pos = comment.span.start;
            } else {
                break;
            }
        }
        leading
    }

    /// Finds adjacent leading comments for an import statement
    fn find_adjacent_leading_comments(
        &self,
        all_comments: &[Comment],
        source_text: &SourceText,
        import_start: u32,
    ) -> std::vec::Vec<Comment> {
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

        // Filter and collect own-line comments
        let candidates: std::vec::Vec<&Comment> = all_comments[..end_idx]
            .iter()
            .filter(|c| self.is_own_line_comment(c, source_text))
            .collect();

        self.find_adjacent_comments_backwards(&candidates, source_text, import_start)
            .into_iter()
            .copied()
            .collect()
    }

    /// Gets leading comment spans for an import (excluding separator comments)
    #[inline]
    fn get_leading_comment_spans_for_import(
        &self,
        import_span: Span,
        all_comments: &[Comment],
        source_text: &SourceText,
        separator_comment_spans: &CommentSpanSet,
    ) -> std::vec::Vec<Span> {
        // Binary search to find comments before import_span.start
        let end_idx = all_comments
            .binary_search_by(|c| {
                if c.span.end <= import_span.start {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|idx| idx);

        // Filter own-line comments that are not separator comments
        let candidates: std::vec::Vec<&Comment> = all_comments[..end_idx]
            .iter()
            .filter(|c| {
                !separator_comment_spans.contains(&c.span)
                    && self.is_own_line_comment(c, source_text)
            })
            .collect();

        // Use common helper to find adjacent comments, then extract spans
        let mut leading_comment_spans: std::vec::Vec<Span> = self
            .find_adjacent_comments_backwards(&candidates, source_text, import_span.start)
            .into_iter()
            .map(|c| c.span)
            .collect();

        // Reverse to get source order
        leading_comment_spans.reverse();
        leading_comment_spans
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

    /// Gets trailing inline comment span for an import
    #[inline]
    fn get_trailing_comment_span_for_import(
        &self,
        import_span: Span,
        all_comments: &[Comment],
        source_text: &SourceText,
    ) -> Option<Span> {
        // Binary search to find the first comment at or after import_span.end
        let start_idx = all_comments
            .binary_search_by_key(&import_span.end, |c| c.span.start)
            .unwrap_or_else(|idx| idx);

        // Check if the first candidate is on the same line
        all_comments
            .get(start_idx)
            .filter(|c| !source_text.contains_newline_between(import_span.end, c.span.start))
            .map(|c| c.span)
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

type CommentSpanSet = FxHashSet<Span>;

/// Information about how to partition an import group
struct PartitionInfo {
    should_partition: bool,
    separator_comments: std::vec::Vec<Span>,
    newline_before_comments: bool,
    newline_after_comments: bool,
}

impl PartitionInfo {
    fn no_partition() -> Self {
        Self {
            should_partition: false,
            separator_comments: std::vec::Vec::new(),
            newline_before_comments: false,
            newline_after_comments: false,
        }
    }

    fn with_separator_comments(
        comments: std::vec::Vec<Span>,
        newline_before: bool,
        newline_after: bool,
    ) -> Self {
        Self {
            should_partition: true,
            separator_comments: comments,
            newline_before_comments: newline_before,
            newline_after_comments: newline_after,
        }
    }

    fn with_newline() -> Self {
        Self {
            should_partition: true,
            separator_comments: std::vec::Vec::new(),
            newline_before_comments: false,
            newline_after_comments: false,
        }
    }
}

/// Group of import statements that should be sorted together
#[derive(Debug, Default)]
struct ImportGroup {
    /// Original indices of import statements in the program body
    imports: std::vec::Vec<usize>,
    /// Separator comment spans that appear before this group (if partition_by_comment is enabled)
    separator_comment_spans: std::vec::Vec<Span>,
    /// Whether there's an empty line before the separator comments
    has_newline_before_comments: bool,
    /// Whether there's an empty line after the separator comments
    has_newline_after_comments: bool,
}

/// Information about an import and its associated comments
#[derive(Debug, Clone)]
struct ImportWithComments {
    /// Index of the import in the program body
    index: usize,
    /// Source string for sorting
    source: String,
    /// Leading comment spans (to be read from source_text during output)
    leading_comment_spans: std::vec::Vec<Span>,
    /// Trailing comment span (inline comment at the end of the line)
    trailing_comment_span: Option<Span>,
    /// Whether this is a side-effect import (no specifiers)
    is_side_effect: bool,
    /// Whether this import should be ignored during sorting (keep original position)
    is_ignored: bool,
}

impl ImportGroup {
    fn new() -> Self {
        Self::default()
    }

    fn sort_with_comments(
        &mut self,
        program_body: &FormatProgramBody<'_, '_>,
        options: SortImports,
        source_text: &SourceText,
        all_comments: &[Comment],
        separator_comment_spans: &CommentSpanSet,
    ) -> std::vec::Vec<ImportWithComments> {
        if self.imports.is_empty() {
            return std::vec::Vec::new();
        }

        // Even for single import, we need to extract comments, so no early return

        // Create sortable import entries with comment info
        let mut sortable: std::vec::Vec<ImportWithComments> = self
            .imports
            .iter()
            .filter_map(|&idx| {
                let stmt = program_body.iter().nth(idx)?;
                match stmt.as_ref() {
                    Statement::ImportDeclaration(import) => {
                        let import = import.as_ref();
                        let source = import.source.value.as_str().to_string();

                        // Get leading comment spans for this import
                        let leading_comment_spans = program_body
                            .get_leading_comment_spans_for_import(
                                import.span,
                                all_comments,
                                source_text,
                                separator_comment_spans,
                            );

                        // Get trailing inline comment span for this import
                        let trailing_comment_span = program_body
                            .get_trailing_comment_span_for_import(
                                import.span,
                                all_comments,
                                source_text,
                            );

                        // Check if this is a side-effect import (no specifiers)
                        let is_side_effect = import.specifiers.is_none()
                            || import.specifiers.as_ref().unwrap().is_empty();

                        // Determine if this import should be ignored during sorting
                        let is_ignored = !options.sort_side_effects && is_side_effect;

                        Some(ImportWithComments {
                            index: idx,
                            source,
                            leading_comment_spans,
                            trailing_comment_span,
                            is_side_effect,
                            is_ignored,
                        })
                    }
                    _ => None,
                }
            })
            .collect();

        // Sort by import source, respecting is_ignored flag
        // Avoid clones by using in-place sorting and moving values

        // Partition: extract ignored imports (with their original positions)
        // Preallocate capacity for worst case (all ignored or all to_sort)
        let sortable_len = sortable.len();
        let mut ignored = std::vec::Vec::with_capacity(sortable_len);
        let mut to_sort = std::vec::Vec::with_capacity(sortable_len);

        for (original_pos, import) in sortable.into_iter().enumerate() {
            if import.is_ignored {
                ignored.push((original_pos, import));
            } else {
                to_sort.push(import);
            }
        }

        // Sort non-ignored imports
        to_sort.sort_by(|a, b| {
            let ord = if options.ignore_case {
                a.source.cow_to_lowercase().cmp(&b.source.cow_to_lowercase())
            } else {
                a.source.cmp(&b.source)
            };

            if options.order.is_desc() { ord.reverse() } else { ord }
        });

        // Merge back: place ignored imports at original positions
        // Ignored imports are already sorted by position, so we can merge efficiently
        let total_len = ignored.len() + to_sort.len();
        let mut result = std::vec::Vec::with_capacity(total_len);
        let mut to_sort_iter = to_sort.into_iter();
        let mut ignored_iter = ignored.into_iter();
        let mut next_ignored = ignored_iter.next();

        for original_pos in 0..total_len {
            // Check if current position has an ignored import
            if let Some((pos, _)) = next_ignored
                && pos == original_pos
            {
                // Consume the ignored import
                if let Some((_, import)) = next_ignored.take() {
                    result.push(import);
                    next_ignored = ignored_iter.next();
                }
                continue;
            }
            // Otherwise, take from sorted imports
            if let Some(import) = to_sort_iter.next() {
                result.push(import);
            }
        }

        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Directive<'a>>> {
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
