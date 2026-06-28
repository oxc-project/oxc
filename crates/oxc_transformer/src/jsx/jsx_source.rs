//! React JSX Source
//!
//! This plugin adds `__source` attribute to JSX elements.
//!
//! > This plugin is included in `preset-react`.
//!
//! ## Example
//!
//! Input:
//! ```js
//! <div>foo</div>;
//! <Bar>foo</Bar>;
//! <>foo</>;
//! ```
//!
//! Output:
//! ```js
//! var _jsxFileName = "<CWD>/test.js";
//! <div __source={
//!     { fileName: _jsxFileName, lineNumber: 1, columnNumber: 1 }
//! }>foo</div>;
//! <Bar __source={
//!     { fileName: _jsxFileName, lineNumber: 2, columnNumber: 1 }
//! }>foo</Bar>;
//! <>foo</>;
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source).
//!
//! ## References:
//!
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/v7.26.2/packages/babel-plugin-transform-react-jsx-source/src/index.ts>

use oxc_allocator::ArenaVec;
use oxc_ast::{ast::*, builder::NONE};
use oxc_span::SPAN;
use oxc_syntax::{number::NumberBase, symbol::SymbolFlags};
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

const SOURCE: &str = "__source";
const FILE_NAME_VAR: &str = "jsxFileName";

pub struct JsxSource<'a> {
    filename_var: Option<BoundIdentifier<'a>>,
    /// Byte offset of the start of each line, built lazily from the source text on
    /// first use. See [`JsxSource::build_line_offset_table`].
    line_offset_table: Option<Vec<u32>>,
}

impl JsxSource<'_> {
    pub fn new() -> Self {
        Self { filename_var: None, line_offset_table: None }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for JsxSource<'a> {
    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(stmt) = self.get_filename_var_statement(ctx) {
            ctx.state.top_level_statements.insert_statement(stmt);
        }
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_source_attribute(elem, ctx);
    }
}

/// Build a table of the byte offset of the start of each line.
///
/// Lines are delimited by the same set of line breaks `ropey` recognises (its default
/// Unicode set), so the line/column computed from this table match the previous
/// `oxc_data_structures::rope`-based implementation exactly:
/// LF (`\n`), VT (`\u{B}`), FF (`\u{C}`), CR (`\r`), CRLF (counted as a single break),
/// NEL (`\u{85}`), LS (`\u{2028}`) and PS (`\u{2029}`).
///
/// Scanning is byte-wise: the single-byte breaks are all `< 0x80` and the multi-byte ones
/// (`NEL`/`LS`/`PS`) start with lead bytes that never appear as UTF-8 continuation bytes,
/// so no break can be matched inside another code point.
fn build_line_offset_table(source_text: &str) -> Vec<u32> {
    let bytes = source_text.as_bytes();
    let mut line_starts = vec![0u32];
    let mut i = 0;
    while i < bytes.len() {
        let break_len = match bytes[i] {
            // LF, VT, FF
            0x0A | 0x0B | 0x0C => 1,
            // CR on its own, or CRLF treated as a single break
            0x0D => 1 + usize::from(bytes.get(i + 1) == Some(&0x0A)),
            // NEL = U+0085 = 0xC2 0x85
            0xC2 if bytes.get(i + 1) == Some(&0x85) => 2,
            // LS = U+2028 = 0xE2 0x80 0xA8, PS = U+2029 = 0xE2 0x80 0xA9
            0xE2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(0xA8 | 0xA9)) =>
            {
                3
            }
            _ => 0,
        };
        if break_len == 0 {
            i += 1;
        } else {
            i += break_len;
            line_starts.push(i as u32);
        }
    }
    line_starts
}

/// Compute 1-indexed line and (UTF-16) column for `offset` from a line-offset table built
/// by [`build_line_offset_table`].
fn line_column(source_text: &str, line_starts: &[u32], offset: u32) -> (u32, u32) {
    // The line index is the number of line breaks before `offset`. `line_starts` is sorted
    // and always begins with `0`, so at least one entry is `<= offset` and the subtraction
    // never underflows.
    let offset = offset as usize;
    let line = line_starts.partition_point(|&start| start as usize <= offset) - 1;
    let line_offset = line_starts[line] as usize;
    // Column is measured in UTF-16 code units, matching Babel.
    let column = source_text[line_offset..offset].encode_utf16().count();
    // line and column are zero-indexed, but we want 1-indexed
    (line as u32 + 1, column as u32 + 1)
}

impl<'a> JsxSource<'a> {
    /// Get line and column from offset and source text.
    ///
    /// Line number starts at 1.
    /// Column number is in UTF-16 characters, and starts at 1.
    ///
    /// This matches Babel's output.
    pub fn get_line_column(&mut self, offset: u32, ctx: &TraverseCtx<'a>) -> (u32, u32) {
        let source_text = ctx.state.source_text;
        let line_starts =
            self.line_offset_table.get_or_insert_with(|| build_line_offset_table(source_text));
        line_column(source_text, line_starts, offset)
    }

    pub fn get_object_property_kind_for_jsx_plugin(
        &mut self,
        line: u32,
        column: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let kind = PropertyKind::Init;
        let key = PropertyKey::new_static_identifier(SPAN, SOURCE, ctx);
        let value = self.get_source_object(line, column, ctx);
        ObjectPropertyKind::new_object_property(SPAN, kind, key, value, false, false, false, ctx)
    }

    /// `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
    ///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn add_source_attribute(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Don't add `__source` if this node was generated
        if elem.span.is_unspanned() {
            return;
        }

        // Don't add `__source` if it already exists
        if elem.attributes.iter().any(|item| {
            matches!(item, JSXAttributeItem::Attribute(attribute)
                if matches!(&attribute.name, JSXAttributeName::Identifier(ident) if ident.name == SOURCE))
        }) {
            return;
        }

        let key = JSXAttributeName::new_identifier(SPAN, SOURCE, ctx);
        let (line, column) = self.get_line_column(elem.span.start, ctx);
        let object = self.get_source_object(line, column, ctx);
        let value =
            JSXAttributeValue::new_expression_container(SPAN, JSXExpression::from(object), ctx);
        let attribute_item = JSXAttributeItem::new_attribute(SPAN, key, Some(value), ctx);
        elem.attributes.push(attribute_item);
    }

    #[expect(clippy::cast_lossless)]
    pub fn get_source_object(
        &mut self,
        line: u32,
        column: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let kind = PropertyKind::Init;

        let filename = {
            let key = PropertyKey::new_static_identifier(SPAN, "fileName", ctx);
            let value = self.get_filename_var(ctx).create_read_expression(ctx);
            ObjectPropertyKind::new_object_property(
                SPAN, kind, key, value, false, false, false, ctx,
            )
        };

        let line_number = {
            let key = PropertyKey::new_static_identifier(SPAN, "lineNumber", ctx);
            let value =
                Expression::new_numeric_literal(SPAN, line as f64, None, NumberBase::Decimal, ctx);
            ObjectPropertyKind::new_object_property(
                SPAN, kind, key, value, false, false, false, ctx,
            )
        };

        let column_number = {
            let key = PropertyKey::new_static_identifier(SPAN, "columnNumber", ctx);
            let value = Expression::new_numeric_literal(
                SPAN,
                column as f64,
                None,
                NumberBase::Decimal,
                ctx,
            );
            ObjectPropertyKind::new_object_property(
                SPAN, kind, key, value, false, false, false, ctx,
            )
        };

        let properties = ArenaVec::from_array_in([filename, line_number, column_number], ctx);
        Expression::new_object_expression(SPAN, properties, ctx)
    }

    pub fn get_filename_var_statement(&self, ctx: &TraverseCtx<'a>) -> Option<Statement<'a>> {
        let decl = self.get_filename_var_declarator(ctx)?;

        let var_decl = Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ArenaVec::from_value_in(decl, ctx),
            false,
            ctx,
        );
        Some(var_decl)
    }

    pub fn get_filename_var_declarator(
        &self,
        ctx: &TraverseCtx<'a>,
    ) -> Option<VariableDeclarator<'a>> {
        let filename_var = self.filename_var.as_ref()?;

        let id = filename_var.create_binding_pattern(ctx);
        let source_path = Str::from_str_in(&ctx.state.source_path.to_string_lossy(), ctx);
        let init = Expression::new_string_literal(SPAN, source_path, None, ctx);
        let decl = VariableDeclarator::new(
            SPAN,
            VariableDeclarationKind::Var,
            id,
            NONE,
            Some(init),
            false,
            ctx,
        );
        Some(decl)
    }

    fn get_filename_var(&mut self, ctx: &mut TraverseCtx<'a>) -> &BoundIdentifier<'a> {
        self.filename_var.get_or_insert_with(|| {
            ctx.generate_uid_in_root_scope(FILE_NAME_VAR, SymbolFlags::FunctionScopedVariable)
        })
    }
}

#[cfg(test)]
mod test {
    use super::{build_line_offset_table, line_column};

    /// 1-indexed line and UTF-16 column, matching Babel / the old rope implementation.
    fn lc(source: &str, offset: u32) -> (u32, u32) {
        line_column(source, &build_line_offset_table(source), offset)
    }

    #[test]
    fn line_column_across_line_breaks() {
        assert_eq!(lc("", 0), (1, 1));
        assert_eq!(lc("abc", 0), (1, 1));
        assert_eq!(lc("abc", 2), (1, 3));
        // LF, lone CR, CRLF (counted as a single break)
        assert_eq!(lc("a\nb", 2), (2, 1));
        assert_eq!(lc("a\rb", 2), (2, 1));
        assert_eq!(lc("a\r\nb", 3), (2, 1));
        // LF immediately followed by CR is two separate breaks
        assert_eq!(lc("a\n\rb", 3), (3, 1));
        // VT, FF, NEL, LS, PS are all line breaks
        assert_eq!(lc("a\u{0B}b", 2), (2, 1));
        assert_eq!(lc("a\u{0C}b", 2), (2, 1));
        assert_eq!(lc("a\u{85}b", 3), (2, 1));
        assert_eq!(lc("a\u{2028}b", 4), (2, 1));
        assert_eq!(lc("a\u{2029}b", 4), (2, 1));
        // Column counts UTF-16 code units: an astral char is 2 units.
        assert_eq!(lc("𠮷x", 4), (1, 3));
        assert_eq!(lc("a\n𠮷x", 6), (2, 3));
    }
}
