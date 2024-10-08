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
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-react-jsx-source/src/index.ts>

use oxc_ast::{ast::*, NONE};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Span, SPAN};
use oxc_syntax::{number::NumberBase, symbol::SymbolFlags};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};
use ropey::Rope;

use crate::TransformCtx;

use super::utils::get_line_column;

const SOURCE: &str = "__source";
const FILE_NAME_VAR: &str = "jsxFileName";

pub struct ReactJsxSource<'a, 'ctx> {
    filename_var: Option<BoundIdentifier<'a>>,
    source_rope: Option<Rope>,
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ReactJsxSource<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { filename_var: None, source_rope: None, ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ReactJsxSource<'a, 'ctx> {
    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(stmt) = self.get_filename_var_statement(ctx) {
            self.ctx.top_level_statements.insert_statement(stmt);
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

impl<'a, 'ctx> ReactJsxSource<'a, 'ctx> {
    pub fn get_line_column(&mut self, offset: u32) -> (usize, usize) {
        let source_rope =
            self.source_rope.get_or_insert_with(|| Rope::from_str(self.ctx.source_text));
        get_line_column(source_rope, offset, self.ctx.source_text)
    }

    pub fn get_object_property_kind_for_jsx_plugin(
        &mut self,
        line: usize,
        column: usize,
        ctx: &mut TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let kind = PropertyKind::Init;
        let key = ctx.ast.property_key_identifier_name(SPAN, SOURCE);
        let value = self.get_source_object(line, column, ctx);
        ctx.ast
            .object_property_kind_object_property(SPAN, kind, key, value, None, false, false, false)
    }

    pub fn report_error(&self, span: Span) {
        let error = OxcDiagnostic::warn("Duplicate __source prop found.").with_label(span);
        self.ctx.error(error);
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

        // Check if `__source` attribute already exists
        for item in &elem.attributes {
            if let JSXAttributeItem::Attribute(attribute) = item {
                if let JSXAttributeName::Identifier(ident) = &attribute.name {
                    if ident.name == SOURCE {
                        self.report_error(ident.span);
                        return;
                    }
                }
            }
        }

        let key = ctx.ast.jsx_attribute_name_jsx_identifier(SPAN, SOURCE);
        // TODO: We shouldn't calculate line + column from scratch each time as it's expensive.
        // Build a table of byte indexes of each line's start on first usage, and save it.
        // Then calculate line and column from that.
        let (line, column) = self.get_line_column(elem.span.start);
        let object = self.get_source_object(line, column, ctx);
        let value =
            ctx.ast.jsx_attribute_value_jsx_expression_container(SPAN, JSXExpression::from(object));
        let attribute_item = ctx.ast.jsx_attribute_item_jsx_attribute(SPAN, key, Some(value));
        elem.attributes.push(attribute_item);
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn get_source_object(
        &mut self,
        line: usize,
        column: usize,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let kind = PropertyKind::Init;

        let filename = {
            let key = ctx.ast.property_key_identifier_name(SPAN, "fileName");
            let value = self.get_filename_var(ctx).create_read_expression(ctx);
            ctx.ast.object_property_kind_object_property(
                SPAN, kind, key, value, None, false, false, false,
            )
        };

        let line_number = {
            let key = ctx.ast.property_key_identifier_name(SPAN, "lineNumber");
            let value = ctx.ast.expression_numeric_literal(
                SPAN,
                line as f64,
                line.to_string(),
                NumberBase::Decimal,
            );
            ctx.ast.object_property_kind_object_property(
                SPAN, kind, key, value, None, false, false, false,
            )
        };

        let column_number = {
            let key = ctx.ast.property_key_identifier_name(SPAN, "columnNumber");
            let value = ctx.ast.expression_numeric_literal(
                SPAN,
                column as f64,
                column.to_string(),
                NumberBase::Decimal,
            );
            ctx.ast.object_property_kind_object_property(
                SPAN, kind, key, value, None, false, false, false,
            )
        };

        let mut properties = ctx.ast.vec_with_capacity(3);
        properties.push(filename);
        properties.push(line_number);
        properties.push(column_number);
        ctx.ast.expression_object(SPAN, properties, None)
    }

    pub fn get_filename_var_statement(&self, ctx: &mut TraverseCtx<'a>) -> Option<Statement<'a>> {
        let decl = self.get_filename_var_declarator(ctx)?;

        let var_decl = Statement::VariableDeclaration(ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(decl),
            false,
        ));
        Some(var_decl)
    }

    pub fn get_filename_var_declarator(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<VariableDeclarator<'a>> {
        let filename_var = self.filename_var.as_ref()?;

        let var_kind = VariableDeclarationKind::Var;
        let id = {
            let ident = filename_var.create_binding_identifier();
            let ident = ctx.ast.binding_pattern_kind_from_binding_identifier(ident);
            ctx.ast.binding_pattern(ident, NONE, false)
        };
        let decl = {
            let init =
                ctx.ast.expression_string_literal(SPAN, self.ctx.source_path.to_string_lossy());
            ctx.ast.variable_declarator(SPAN, var_kind, id, Some(init), false)
        };
        Some(decl)
    }

    fn get_filename_var(&mut self, ctx: &mut TraverseCtx<'a>) -> &BoundIdentifier<'a> {
        self.filename_var.get_or_insert_with(|| {
            ctx.generate_uid_in_root_scope(FILE_NAME_VAR, SymbolFlags::FunctionScopedVariable)
        })
    }
}
