mod options;

use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    identifier::{is_irregular_whitespace, is_line_terminator},
    xml_entities::XML_ENTITIES,
};

pub use self::options::{ReactJsxOptions, ReactJsxRuntime};
use crate::context::TransformerCtx;

#[derive(Debug, Error, Diagnostic)]
#[error("pragma and pragmaFrag cannot be set when runtime is automatic.")]
#[diagnostic(severity(warning), help("Remove `pragma` and `pragmaFrag` options."))]
struct PragmaAndPragmaFragCannotBeSet;

/// Transform React JSX
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
pub struct ReactJsx<'a> {
    ast: Rc<AstBuilder<'a>>,
    ctx: TransformerCtx<'a>,
    options: ReactJsxOptions,

    imports: Vec<'a, Statement<'a>>,
    import_jsx: bool,
    import_jsxs: bool,
    import_fragment: bool,
    import_create_element: bool,
    require_jsx_runtime: bool,
    // Will be store jsx runtime importer, like `react/jsx-runtime`
    jsx_runtime_importer: Atom,
}

enum JSXElementOrFragment<'a, 'b> {
    Element(&'b JSXElement<'a>),
    Fragment(&'b JSXFragment<'a>),
}

impl<'a, 'b> JSXElementOrFragment<'a, 'b> {
    fn attributes(&self) -> Option<&'b Vec<'a, JSXAttributeItem<'a>>> {
        match self {
            Self::Element(e) if !e.opening_element.attributes.is_empty() => {
                Some(&e.opening_element.attributes)
            }
            _ => None,
        }
    }

    fn children(&self) -> &'b Vec<'a, JSXChild<'a>> {
        match self {
            Self::Element(e) => &e.children,
            Self::Fragment(e) => &e.children,
        }
    }

    /// The react jsx/jsxs transform falls back to `createElement` when an explicit `key` argument comes after a spread
    /// <https://github.com/microsoft/TypeScript/blob/6134091642f57c32f50e7b5604635e4d37dd19e8/src/compiler/transformers/jsx.ts#L264-L278>
    fn has_key_after_props_spread(&self) -> bool {
        let Self::Element(e) = self else { return false };
        let mut spread = false;
        for attr in &e.opening_element.attributes {
            if matches!(attr, JSXAttributeItem::SpreadAttribute(_)) {
                spread = true;
            } else if spread && matches!(attr, JSXAttributeItem::Attribute(a) if a.is_key()) {
                return true;
            }
        }
        false
    }
}

impl<'a> ReactJsx<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, ctx: TransformerCtx<'a>, options: ReactJsxOptions) -> Self {
        let imports = ast.new_vec();
        let options = options.with_comments(&ctx.semantic());

        let jsx_runtime_importer =
            if options.import_source == "react" || options.runtime.is_classic() {
                Atom::new_inline("react/jsx-runtime")
            } else {
                Atom::from(format!("{}/jsx-runtime", options.import_source))
            };

        Self {
            ast,
            ctx,
            options,
            imports,
            jsx_runtime_importer,
            require_jsx_runtime: false,
            import_jsx: false,
            import_jsxs: false,
            import_fragment: false,
            import_create_element: false,
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::JSXElement(e) => {
                *expr = self.transform_jsx(&JSXElementOrFragment::Element(e));
            }
            Expression::JSXFragment(e) => {
                *expr = self.transform_jsx(&JSXElementOrFragment::Fragment(e));
            }
            _ => {}
        }
    }

    pub fn add_react_jsx_runtime_imports(&mut self, program: &mut Program<'a>) {
        if self.options.runtime.is_classic() {
            return;
        }

        if self.options.pragma != "React.createElement"
            || self.options.pragma_frag != "React.Fragment"
        {
            self.ctx.error(PragmaAndPragmaFragCannotBeSet);
            return;
        }

        let imports = self.ast.move_statement_vec(&mut self.imports);
        let index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ModuleDeclaration(m) if m.is_import()))
            .map_or(0, |i| i + 1);
        program.body.splice(index..index, imports);
    }

    fn new_string_literal(name: &str) -> StringLiteral {
        StringLiteral::new(SPAN, name.into())
    }

    fn add_import<'b>(
        &mut self,
        e: &JSXElementOrFragment<'a, 'b>,
        has_key_after_props_spread: bool,
        need_jsxs: bool,
    ) {
        if self.options.runtime.is_classic() {
            return;
        }
        match e {
            JSXElementOrFragment::Element(_) if has_key_after_props_spread => {
                self.add_import_create_element();
            }
            JSXElementOrFragment::Element(_) if need_jsxs => self.add_import_jsxs(),
            JSXElementOrFragment::Element(_) => self.add_import_jsx(),
            JSXElementOrFragment::Fragment(_) => self.add_import_fragment(),
        }
    }

    fn add_require_jsx_runtime(&mut self) {
        if !self.require_jsx_runtime {
            self.require_jsx_runtime = true;
            self.add_require_statement(
                "_reactJsxRuntime",
                Self::new_string_literal(self.jsx_runtime_importer.as_str()),
                false,
            );
        }
    }

    fn add_import_jsx(&mut self) {
        if self.ctx.source_type().is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_jsx {
            self.import_jsx = true;
            self.add_import_statement(
                "jsx",
                "_jsx",
                Self::new_string_literal(self.jsx_runtime_importer.as_str()),
            );
        }
    }

    fn add_import_jsxs(&mut self) {
        if self.ctx.source_type().is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_jsxs {
            self.import_jsxs = true;
            let source = Self::new_string_literal(self.jsx_runtime_importer.as_str());
            self.add_import_statement("jsxs", "_jsxs", source);
        }
    }

    fn add_import_fragment(&mut self) {
        if self.ctx.source_type().is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_fragment {
            self.import_fragment = true;
            let source = Self::new_string_literal(self.jsx_runtime_importer.as_str());
            self.add_import_statement("Fragment", "_Fragment", source);
            self.add_import_jsx();
        }
    }

    fn add_import_create_element(&mut self) {
        if !self.import_create_element {
            self.import_create_element = true;

            if self.ctx.source_type().is_script() {
                self.add_require_statement(
                    "_react",
                    Self::new_string_literal(self.options.import_source.as_ref()),
                    true,
                );
            } else {
                let source = Self::new_string_literal(self.options.import_source.as_ref());
                self.add_import_statement("createElement", "_createElement", source);
            }
        }
    }

    fn add_import_statement(&mut self, imported: &str, local: &str, source: StringLiteral) {
        let mut specifiers = self.ast.new_vec_with_capacity(1);
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
            span: SPAN,
            imported: ModuleExportName::Identifier(IdentifierName::new(SPAN, imported.into())),
            local: BindingIdentifier::new(SPAN, local.into()),
            import_kind: ImportOrExportKind::Value,
        }));
        let import_statement = self.ast.import_declaration(
            SPAN,
            Some(specifiers),
            source,
            None,
            ImportOrExportKind::Value,
        );
        let decl =
            self.ast.module_declaration(ModuleDeclaration::ImportDeclaration(import_statement));
        self.imports.push(decl);
    }

    fn add_require_statement(&mut self, variable_name: &str, source: StringLiteral, front: bool) {
        let callee = self
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, "require".into()));
        let arguments = self
            .ast
            .new_vec_single(Argument::Expression(self.ast.literal_string_expression(source)));
        let init = self.ast.call_expression(SPAN, callee, arguments, false, None);
        let id = self.ast.binding_pattern(
            self.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, variable_name.into())),
            None,
            false,
        );
        let decl = self.ast.new_vec_single(self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            id,
            Some(init),
            false,
        ));

        let variable_declaration = self.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            decl,
            Modifiers::empty(),
        );
        let stmt = Statement::Declaration(Declaration::VariableDeclaration(variable_declaration));

        if front {
            self.imports.insert(0, stmt);
        } else {
            self.imports.push(stmt);
        }
    }

    fn transform_jsx<'b>(&mut self, e: &JSXElementOrFragment<'a, 'b>) -> Expression<'a> {
        let is_classic = self.options.runtime.is_classic();
        let is_automatic = self.options.runtime.is_automatic();
        let has_key_after_props_spread = e.has_key_after_props_spread();

        // TODO: compute the correct capacity for both runtimes
        let mut arguments = self.ast.new_vec_with_capacity(1);

        arguments.push(Argument::Expression(match e {
            JSXElementOrFragment::Element(e) => {
                self.transform_element_name(&e.opening_element.name)
            }
            JSXElementOrFragment::Fragment(_) => self.get_fragment(),
        }));

        // The key prop in `<div key={true} />`
        let mut key_prop = None;

        let attributes = e.attributes();
        let attributes_len = attributes.map_or(0, |attrs| attrs.len());

        // Add `null` to second argument in classic mode
        if is_classic && attributes_len == 0 {
            let null_expr = self.ast.literal_null_expression(NullLiteral::new(SPAN));
            arguments.push(Argument::Expression(null_expr));
        }

        // The object properties for the second argument of `React.createElement`
        let mut properties = self.ast.new_vec_with_capacity(0);

        if let Some(attributes) = attributes {
            // TODO: compute the correct capacity for both runtimes

            for attribute in attributes {
                // optimize `{...prop}` to `prop` in static mode
                if is_classic && attributes_len == 1 {
                    if let JSXAttributeItem::SpreadAttribute(spread) = attribute {
                        // deopt if spreading an object with `__proto__` key
                        if !matches!(&spread.argument, Expression::ObjectExpression(o) if o.has_proto())
                        {
                            arguments.push(Argument::Expression(self.ast.copy(&spread.argument)));
                            continue;
                        }
                    }
                }
                // In automatic mode, extract the key before spread prop,
                // and add it to the third argument later.
                if is_automatic && !has_key_after_props_spread {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        if attr.is_key() {
                            key_prop = attr.value.as_ref();
                            continue;
                        }
                    }
                }

                // Add attribute to prop object
                self.transform_jsx_attribute_item(&mut properties, attribute);
            }
        }

        let mut need_jsxs = false;

        let children = e.children();

        // Append children to object properties in automatic mode
        if is_automatic {
            let allocator = self.ast.allocator;
            let mut children = Vec::from_iter_in(
                children.iter().filter_map(|child| self.transform_jsx_child(child)),
                allocator,
            );
            let children_len = children.len();
            if children_len != 0 {
                let value = if children_len == 1 {
                    children.pop().unwrap()
                } else {
                    let elements = Vec::from_iter_in(
                        children.into_iter().map(ArrayExpressionElement::Expression),
                        allocator,
                    );
                    need_jsxs = true;
                    self.ast.array_expression(SPAN, elements, None)
                };

                let kind = PropertyKind::Init;
                let ident = IdentifierName::new(SPAN, "children".into());
                let key = self.ast.property_key_identifier(ident);
                let object_property =
                    self.ast.object_property(SPAN, kind, key, value, None, false, false, false);
                properties.push(ObjectPropertyKind::ObjectProperty(object_property));
            }
        }

        self.add_import(e, has_key_after_props_spread, need_jsxs);

        if !properties.is_empty() || is_automatic {
            let object_expression = self.ast.object_expression(SPAN, properties, None);
            arguments.push(Argument::Expression(object_expression));
        }

        if is_automatic && key_prop.is_some() {
            arguments.push(Argument::Expression(self.transform_jsx_attribute_value(key_prop)));
        }

        if is_classic && !children.is_empty() {
            arguments.extend(
                children
                    .iter()
                    .filter_map(|child| self.transform_jsx_child(child))
                    .map(Argument::Expression),
            );
        }

        let callee = self.get_create_element(has_key_after_props_spread, need_jsxs);
        self.ast.call_expression(SPAN, callee, arguments, false, None)
    }

    fn get_react_references(&mut self) -> Expression<'a> {
        let ident = IdentifierReference::new(SPAN, "React".into());
        self.ast.identifier_reference_expression(ident)
    }

    fn get_static_member_expression(
        &self,
        object_ident_name: &str,
        property_name: &str,
    ) -> Expression<'a> {
        let property = IdentifierName::new(SPAN, property_name.into());
        let ident = IdentifierReference::new(SPAN, object_ident_name.into());
        let object = self.ast.identifier_reference_expression(ident);
        self.ast.static_member_expression(SPAN, object, property, false)
    }

    /// Get the callee from `pragma` and `pragmaFrag`
    fn get_call_expression_callee(&self, literal_callee: &str) -> Expression<'a> {
        let mut callee = literal_callee.split('.');
        let member = callee.next().unwrap();
        let property = callee.next();
        property.map_or_else(
            || {
                let ident = IdentifierReference::new(SPAN, member.into());
                self.ast.identifier_reference_expression(ident)
            },
            |property_name| self.get_static_member_expression(member, property_name),
        )
    }

    fn get_create_element(
        &mut self,
        has_key_after_props_spread: bool,
        jsxs: bool,
    ) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                if self.options.pragma == "React.createElement" {
                    let object = self.get_react_references();
                    let property = IdentifierName::new(SPAN, "createElement".into());
                    return self.ast.static_member_expression(SPAN, object, property, false);
                }

                self.get_call_expression_callee(self.options.pragma.as_ref())
            }
            ReactJsxRuntime::Automatic => {
                let is_script = self.ctx.source_type().is_script();
                let name = if is_script {
                    if has_key_after_props_spread {
                        "createElement"
                    } else if jsxs {
                        "jsxs"
                    } else {
                        "jsx"
                    }
                } else if has_key_after_props_spread {
                    "_createElement"
                } else if jsxs {
                    "_jsxs"
                } else {
                    "_jsx"
                };

                if is_script {
                    let object_ident_name =
                        if has_key_after_props_spread { "_react" } else { "_reactJsxRuntime" };
                    self.get_static_member_expression(object_ident_name, name)
                } else {
                    let ident = IdentifierReference::new(SPAN, name.into());
                    self.ast.identifier_reference_expression(ident)
                }
            }
        }
    }

    fn get_fragment(&mut self) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                if self.options.pragma_frag == "React.Fragment" {
                    let object = self.get_react_references();
                    let property = IdentifierName::new(SPAN, "Fragment".into());
                    return self.ast.static_member_expression(SPAN, object, property, false);
                }

                self.get_call_expression_callee(self.options.pragma_frag.as_ref())
            }
            ReactJsxRuntime::Automatic => {
                if self.ctx.source_type().is_script() {
                    self.get_static_member_expression("_reactJsxRuntime", "Fragment")
                } else {
                    let ident = IdentifierReference::new(SPAN, "_Fragment".into());
                    self.ast.identifier_reference_expression(ident)
                }
            }
        }
    }

    fn get_attribute_name(&self, name: &JSXAttributeName<'a>) -> PropertyKey<'a> {
        match name {
            JSXAttributeName::Identifier(ident) => {
                let name = ident.name.clone();
                if ident.name.contains('-') {
                    let expr = self.ast.literal_string_expression(StringLiteral::new(SPAN, name));
                    self.ast.property_key_expression(expr)
                } else {
                    self.ast.property_key_identifier(IdentifierName::new(SPAN, name))
                }
            }
            JSXAttributeName::NamespacedName(name) => {
                let name = Atom::from(name.to_string());
                let expr = self.ast.literal_string_expression(StringLiteral::new(SPAN, name));
                self.ast.property_key_expression(expr)
            }
        }
    }

    fn transform_element_name(&self, name: &JSXElementName<'a>) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(ident) => {
                let name = ident.name.clone();
                if ident.name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                    self.ast.literal_string_expression(StringLiteral::new(SPAN, name))
                } else {
                    self.ast.identifier_reference_expression(IdentifierReference::new(SPAN, name))
                }
            }
            JSXElementName::MemberExpression(member_expr) => {
                self.transform_jsx_member_expression(member_expr)
            }
            JSXElementName::NamespacedName(name) => {
                // TODO
                // If the flag "throwIfNamespace" is false
                // print XMLNamespace like string literal
                // if self.options.throw_if_namespace.is_some_and(|v| !v) {
                // }
                let string_literal = StringLiteral::new(SPAN, Atom::from(name.to_string()));
                self.ast.literal_string_expression(string_literal)
            }
        }
    }

    fn transform_jsx_attribute_item(
        &mut self,
        properties: &mut Vec<'a, ObjectPropertyKind<'a>>,
        attribute: &JSXAttributeItem<'a>,
    ) {
        match attribute {
            JSXAttributeItem::Attribute(attr) => {
                let kind = PropertyKind::Init;
                let key = self.get_attribute_name(&attr.name);
                let value = self.transform_jsx_attribute_value(attr.value.as_ref());
                let object_property =
                    self.ast.object_property(SPAN, kind, key, value, None, false, false, false);
                let object_property = ObjectPropertyKind::ObjectProperty(object_property);
                properties.push(object_property);
            }
            JSXAttributeItem::SpreadAttribute(attr) => match &attr.argument {
                Expression::ObjectExpression(expr) if !expr.has_proto() => {
                    for object_property in &expr.properties {
                        properties.push(self.ast.copy(object_property));
                    }
                }
                expr => {
                    let argument = self.ast.copy(expr);
                    let spread_property = self.ast.spread_element(SPAN, argument);
                    let object_property = ObjectPropertyKind::SpreadProperty(spread_property);
                    properties.push(object_property);
                }
            },
        }
    }

    fn transform_jsx_attribute_value(
        &mut self,
        value: Option<&JSXAttributeValue<'a>>,
    ) -> Expression<'a> {
        match value {
            Some(JSXAttributeValue::StringLiteral(s)) => {
                let jsx_text = Self::decode_entities(s.value.as_str());
                let literal = StringLiteral::new(s.span, jsx_text.into());
                self.ast.literal_string_expression(literal)
            }
            Some(JSXAttributeValue::Element(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Element(e))
            }
            Some(JSXAttributeValue::Fragment(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e))
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => match &c.expression {
                JSXExpression::Expression(e) => self.ast.copy(e),
                JSXExpression::EmptyExpression(_e) => {
                    self.ast.literal_boolean_expression(BooleanLiteral::new(SPAN, true))
                }
            },
            None => self.ast.literal_boolean_expression(BooleanLiteral::new(SPAN, true)),
        }
    }

    fn transform_jsx_member_expression(&self, expr: &JSXMemberExpression<'a>) -> Expression<'a> {
        let object = match &expr.object {
            JSXMemberExpressionObject::Identifier(ident) => {
                self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    ident.name.clone(),
                ))
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.transform_jsx_member_expression(expr)
            }
        };
        let property = IdentifierName::new(SPAN, expr.property.name.clone());
        self.ast.static_member_expression(SPAN, object, property, false)
    }

    fn transform_jsx_child(&mut self, child: &JSXChild<'a>) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => self.transform_jsx_text(text.value.as_str()),
            JSXChild::ExpressionContainer(e) => match &e.expression {
                JSXExpression::Expression(e) => Some(self.ast.copy(e)),
                JSXExpression::EmptyExpression(_) => None,
            },
            JSXChild::Element(e) => Some(self.transform_jsx(&JSXElementOrFragment::Element(e))),
            JSXChild::Fragment(e) => Some(self.transform_jsx(&JSXElementOrFragment::Fragment(e))),
            JSXChild::Spread(_) => {
                // Babel: Spread children are not supported in React.
                None
            }
        }
    }

    fn transform_jsx_text(&self, text: &str) -> Option<Expression<'a>> {
        Self::fixup_whitespace_and_decode_entities(text).map(|s| {
            let s = StringLiteral::new(SPAN, s.into());
            self.ast.literal_string_expression(s)
        })
    }

    /// JSX trims whitespace at the end and beginning of lines, except that the
    /// start/end of a tag is considered a start/end of a line only if that line is
    /// on the same line as the closing tag. See examples in
    /// tests/cases/conformance/jsx/tsxReactEmitWhitespace.tsx
    /// See also https://www.w3.org/TR/html4/struct/text.html#h-9.1 and https://www.w3.org/TR/CSS2/text.html#white-space-model
    ///
    /// An equivalent algorithm would be:
    /// - If there is only one line, return it.
    /// - If there is only whitespace (but multiple lines), return `undefined`.
    /// - Split the text into lines.
    /// - 'trimRight' the first line, 'trimLeft' the last line, 'trim' middle lines.
    /// - Decode entities on each line (individually).
    /// - Remove empty lines and join the rest with " ".
    ///
    /// <https://github.com/microsoft/TypeScript/blob/f0374ce2a9c465e27a15b7fa4a347e2bd9079450/src/compiler/transformers/jsx.ts#L557-L608>
    fn fixup_whitespace_and_decode_entities(text: &str) -> Option<String> {
        let mut acc: Option<String> = None;
        let mut first_non_whitespace: Option<usize> = Some(0);
        let mut last_non_whitespace: Option<usize> = None;
        let mut i: usize = 0;
        for c in text.chars() {
            if is_line_terminator(c) {
                if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                    acc = Some(Self::add_line_of_jsx_text(acc, &text[first..=last]));
                }
                first_non_whitespace = None;
            } else if c != ' ' && !is_irregular_whitespace(c) {
                last_non_whitespace = Some(i);
                if first_non_whitespace.is_none() {
                    first_non_whitespace.replace(i);
                }
            }
            i += c.len_utf8();
        }
        if let Some(first) = first_non_whitespace {
            Some(Self::add_line_of_jsx_text(acc, &text[first..]))
        } else {
            acc
        }
    }

    fn add_line_of_jsx_text(acc: Option<String>, trimmed_line: &str) -> String {
        let decoded = Self::decode_entities(trimmed_line);
        if let Some(acc) = acc {
            format!("{acc} {decoded}")
        } else {
            decoded
        }
    }

    /// * Replace entities like "&nbsp;", "&#123;", and "&#xDEADBEEF;" with the characters they encode.
    /// * See https://en.wikipedia.org/wiki/List_of_XML_and_HTML_character_entity_references
    /// Code adapted from <https://github.com/microsoft/TypeScript/blob/514f7e639a2a8466c075c766ee9857a30ed4e196/src/compiler/transformers/jsx.ts#L617C1-L635>
    fn decode_entities(s: &str) -> String {
        let mut buffer = vec![];
        let mut chars = s.bytes().enumerate();
        let mut prev = 0;
        while let Some((i, c)) = chars.next() {
            if c == b'&' {
                let start = i;
                let mut end = None;
                for (j, c) in chars.by_ref() {
                    if c == b';' {
                        end.replace(j);
                        break;
                    }
                }
                if let Some(end) = end {
                    let word = &s[start + 1..end];
                    buffer.extend_from_slice(s[prev..start].as_bytes());
                    prev = end + 1;
                    if let Some(c) = XML_ENTITIES.get(word) {
                        buffer.extend_from_slice(c.to_string().as_bytes());
                    }
                }
            }
        }
        buffer.extend_from_slice(s[prev..].as_bytes());
        // Safety: The buffer is constructed from valid utf chars.
        unsafe { String::from_utf8_unchecked(buffer) }
    }
}
