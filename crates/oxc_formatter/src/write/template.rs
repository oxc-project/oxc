use unicode_width::UnicodeWidthStr;

use std::cmp;

use oxc_allocator::{StringBuilder, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    IndentWidth,
    ast_nodes::{AstNode, AstNodeIterator},
    format, format_args,
    formatter::{
        Format, FormatElement, FormatResult, Formatter, VecBuffer,
        buffer::RemoveSoftLinesBuffer,
        prelude::{document::Document, *},
        printer::Printer,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    utils::{
        call_expression::is_test_each_pattern,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    },
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let template = TemplateLike::TemplateLiteral(self);
        write!(f, template)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Format the tag and type arguments
        write!(f, [self.tag(), self.type_arguments(), line_suffix_boundary()])?;

        let quasi = self.quasi();

        quasi.format_leading_comments(f);

        if is_test_each_pattern(&self.tag) {
            let template = &EachTemplateTable::from_template(quasi, f)?;
            // Use table formatting
            write!(f, template)
        } else {
            let template = TemplateLike::TemplateLiteral(quasi);
            write!(f, template)
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value.raw.as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTemplateLiteralType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let template = TemplateLike::TSTemplateLiteralType(self);
        write!(f, template)
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
    pub fn span(&self) -> Span {
        match self {
            Self::TemplateLiteral(t) => t.span,
            Self::TSTemplateLiteralType(t) => t.span,
        }
    }

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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;

        let quasis = self.quasis();
        let mut indention = TemplateElementIndention::default();
        let mut after_new_line = false;

        let mut expression_iterator = match self {
            Self::TemplateLiteral(t) => {
                TemplateExpressionIterator::Expression(t.expressions().iter())
            }
            Self::TSTemplateLiteralType(t) => TemplateExpressionIterator::TSType(t.types().iter()),
        };

        for quasi in quasis {
            write!(f, quasi)?;

            let quasi_text = quasi.value.raw.as_str();

            if let Some(expr) = expression_iterator.next() {
                let tab_width = u32::from(f.options().indent_width.value());
                indention =
                    TemplateElementIndention::after_last_new_line(quasi_text, tab_width, indention);
                after_new_line = quasi_text.ends_with('\n');
                let options = FormatTemplateExpressionOptions { indention, after_new_line };
                FormatTemplateExpression::new(&expr, options).fmt(f)?;
            }
        }

        write!(f, "`")
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let options = self.options;

        let mut has_comment_in_expression = false;

        // First, format the expression to check if it will break
        // Special handling for array expressions - force flat mode
        let format_expression = format_once(|f| match self.expression {
            TemplateExpression::Expression(e) => {
                let leading_comments = f.context().comments().comments_before(e.span().start);
                FormatLeadingComments::Comments(leading_comments).fmt(f)?;
                FormatNodeWithoutTrailingComments(e).fmt(f)?;
                let trailing_comments =
                    f.context().comments().comments_before_character(e.span().start, b'}');
                has_comment_in_expression =
                    !leading_comments.is_empty() || !trailing_comments.is_empty();
                FormatTrailingComments::Comments(trailing_comments).fmt(f)
            }
            TemplateExpression::TSType(t) => write!(f, t),
        });

        // Intern the expression to check if it will break
        let interned_expression = f.intern(&format_expression)?;

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
                match &interned_expression {
                    Some(element) => buffer.write_element(element.clone()),
                    None => Ok(()),
                }
            }
            TemplateElementLayout::Fit => {
                // Determine if we should add indentation based on expression complexity
                let indent = match self.expression {
                    TemplateExpression::Expression(e) => {
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
                    }
                    TemplateExpression::TSType(t) => {
                        self.options.after_new_line || is_complex_type(t.as_ref())
                    }
                };

                match &interned_expression {
                    Some(element) if indent => {
                        write!(
                            f,
                            [soft_block_indent(&format_with(|f| f.write_element(element.clone())))]
                        )
                    }
                    Some(element) => f.write_element(element.clone()),
                    None => Ok(()),
                }
            }
        });

        let format_indented = format_with(|f: &mut Formatter<'_, 'a>| {
            if options.after_new_line {
                // Apply dedent_to_root for expressions after newlines
                write!(f, [dedent_to_root(&format_inner)])
            } else {
                write_with_indention(&format_inner, options.indention, f.options().indent_width, f)
            }
        });

        // Wrap in ${...} with group
        write!(f, [group(&format_args!("${", format_indented, line_suffix_boundary(), "}"))])
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
) -> FormatResult<()>
where
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
            f.write_element(FormatElement::Tag(Tag::StartIndent))?;
        }

        write!(f, [content])?;

        for _ in 0..level {
            f.write_element(FormatElement::Tag(Tag::EndIndent))?;
        }

        Ok(())
    });

    // Adds any necessary `align` for spaces not covered by indent level.
    let format_aligned = format_with(|f| {
        if spaces == 0 {
            write!(f, [format_indented])
        } else {
            write!(f, [align(spaces, &format_indented)])
        }
    });

    write!(f, [dedent_to_root(&format_aligned)])
}

/// Check if a TypeScript type is complex enough to warrant line breaks
#[inline]
fn is_complex_type(ts_type: &TSType) -> bool {
    matches!(
        ts_type,
        TSType::TSConditionalType(_)
            | TSType::TSMappedType(_)
            | TSType::TSTypeLiteral(_)
            | TSType::TSIntersectionType(_)
            | TSType::TSUnionType(_)
            | TSType::TSTupleType(_)
            | TSType::TSArrayType(_)
            | TSType::TSIndexedAccessType(_)
    )
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [text("|")])
    }
}

impl<'a> EachTemplateTable<'a> {
    pub(crate) fn from_template(
        quasi: &AstNode<'a, TemplateLiteral<'a>>,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<Self> {
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

        for (i, expr) in quasi.expressions().iter().enumerate() {
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
            )?;

            recording.stop();

            let root = Document::from(vec_buffer.into_vec());

            // let range = element.range();
            let print_options = f.options().as_print_options();
            let printed = Printer::new(print_options).print(&root)?;
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

        let table = builder.finish();

        Ok(table)
    }
}

impl<'a> Format<'a> for EachTemplateTable<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let table_content = format_with(|f| {
            let mut current_column: usize = 0;
            let mut current_row: usize = 0;

            let mut iter = self.elements.iter().peekable();

            write!(f, [hard_line_break()])?;

            while let Some(element) = iter.next() {
                let next_item = iter.peek();
                let is_last = next_item.is_none();
                let is_last_in_row =
                    matches!(next_item, Some(EachTemplateElement::LineBreak)) || is_last;

                match element {
                    EachTemplateElement::Column(column) => {
                        let mut text = if current_column != 0
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

                                text.push_str(&padding);
                            }

                            text.push(' ');
                        }

                        write!(f, [dynamic_text(text.into_str())])?;

                        if !is_last_in_row {
                            write!(f, [EachTemplateSeparator])?;
                        }

                        current_column += 1;
                    }
                    EachTemplateElement::LineBreak => {
                        current_column = 0;
                        current_row += 1;

                        if !is_last {
                            write!(f, [hard_line_break()])?;
                        }
                    }
                }
            }
            Ok(())
        });

        write!(f, ["`", indent(&format_args!(table_content)), hard_line_break(), "`"])
    }
}
