use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Span, SPAN};
use oxc_syntax::{number::NumberBase, symbol::SymbolFlags};
use oxc_traverse::TraverseCtx;

use crate::{context::Ctx, helpers::bindings::BoundIdentifier};

use super::utils::get_line_column;

const SOURCE: &str = "__source";
const FILE_NAME_VAR: &str = "jsxFileName";

/// [plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source)
///
/// This plugin is included in `preset-react` and only enabled in development mode.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
pub struct ReactJsxSource<'a> {
    ctx: Ctx<'a>,
    filename_var: Option<BoundIdentifier<'a>>,
}

impl<'a> ReactJsxSource<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { ctx, filename_var: None }
    }

    pub fn transform_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_source_attribute(elem, ctx);
    }

    pub fn get_object_property_kind_for_jsx_plugin(
        &mut self,
        line: usize,
        column: usize,
        ctx: &mut TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let kind = PropertyKind::Init;
        let ident = IdentifierName::new(SPAN, SOURCE.into());
        let key = self.ctx.ast.property_key_identifier(ident);
        let value = self.get_source_object(line, column, ctx);
        let obj = self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false);
        ObjectPropertyKind::ObjectProperty(obj)
    }

    pub fn report_error(&self, span: Span) {
        let error = OxcDiagnostic::warn("Duplicate __source prop found.").with_label(span);
        self.ctx.error(error);
    }
}

impl<'a> ReactJsxSource<'a> {
    /// `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
    ///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn add_source_attribute(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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

        let key = JSXAttributeName::Identifier(
            self.ctx.ast.alloc(self.ctx.ast.jsx_identifier(SPAN, SOURCE.into())),
        );
        let (line, column) = get_line_column(elem.span.start, self.ctx.source_text);
        let object = self.get_source_object(line, column, ctx);
        let expr = self.ctx.ast.jsx_expression_container(SPAN, JSXExpression::from(object));
        let value = JSXAttributeValue::ExpressionContainer(expr);
        let attribute_item = self.ctx.ast.jsx_attribute(SPAN, key, Some(value));
        elem.attributes.push(JSXAttributeItem::Attribute(attribute_item));
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
            let name = IdentifierName::new(SPAN, "fileName".into());
            let key = self.ctx.ast.property_key_identifier(name);
            let ident = self.get_filename_var(ctx).create_read_reference(ctx);
            let value = self.ctx.ast.identifier_reference_expression(ident);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let line_number = {
            let ident = IdentifierName::new(SPAN, "lineNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(
                SPAN,
                line as f64,
                self.ctx.ast.new_str(&line.to_string()),
                NumberBase::Decimal,
            );
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let column_number = {
            let ident = IdentifierName::new(SPAN, "columnNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(
                SPAN,
                column as f64,
                self.ctx.ast.new_str(&column.to_string()),
                NumberBase::Decimal,
            );
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let mut properties = self.ctx.ast.new_vec();
        properties.push(ObjectPropertyKind::ObjectProperty(filename));
        properties.push(ObjectPropertyKind::ObjectProperty(line_number));
        properties.push(ObjectPropertyKind::ObjectProperty(column_number));
        self.ctx.ast.object_expression(SPAN, properties, None)
    }

    pub fn get_var_file_name_statement(&mut self) -> Option<Statement<'a>> {
        let filename_var = self.filename_var.as_ref()?;

        let var_kind = VariableDeclarationKind::Var;
        let id = {
            let ident = filename_var.create_binding_identifier();
            let ident = self.ctx.ast.binding_pattern_identifier(ident);
            self.ctx.ast.binding_pattern(ident, None, false)
        };
        let decl = {
            let string = self.ctx.ast.string_literal(SPAN, &self.ctx.source_path.to_string_lossy());
            let init = self.ctx.ast.literal_string_expression(string);
            let decl = self.ctx.ast.variable_declarator(SPAN, var_kind, id, Some(init), false);
            self.ctx.ast.new_vec_single(decl)
        };
        let var_decl = self.ctx.ast.variable_declaration(SPAN, var_kind, decl, Modifiers::empty());
        Some(Statement::VariableDeclaration(var_decl))
    }

    fn get_filename_var(&mut self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        if self.filename_var.is_none() {
            self.filename_var = Some(BoundIdentifier::new_root_uid(
                FILE_NAME_VAR,
                SymbolFlags::FunctionScopedVariable,
                ctx,
            ));
        }
        self.filename_var.as_ref().unwrap().clone()
    }
}
