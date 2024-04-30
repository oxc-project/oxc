mod diagnostics;

use std::rc::Rc;

use oxc_ast::ast::*;
use oxc_span::{Span, SPAN};
use oxc_syntax::number::NumberBase;

use crate::context::Ctx;

use self::diagnostics::DuplicateSourceProp;

const SOURCE: &str = "__source";
const FILE_NAME_VAR: &str = "_jsxFileName";

/// [plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source)
///
/// This plugin is included in `preset-react` and only enabled in development mode.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
///
/// TODO: get lineNumber and columnNumber from somewhere
pub struct ReactJsxSource<'a> {
    ctx: Ctx<'a>,

    /// Has `var _jsxFileName = "";` been added to program.statements?
    should_add_jsx_file_name_variable: bool,
}

impl<'a> ReactJsxSource<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx), should_add_jsx_file_name_variable: false }
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        if !self.should_add_jsx_file_name_variable {
            return;
        }
        let statement = self.get_var_file_name_statement();
        program.body.insert(0, statement);
    }

    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        self.add_source_attribute(elem);
    }

    pub fn get_object_property_kind_for_jsx_plugin(&mut self) -> ObjectPropertyKind<'a> {
        self.should_add_jsx_file_name_variable = true;
        let kind = PropertyKind::Init;
        let ident = IdentifierName::new(SPAN, SOURCE.into());
        let key = self.ctx.ast.property_key_identifier(ident);
        let value = self.get_source_object();
        let obj = self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false);
        ObjectPropertyKind::ObjectProperty(obj)
    }

    pub fn report_error(&self, span: Span) {
        self.ctx.error(DuplicateSourceProp(span));
    }
}

impl<'a> ReactJsxSource<'a> {
    /// `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
    ///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn add_source_attribute(&mut self, elem: &mut JSXOpeningElement<'a>) {
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

        self.should_add_jsx_file_name_variable = true;

        let key = JSXAttributeName::Identifier(
            self.ctx.ast.alloc(self.ctx.ast.jsx_identifier(SPAN, SOURCE.into())),
        );
        let object = self.get_source_object();
        let expr = self.ctx.ast.jsx_expression_container(SPAN, JSXExpression::from(object));
        let value = JSXAttributeValue::ExpressionContainer(expr);
        let attribute_item = self.ctx.ast.jsx_attribute(SPAN, key, Some(value));
        elem.attributes.push(JSXAttributeItem::Attribute(attribute_item));
    }

    fn get_source_object(&self) -> Expression<'a> {
        let kind = PropertyKind::Init;

        let filename = {
            let name = IdentifierName::new(SPAN, "fileName".into());
            let key = self.ctx.ast.property_key_identifier(name);
            let ident = self.ctx.ast.identifier_reference(SPAN, FILE_NAME_VAR);
            let value = self.ctx.ast.identifier_reference_expression(ident);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let line_number = {
            let ident = IdentifierName::new(SPAN, "lineNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let column_number = {
            let ident = IdentifierName::new(SPAN, "columnNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let mut properties = self.ctx.ast.new_vec();
        properties.push(ObjectPropertyKind::ObjectProperty(filename));
        properties.push(ObjectPropertyKind::ObjectProperty(line_number));
        properties.push(ObjectPropertyKind::ObjectProperty(column_number));
        self.ctx.ast.object_expression(SPAN, properties, None)
    }

    fn get_var_file_name_statement(&self) -> Statement<'a> {
        let var_kind = VariableDeclarationKind::Var;
        let id = {
            let ident = BindingIdentifier::new(SPAN, FILE_NAME_VAR.into());
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
        Statement::VariableDeclaration(var_decl)
    }
}
