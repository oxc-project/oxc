use unicode_width::UnicodeWidthStr;

use std::cmp;

use oxc_allocator::{Allocator, StringBuilder, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{
    IndentWidth,
    ast_nodes::{AstNode, AstNodeIterator, AstNodes},
    format_args,
    formatter::{
        Format, FormatElement, Formatter, TailwindContextEntry, VecBuffer,
        buffer::RemoveSoftLinesBuffer,
        prelude::{document::Document, *},
        printer::Printer,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    utils::{
        call_expression::is_test_each_pattern,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        tailwindcss::{is_tailwind_function_call, write_tailwind_template_element},
    },
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        // Angular `@Component({ template, styles })`
        if try_format_angular_component(self, f) {
            return;
        }
        // styled-jsx: <style jsx>{`...`}</style> or <div css={`...`} />
        if try_format_css_template(self, f) {
            return;
        }
        let template = TemplateLike::TemplateLiteral(self);
        write!(f, template);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        // Format the tag and type arguments
        write!(f, [self.tag(), self.type_arguments()]);

        let quasi = self.quasi();

        let comments = f.context().comments().comments_before(quasi.span.start);
        if !comments.is_empty() {
            write!(
                f,
                [group(&format_args!(
                    soft_line_break_or_space(),
                    FormatLeadingComments::Comments(comments)
                ))]
            );
        }

        write!(f, [line_suffix_boundary()]);

        // Check if this is a Tailwind function call (e.g., tw`flex p-4`)
        // Extract context entry before mutating f
        let tailwind_ctx_to_push = f
            .options()
            .sort_tailwindcss
            .as_ref()
            .filter(|opts| is_tailwind_function_call(&self.tag, opts))
            .map(|opts| TailwindContextEntry::new(opts.preserve_whitespace));

        if let Some(ctx) = tailwind_ctx_to_push {
            f.context_mut().push_tailwind_context(ctx);
        }

        if try_format_embedded_template(self, f) {
        } else if is_test_each_pattern(&self.tag) {
            let template = &EachTemplateTable::from_template(quasi, f);
            // Use table formatting
            write!(f, template);
        } else {
            let template = TemplateLike::TemplateLiteral(quasi);
            write!(f, template);
        }

        if tailwind_ctx_to_push.is_some() {
            f.context_mut().pop_tailwind_context();
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let source = f.source_text().text_for(self);

        // Check if we're in a Tailwind context via the stack
        // (handles JSXAttribute, CallExpression, TaggedTemplateExpression, and nested contexts)
        let tailwind_ctx = f.context().tailwind_context().copied().filter(|_| {
            // No whitespace means only one class, so no need to sort
            source.as_bytes().iter().any(|&b| b.is_ascii_whitespace())
        });

        if let Some(ctx) = tailwind_ctx {
            write_tailwind_template_element(self, ctx, f);
        } else {
            write!(f, text(self.value.raw.as_str()));
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTemplateLiteralType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let template = TemplateLike::TSTemplateLiteralType(self);
        write!(f, template);
    }
}

/// Layout strategy for template element expressions
#[derive(Debug, Clone, Copy)]
enum TemplateElementLayout {
    /// Tries to format the expression on a single line regardless of the print width.
    SingleLine,

    /// Tries to format the expression on a single line but may break the expression if the line otherwise exceeds the print width.
    Fit,
}

/// The indentation derived from a position in the source document. Consists of indentation level and spaces
#[derive(Debug, Copy, Clone, Default)]
pub struct TemplateElementIndention(u32);

impl TemplateElementIndention {
    /// Returns the indentation level
    pub(crate) fn level(self, indent_width: IndentWidth) -> u32 {
        self.0 / u32::from(indent_width.value())
    }

    /// Returns the number of space indents on top of the indent level
    pub(crate) fn align(self, indent_width: IndentWidth) -> u8 {
        let remainder = self.0 % u32::from(indent_width.value());
        remainder.try_into().unwrap_or(u8::MAX)
    }

    /// Computes the indentation after the last new line character.
    pub(crate) fn after_last_new_line(
        text: &str,
        tab_width: u32,
        previous_indention: Self,
    ) -> Self {
        let by_new_line = text.rsplit_once('\n');

        let size = match by_new_line {
            None => previous_indention.0,
            Some((_, after_new_line)) => {
                let mut size: u32 = 0;

                for byte in after_new_line.bytes() {
                    match byte {
                        b'\t' => {
                            // Tabs behave in a way that they are aligned to the nearest
                            // multiple of tab_width:
                            // number of spaces -> added size
                            // 0 -> 4, 1 -> 4, 2 -> 4, 3 -> 4
                            // 4 -> 8, 5 -> 8, 6 -> 8, 7 -> 8 ..
                            // Or in other words, it clips the size to the next multiple of tab width.
                            size = size + tab_width - (size % tab_width);
                        }
                        b' ' => {
                            size += 1;
                        }
                        _ => break,
                    }
                }

                size
            }
        };

        Self(size)
    }
}

/// Unified enum for handling both JS template literals and TS template literal types
pub enum TemplateLike<'a, 'b> {
    TemplateLiteral(&'b AstNode<'a, TemplateLiteral<'a>>),
    TSTemplateLiteralType(&'b AstNode<'a, TSTemplateLiteralType<'a>>),
}

impl<'a> TemplateLike<'a, '_> {
    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, ArenaVec<'a, TemplateElement<'a>>> {
        match self {
            Self::TemplateLiteral(t) => t.quasis(),
            Self::TSTemplateLiteralType(t) => t.quasis(),
        }
    }
}

/// Iterator that yields template expressions without allocation
enum TemplateExpressionIterator<'a> {
    Expression(AstNodeIterator<'a, Expression<'a>>),
    TSType(AstNodeIterator<'a, TSType<'a>>),
}

impl<'a> Iterator for TemplateExpressionIterator<'a> {
    type Item = TemplateExpression<'a, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Expression(iter) => iter.next().map(TemplateExpression::Expression),
            Self::TSType(iter) => iter.next().map(TemplateExpression::TSType),
        }
    }
}

impl<'a> Format<'a> for TemplateLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, "`");

        let quasis = self.quasis();
        let mut indention = TemplateElementIndention::default();

        let mut expression_iterator = match self {
            Self::TemplateLiteral(t) => {
                TemplateExpressionIterator::Expression(t.expressions().iter())
            }
            Self::TSTemplateLiteralType(t) => TemplateExpressionIterator::TSType(t.types().iter()),
        };

        // Check if we're in a Tailwind context - if so, we need to push expression context
        let tailwind_ctx = f.context().tailwind_context().copied();

        // When in Tailwind context with preserve_whitespace false, newlines are collapsed
        let tailwind_collapses_newlines = tailwind_ctx.is_some_and(|ctx| !ctx.preserve_whitespace);

        let quasis_len = quasis.len();

        for (i, quasi) in quasis.iter().enumerate() {
            // If in Tailwind context, push context with quasi position for boundary detection
            if let Some(ctx) = tailwind_ctx {
                let is_first = i == 0;
                let is_last = i == quasis_len - 1;
                f.context_mut().push_tailwind_context(ctx.with_quasi_position(is_first, is_last));
            }

            write!(f, *quasi);

            // Pop quasi position context
            if tailwind_ctx.is_some() {
                f.context_mut().pop_tailwind_context();
            }

            let quasi_text = quasi.value.raw.as_str();

            if let Some(expr) = expression_iterator.next() {
                // Only calculate indention if newlines are NOT being collapsed
                if !tailwind_collapses_newlines {
                    let tab_width = u32::from(f.options().indent_width.value());
                    indention = TemplateElementIndention::after_last_new_line(
                        quasi_text, tab_width, indention,
                    );
                }
                // When Tailwind collapses newlines, treat as if there's no newline
                let after_new_line = !tailwind_collapses_newlines && quasi_text.ends_with('\n');
                let options = FormatTemplateExpressionOptions { indention, after_new_line };

                // If in Tailwind context, push template expression context with quasi whitespace info
                if let Some(ctx) = tailwind_ctx {
                    let quasi_before_has_trailing_ws =
                        quasi_text.ends_with(|c: char| c.is_ascii_whitespace());
                    let quasi_after_has_leading_ws = quasis.get(i + 1).is_some_and(|q| {
                        q.value.raw.as_str().starts_with(|c: char| c.is_ascii_whitespace())
                    });

                    f.context_mut().push_tailwind_context(
                        TailwindContextEntry::template_expression(
                            ctx,
                            quasi_before_has_trailing_ws,
                            quasi_after_has_leading_ws,
                        ),
                    );
                }

                FormatTemplateExpression::new(&expr, options).fmt(f);

                // Pop the template expression context
                if tailwind_ctx.is_some() {
                    f.context_mut().pop_tailwind_context();
                }
            }
        }

        write!(f, "`");
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FormatTemplateExpressionOptions {
    /// The indentation to use for this element
    pub(crate) indention: TemplateElementIndention,

    /// Does the last template chunk (text element) end with a new line?
    pub(crate) after_new_line: bool,
}

pub(super) enum TemplateExpression<'a, 'b> {
    Expression(&'b AstNode<'a, Expression<'a>>),
    TSType(&'b AstNode<'a, TSType<'a>>),
}

impl TemplateExpression<'_, '_> {
    pub fn as_expression(&self) -> Option<&AstNode<'_, Expression<'_>>> {
        match self {
            Self::Expression(e) => Some(e),
            Self::TSType(_) => None,
        }
    }
}

impl GetSpan for TemplateExpression<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::Expression(e) => e.span(),
            Self::TSType(t) => t.span(),
        }
    }
}

pub struct FormatTemplateExpression<'a, 'b> {
    expression: &'b TemplateExpression<'a, 'b>,
    options: FormatTemplateExpressionOptions,
}

impl<'a, 'b> FormatTemplateExpression<'a, 'b> {
    pub fn new(
        expression: &'b TemplateExpression<'a, 'b>,
        options: FormatTemplateExpressionOptions,
    ) -> Self {
        Self { expression, options }
    }
}

impl<'a> Format<'a> for FormatTemplateExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let options = self.options;

        let mut has_comment_in_expression = false;

        // First, format the expression to check if it will break
        // Special handling for array expressions - force flat mode
        let format_expression = format_once(|f| match self.expression {
            TemplateExpression::Expression(e) => {
                let leading_comments = f.context().comments().comments_before(e.span().start);
                FormatLeadingComments::Comments(leading_comments).fmt(f);
                FormatNodeWithoutTrailingComments(e).fmt(f);
                let trailing_comments =
                    f.context().comments().comments_before_character(e.span().start, b'}');
                has_comment_in_expression =
                    !leading_comments.is_empty() || !trailing_comments.is_empty();
                FormatTrailingComments::Comments(trailing_comments).fmt(f);
            }
            TemplateExpression::TSType(t) => write!(f, t),
        });

        // Intern the expression to check if it will break
        let interned_expression = f.intern(&format_expression);

        let layout = if self.expression.has_new_line_in_range(f) {
            TemplateElementLayout::Fit
        } else {
            let will_break = interned_expression.as_ref().is_some_and(FormatElement::will_break);

            // Make sure the expression won't break to prevent reformat issue
            if will_break { TemplateElementLayout::Fit } else { TemplateElementLayout::SingleLine }
        };

        // We don't need to calculate indentation here as it's already tracked in options

        // Format based on layout
        let format_inner = format_with(|f: &mut Formatter<'_, 'a>| match layout {
            TemplateElementLayout::SingleLine => {
                // Remove soft line breaks for single-line layout
                let mut buffer = RemoveSoftLinesBuffer::new(f);
                if let Some(element) = &interned_expression {
                    buffer.write_element(element.clone());
                }
            }
            TemplateElementLayout::Fit => {
                // Determine if we should add indentation based on expression complexity
                let indent = self.expression.as_expression().is_some_and(|e| {
                    has_comment_in_expression
                        || match e.as_ref() {
                            Expression::StaticMemberExpression(_)
                            | Expression::ComputedMemberExpression(_)
                            | Expression::PrivateFieldExpression(_)
                            | Expression::ConditionalExpression(_)
                            | Expression::SequenceExpression(_)
                            | Expression::TSAsExpression(_)
                            | Expression::TSSatisfiesExpression(_)
                            | Expression::BinaryExpression(_)
                            | Expression::LogicalExpression(_)
                            | Expression::Identifier(_) => true,
                            Expression::ChainExpression(chain) => {
                                chain.expression.is_member_expression()
                            }
                            _ => false,
                        }
                });

                match &interned_expression {
                    Some(element) if indent => {
                        write!(
                            f,
                            [soft_block_indent(&format_with(|f| f.write_element(element.clone())))]
                        );
                    }
                    Some(element) => f.write_element(element.clone()),
                    None => (),
                }
            }
        });

        let format_indented = format_with(|f: &mut Formatter<'_, 'a>| {
            if options.after_new_line {
                // Apply dedent_to_root for expressions after newlines
                write!(f, [dedent_to_root(&format_inner)]);
            } else {
                write_with_indention(&format_inner, options.indention, f.options().indent_width, f);
            }
        });

        // Wrap in ${...} with group
        write!(f, [group(&format_args!("${", format_indented, line_suffix_boundary(), "}"))]);
    }
}

impl<'a> TemplateExpression<'a, '_> {
    fn has_new_line_in_range(&self, f: &Formatter<'_, 'a>) -> bool {
        let span = self.span();
        f.source_text().has_newline_before(span.start)
            || f.source_text().has_newline_after(span.end)
            || f.source_text().contains_newline(span)
    }
}

/// Writes `content` with the specified `indention`.
fn write_with_indention<'a, Content>(
    content: &Content,
    indention: TemplateElementIndention,
    indent_width: IndentWidth,
    f: &mut Formatter<'_, 'a>,
) where
    Content: Format<'a>,
{
    let level = indention.level(indent_width);
    let spaces = indention.align(indent_width);

    if level == 0 && spaces == 0 {
        return write!(f, [content]);
    }

    // Adds as many nested `indent` elements until it reaches the desired indention level.
    let format_indented = format_with(|f| {
        for _ in 0..level {
            f.write_element(FormatElement::Tag(Tag::StartIndent));
        }

        write!(f, [content]);

        for _ in 0..level {
            f.write_element(FormatElement::Tag(Tag::EndIndent));
        }
    });

    // Adds any necessary `align` for spaces not covered by indent level.
    let format_aligned = format_with(|f| {
        if spaces == 0 {
            write!(f, [format_indented]);
        } else {
            write!(f, [align(spaces, &format_indented)]);
        }
    });

    write!(f, [dedent_to_root(&format_aligned)]);
}

#[derive(Debug)]
enum EachTemplateElement<'a> {
    /// A significant value in the test each table. It's a row element.
    Column(EachTemplateColumn<'a>),
    /// Indicates the end of the current row.
    LineBreak,
}

/// Row element containing the column information.
#[derive(Debug)]
struct EachTemplateColumn<'a> {
    /// Formatted text of the column.
    text: &'a str,
    /// Formatted text width.
    width: usize,
    /// Indicates the line break in the text.
    will_break: bool,
}

impl<'a> EachTemplateColumn<'a> {
    fn new(text: &'a str, will_break: bool) -> Self {
        let width = text.width();

        Self { text, width, will_break }
    }
}

struct EachTemplateTableBuilder<'a> {
    /// Holds information about the current row.
    current_row: EachTemplateCurrentRow,
    /// Information about all rows.
    rows: Vec<EachTemplateRow>,
    /// Contains the maximum length of each column of all rows.
    columns_width: Vec<usize>,
    /// Elements for formatting.
    elements: Vec<EachTemplateElement<'a>>,
}

impl<'a> EachTemplateTableBuilder<'a> {
    fn new() -> Self {
        Self {
            current_row: EachTemplateCurrentRow::new(),
            rows: Vec::new(),
            columns_width: Vec::new(),
            elements: Vec::new(),
        }
    }

    fn entry(&mut self, element: EachTemplateElement<'a>) {
        match &element {
            EachTemplateElement::Column(column) => {
                if column.will_break {
                    self.current_row.has_line_break_column = true;
                }

                if !self.current_row.has_line_break_column {
                    self.current_row.column_widths.push(column.width);
                }
            }
            EachTemplateElement::LineBreak => {
                self.next_row();
            }
        }
        self.elements.push(element);
    }

    /// Advance the table state to a new row.
    /// Merge the current row columns width with the table ones if row doesn't contain a line break column.
    fn next_row(&mut self) {
        if !self.current_row.has_line_break_column {
            let table_column_width_iter = self.columns_width.iter_mut();
            let mut row_column_width_iter = self.current_row.column_widths.iter();

            for table_column_width in table_column_width_iter {
                let Some(row_column_width) = row_column_width_iter.next() else { break };
                *table_column_width = cmp::max(*table_column_width, *row_column_width);
            }

            self.columns_width.extend(row_column_width_iter);
        }

        self.rows.push(EachTemplateRow {
            has_line_break_column: self.current_row.has_line_break_column,
        });

        self.current_row.reset();
    }

    fn finish(mut self) -> EachTemplateTable<'a> {
        self.next_row();

        EachTemplateTable {
            rows: self.rows,
            columns_width: self.columns_width,
            elements: self.elements,
        }
    }
}

#[derive(Debug)]
pub struct EachTemplateTable<'a> {
    /// Information about all rows.
    rows: Vec<EachTemplateRow>,
    /// Contains the maximum length of each column of all rows.
    columns_width: Vec<usize>,
    /// Elements for formatting.
    elements: Vec<EachTemplateElement<'a>>,
}

#[derive(Debug)]
struct EachTemplateCurrentRow {
    /// Contains the maximum length of the current column.
    column_widths: Vec<usize>,
    /// Whether the current row contains a column with a line break.
    has_line_break_column: bool,
}

impl EachTemplateCurrentRow {
    fn new() -> Self {
        Self { column_widths: Vec::new(), has_line_break_column: false }
    }

    fn reset(&mut self) {
        self.column_widths.clear();
        self.has_line_break_column = false;
    }
}

#[derive(Debug)]
struct EachTemplateRow {
    /// Whether the current row contains a column with a line break.
    has_line_break_column: bool,
}

/// Separator between columns in a row.
struct EachTemplateSeparator;

impl<'a> Format<'a> for EachTemplateSeparator {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [token("|")]);
    }
}

impl<'a> EachTemplateTable<'a> {
    pub(crate) fn from_template(
        quasi: &AstNode<'a, TemplateLiteral<'a>>,
        f: &mut Formatter<'_, 'a>,
    ) -> Self {
        let mut builder = EachTemplateTableBuilder::new();

        let mut quasi_iter = quasi.quasis.iter();
        // `unwrap()` is okay, because the template literal is guaranteed to have at least one quasi
        let header = quasi_iter.next().unwrap();
        let header_text = header.value.raw.as_str();

        for column in header_text.split_terminator('|') {
            let trimmed = column.trim();
            let text = f.context().allocator().alloc_str(trimmed);
            let column = EachTemplateColumn::new(text, false);
            builder.entry(EachTemplateElement::Column(column));
        }

        builder.entry(EachTemplateElement::LineBreak);

        for expr in quasi.expressions() {
            let mut vec_buffer = VecBuffer::new(f.state_mut());

            // The softline buffer replaces all softline breaks with a space or removes it entirely
            // to "mimic" an infinite print width
            let mut buffer = RemoveSoftLinesBuffer::new(&mut vec_buffer);

            let options = FormatTemplateExpressionOptions {
                after_new_line: false,
                indention: TemplateElementIndention::default(),
            };

            let mut recording = buffer.start_recording();
            write!(
                recording,
                [FormatTemplateExpression::new(&TemplateExpression::Expression(expr), options)]
            );

            recording.stop();

            let root = Document::new(vec_buffer.into_vec(), Vec::default());

            // let range = element.range();
            let print_options = f.options().as_print_options();
            // TODO: if `unwrap()` panics here, it's a internal error
            let printed = Printer::new(print_options, &[]).print(&root).unwrap();
            let text = f.context().allocator().alloc_str(&printed.into_code());
            let will_break = text.contains('\n');

            let column = EachTemplateColumn::new(text, will_break);
            builder.entry(EachTemplateElement::Column(column));

            if let Some(quasi) = quasi_iter.next() {
                let quasi_text = quasi.value.raw.as_str();

                // go to the next line if the current element contains a line break
                if quasi_text.contains('\n') {
                    builder.entry(EachTemplateElement::LineBreak);
                }
            }
        }

        builder.finish()
    }
}

impl<'a> Format<'a> for EachTemplateTable<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let table_content = format_with(|f| {
            let mut current_column: usize = 0;
            let mut current_row: usize = 0;

            let mut iter = self.elements.iter().peekable();

            write!(f, [hard_line_break()]);

            while let Some(element) = iter.next() {
                let next_item = iter.peek();
                let is_last = next_item.is_none();
                let is_last_in_row =
                    matches!(next_item, Some(EachTemplateElement::LineBreak)) || is_last;

                match element {
                    EachTemplateElement::Column(column) => {
                        let mut content = if current_column != 0
                            && (!is_last_in_row || !column.text.is_empty())
                        {
                            StringBuilder::from_strs_array_in(
                                [" ", column.text],
                                f.context().allocator(),
                            )
                        } else {
                            StringBuilder::from_str_in(column.text, f.context().allocator())
                        };

                        // align the column based on the maximum column width in the table
                        if !is_last_in_row {
                            if !self.rows[current_row].has_line_break_column {
                                let column_width = self
                                    .columns_width
                                    .get(current_column)
                                    .copied()
                                    .unwrap_or_default();

                                let padding = " ".repeat(column_width.saturating_sub(column.width));

                                content.push_str(&padding);
                            }

                            content.push(' ');
                        }

                        write!(f, [text(content.into_str())]);

                        if !is_last_in_row {
                            write!(f, [EachTemplateSeparator]);
                        }

                        current_column += 1;
                    }
                    EachTemplateElement::LineBreak => {
                        current_column = 0;
                        current_row += 1;

                        if !is_last {
                            write!(f, [hard_line_break()]);
                        }
                    }
                }
            }
        });

        write!(f, ["`", indent(&format_args!(table_content)), hard_line_break(), "`"]);
    }
}

fn get_tag_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let expr = expr.get_inner_expression();
    match expr {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        Expression::StaticMemberExpression(member) => get_tag_name(&member.object),
        Expression::ComputedMemberExpression(exp) => get_tag_name(&exp.object),
        Expression::CallExpression(call) => get_tag_name(&call.callee),
        _ => None,
    }
}

/// Format embedded language content (CSS, GraphQL, etc.)
/// inside a template literal using an external formatter (Prettier).
///
/// NOTE: Unlike Prettier, which formats embedded languages in-process via its document IR
/// (e.g. `textToDoc()` → `indent([hardline, doc])`),
/// we communicate with the external formatter over a plain text interface.
///
/// This means we must:
/// - Dedent the inherited JS/TS indentation before sending
/// - Reconstruct the template structure (`block_indent()`) from the formatted text
///
/// If `format_embedded()` could return `FormatElement` (IR) directly,
/// most of work in this function would be unnecessary.
fn format_embedded_template<'a>(
    f: &mut Formatter<'_, 'a>,
    language: &str,
    template_content: &str,
) -> bool {
    // Whitespace-only templates become empty backticks.
    // Regular template literals would preserve them as-is.
    if template_content.trim().is_empty() {
        write!(f, ["``"]);
        return true;
    }

    // Strip inherited indentation.
    // So the external formatter receives clean embedded language content.
    // Otherwise, indentation may be duplicated on each formatting pass.
    let template_content = dedent(template_content, f.context().allocator());

    let Some(Ok(formatted)) =
        f.context().external_callbacks().format_embedded(language, template_content)
    else {
        return false;
    };

    // Format with proper template literal structure:
    // - Opening backtick
    // - Hard line break (newline after backtick)
    // - Indented content (each line will be indented)
    // - Hard line break (newline before closing backtick)
    // - Closing backtick
    let format_content = format_with(|f: &mut Formatter<'_, 'a>| {
        let content = f.context().allocator().alloc_str(&formatted);
        for line in LineTerminatorSplitter::new(content) {
            if line.is_empty() {
                write!(f, [empty_line()]);
            } else {
                write!(f, [text(line), hard_line_break()]);
            }
        }
    });

    write!(f, ["`", block_indent(&format_content), "`"]);

    true
}

/// Strip the common leading indentation from all non-empty lines in `text`.
/// Returns the original `text` unchanged if there is no common indentation.
fn dedent<'a>(text: &'a str, allocator: &'a Allocator) -> &'a str {
    let min_indent = text
        .split('\n')
        .filter(|line| !line.trim_ascii_start().is_empty())
        .map(|line| line.bytes().take_while(u8::is_ascii_whitespace).count())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text;
    }

    let mut result = StringBuilder::with_capacity_in(text.len(), allocator);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let strip = line.bytes().take_while(u8::is_ascii_whitespace).count().min(min_indent);
        result.push_str(&line[strip..]);
    }

    result.into_str()
}

/// Try to format a tagged template with the embedded formatter if supported.
/// Returns `true` if formatting was performed, `false` if not applicable.
fn try_format_embedded_template<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    let quasi = &tagged.quasi;
    // TODO: Support expressions in the template
    if !quasi.is_no_substitution_template() {
        return false;
    }

    let language = match get_tag_name(&tagged.tag) {
        Some("css" | "styled") => "tagged-css",
        Some("gql" | "graphql") => "tagged-graphql",
        Some("html") => "tagged-html",
        Some("md" | "markdown") => "tagged-markdown",
        _ => return false,
    };

    let template_content = quasi.quasis[0].value.raw.as_str();

    format_embedded_template(f, language, template_content)
}

/// Check if the template literal is inside a `css` prop or `<style jsx>` element.
///
/// ```jsx
/// <div css={`color: red;`} />
/// <style jsx>{`div { color: red; }`}</style>
/// ```
fn is_in_css_jsx<'a>(node: &AstNode<'a, TemplateLiteral<'a>>) -> bool {
    let AstNodes::JSXExpressionContainer(container) = node.parent() else {
        return false;
    };

    match container.parent() {
        AstNodes::JSXAttribute(attribute) => {
            if let JSXAttributeName::Identifier(ident) = &attribute.name
                && ident.name == "css"
            {
                return true;
            }
        }
        AstNodes::JSXElement(element) => {
            if let JSXElementName::Identifier(ident) = &element.opening_element.name
                && ident.name == "style"
                && element.opening_element.attributes.iter().any(|attr| {
                    matches!(attr.as_attribute().and_then(|a| a.name.as_identifier()), Some(name) if name.name == "jsx")
                })
            {
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Try to format a template literal inside css prop or styled-jsx with the embedded formatter.
/// Returns `true` if formatting was attempted, `false` if not applicable.
fn try_format_css_template<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Support expressions in the template
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    if !is_in_css_jsx(template_literal) {
        return false;
    }

    let quasi = template_literal.quasis();
    let template_content = quasi[0].value.raw.as_str();

    format_embedded_template(f, "styled-jsx", template_content)
}

/// Try to format a template literal inside Angular @Component's template/styles property.
/// Returns `true` if formatting was performed, `false` if not applicable.
fn try_format_angular_component<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Support expressions in the template
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    // Check if inside `@Component` decorator's `template/styles` property
    let Some(language) = get_angular_component_language(template_literal) else {
        return false;
    };

    let quasi = template_literal.quasis();
    let template_content = quasi[0].value.raw.as_str();

    format_embedded_template(f, language, template_content)
}

/// Check if this template literal is one of:
/// ```ts
/// @Component({
///   template: `...`,
///   styles: `...`,
///   // or styles: [`...`]
/// })
/// ```
fn get_angular_component_language(node: &AstNode<'_, TemplateLiteral<'_>>) -> Option<&'static str> {
    let prop = match node.parent() {
        AstNodes::ObjectProperty(prop) => prop,
        AstNodes::ArrayExpression(arr) => {
            let AstNodes::ObjectProperty(prop) = arr.parent() else {
                return None;
            };
            prop
        }
        _ => return None,
    };

    // Skip computed properties
    if prop.computed {
        return None;
    }
    let PropertyKey::StaticIdentifier(key) = &prop.key else {
        return None;
    };

    // Check parent chain: ObjectExpression -> CallExpression(Component) -> Decorator
    let AstNodes::ObjectExpression(obj) = prop.parent() else {
        return None;
    };
    let AstNodes::CallExpression(call) = obj.parent() else {
        return None;
    };
    let Expression::Identifier(ident) = &call.callee else {
        return None;
    };
    if ident.name.as_str() != "Component" {
        return None;
    }
    if !matches!(call.parent(), AstNodes::Decorator(_)) {
        return None;
    }

    let language = match key.name.as_str() {
        "template" => "angular-template",
        "styles" => "angular-styles",
        _ => return None,
    };
    Some(language)
}
