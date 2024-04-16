mod diagnostics;

use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{CompactStr, GetSpan, SPAN};
use oxc_syntax::{
    identifier::{is_irregular_whitespace, is_line_terminator},
    xml_entities::XML_ENTITIES,
};

use crate::{context::Ctx, helpers::module_imports::NamedImport};

pub use super::{
    jsx_self::ReactJsxSelf,
    jsx_source::ReactJsxSource,
    options::{ReactJsxRuntime, ReactOptions},
};

use self::diagnostics::{
    ImportSourceCannotBeSet, NamespaceDoesNotSupport, PragmaAndPragmaFragCannotBeSet,
    SpreadChildrenAreNotSupported, ValuelessKey,
};

/// [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
///
/// This plugin generates production-ready JS code.
///
/// This plugin is included in `preset-react`.
///
/// References:
///
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
pub struct ReactJsx<'a> {
    options: Rc<ReactOptions>,

    ctx: Ctx<'a>,

    pub(super) jsx_self: ReactJsxSelf<'a>,
    pub(super) jsx_source: ReactJsxSource<'a>,

    // States
    require_jsx_runtime: bool,
    jsx_runtime_importer: CompactStr,
    default_runtime: ReactJsxRuntime,

    import_jsx: bool,
    import_jsxs: bool,
    import_fragment: bool,
    import_create_element: bool,
}

// Transforms
impl<'a> ReactJsx<'a> {
    pub fn new(options: &Rc<ReactOptions>, ctx: &Ctx<'a>) -> Self {
        let default_runtime = options.runtime;
        let jsx_runtime_importer =
            if options.import_source == "react" || default_runtime.is_classic() {
                CompactStr::from("react/jsx-runtime")
            } else {
                CompactStr::from(format!("{}/jsx-runtime", options.import_source))
            };

        Self {
            options: Rc::clone(options),
            ctx: Rc::clone(ctx),
            jsx_self: ReactJsxSelf::new(ctx),
            jsx_source: ReactJsxSource::new(ctx),
            require_jsx_runtime: false,
            jsx_runtime_importer,
            import_jsx: false,
            import_jsxs: false,
            import_fragment: false,
            import_create_element: false,
            default_runtime,
        }
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        self.add_runtime_imports(program);
    }

    pub fn transform_jsx_element(&mut self, e: &JSXElement<'a>) -> Expression<'a> {
        self.transform_jsx(&JSXElementOrFragment::Element(e))
    }

    pub fn transform_jsx_fragment(&mut self, e: &JSXFragment<'a>) -> Expression<'a> {
        self.transform_jsx(&JSXElementOrFragment::Fragment(e))
    }

    fn is_script(&self) -> bool {
        self.ctx.semantic.source_type().is_script()
    }

    fn ast(&self) -> &AstBuilder<'a> {
        &self.ctx.ast
    }
}

// Add imports
impl<'a> ReactJsx<'a> {
    pub fn add_runtime_imports(&mut self, program: &mut Program<'a>) {
        if self.options.runtime.is_classic() {
            if self.options.import_source != "react" {
                self.ctx.error(ImportSourceCannotBeSet);
            }
            return;
        }

        if self.options.pragma != "React.createElement"
            || self.options.pragma_frag != "React.Fragment"
        {
            self.ctx.error(PragmaAndPragmaFragCannotBeSet);
            return;
        }

        let imports = self.ctx.module_imports.get_import_statements();
        let index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ModuleDeclaration(m) if m.is_import()))
            .map_or(0, |i| i + 1);
        program.body.splice(index..index, imports);
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
            JSXElementOrFragment::Fragment(_) => {
                self.add_import_fragment();
                if need_jsxs {
                    self.add_import_jsxs();
                }
            }
        }
    }

    fn add_require_jsx_runtime(&mut self) {
        if !self.require_jsx_runtime {
            self.require_jsx_runtime = true;
            self.add_require_statement(
                "_reactJsxRuntime",
                self.jsx_runtime_importer.clone(),
                false,
            );
        }
    }

    fn add_import_jsx(&mut self) {
        if self.is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_jsx {
            self.import_jsx = true;
            self.add_import_statement("jsx", "_jsx", self.jsx_runtime_importer.clone());
        }
    }

    fn add_import_jsxs(&mut self) {
        if self.is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_jsxs {
            self.import_jsxs = true;
            self.add_import_statement("jsxs", "_jsxs", self.jsx_runtime_importer.clone());
        }
    }

    fn add_import_fragment(&mut self) {
        if self.is_script() {
            self.add_require_jsx_runtime();
        } else if !self.import_fragment {
            self.import_fragment = true;
            self.add_import_statement("Fragment", "_Fragment", self.jsx_runtime_importer.clone());
            self.add_import_jsx();
        }
    }

    fn add_import_create_element(&mut self) {
        if !self.import_create_element {
            self.import_create_element = true;
            let source = self.options.import_source.as_ref();
            if self.is_script() {
                self.add_require_statement("_react", source.into(), true);
            } else {
                self.add_import_statement("createElement", "_createElement", source.into());
            }
        }
    }

    fn add_import_statement(&mut self, imported: &str, local: &str, source: CompactStr) {
        let import = NamedImport::new(imported.into(), Some(local.into()));
        self.ctx.module_imports.add_import(source, import);
    }

    fn add_require_statement(&mut self, variable_name: &str, source: CompactStr, front: bool) {
        let import = NamedImport::new(variable_name.into(), None);
        self.ctx.module_imports.add_require(source, import, front);
    }
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

// Transform jsx
impl<'a> ReactJsx<'a> {
    fn transform_jsx<'b>(&mut self, e: &JSXElementOrFragment<'a, 'b>) -> Expression<'a> {
        let is_classic = self.default_runtime.is_classic();
        let is_automatic = self.default_runtime.is_automatic();
        let has_key_after_props_spread = e.has_key_after_props_spread();

        let mut arguments = self.ast().new_vec();
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
            let null_expr = self.ast().literal_null_expression(NullLiteral::new(SPAN));
            arguments.push(Argument::Expression(null_expr));
        }

        // The object properties for the second argument of `React.createElement`
        let mut properties = self.ast().new_vec();

        if let Some(attributes) = attributes {
            for attribute in attributes {
                match attribute {
                    // optimize `{...prop}` to `prop` in static mode
                    JSXAttributeItem::SpreadAttribute(spread)
                        if is_classic && attributes_len == 1 =>
                    {
                        // deopt if spreading an object with `__proto__` key
                        if !matches!(&spread.argument, Expression::ObjectExpression(o) if o.has_proto())
                        {
                            arguments.push(Argument::Expression(self.ast().copy(&spread.argument)));
                            continue;
                        }
                    }
                    JSXAttributeItem::Attribute(attr) if attr.is_key() => {
                        if attr.value.is_none() {
                            self.ctx.error(ValuelessKey(attr.name.span()));
                        }
                        // In automatic mode, extract the key before spread prop,
                        // and add it to the third argument later.
                        if is_automatic && !has_key_after_props_spread {
                            key_prop = attr.value.as_ref();
                            continue;
                        }
                    }
                    _ => {}
                }

                // Add attribute to prop object
                self.transform_jsx_attribute_item(&mut properties, attribute);
            }
        }

        let mut need_jsxs = false;

        let children = e.children();

        // Append children to object properties in automatic mode
        if is_automatic {
            let allocator = self.ast().allocator;
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
                    self.ast().array_expression(SPAN, elements, None)
                };
                let object_property = {
                    let kind = PropertyKind::Init;
                    let ident = IdentifierName::new(SPAN, "children".into());
                    let key = self.ast().property_key_identifier(ident);
                    self.ast().object_property(SPAN, kind, key, value, None, false, false, false)
                };
                properties.push(ObjectPropertyKind::ObjectProperty(object_property));
            }
        }

        if self.options.is_jsx_self_plugin_enabled() {
            properties.push(self.jsx_self.get_object_property_kind_for_jsx_plugin());
        }
        if self.options.is_jsx_source_plugin_enabled() {
            properties.push(self.jsx_source.get_object_property_kind_for_jsx_plugin());
        }

        self.add_import(e, has_key_after_props_spread, need_jsxs);

        if !properties.is_empty() || is_automatic {
            let object_expression = self.ast().object_expression(SPAN, properties, None);
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
        self.ast().call_expression(SPAN, callee, arguments, false, None)
    }

    fn transform_element_name(&self, name: &JSXElementName<'a>) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(ident) => {
                if ident.name == "this" {
                    self.ast().this_expression(SPAN)
                } else if ident.name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                    let string = StringLiteral::new(SPAN, ident.name.clone());
                    self.ast().literal_string_expression(string)
                } else {
                    let ident = IdentifierReference::new(SPAN, ident.name.clone());
                    self.ctx.ast.identifier_reference_expression(ident)
                }
            }
            JSXElementName::MemberExpression(member_expr) => {
                self.transform_jsx_member_expression(member_expr)
            }
            JSXElementName::NamespacedName(name) => {
                if self.options.throw_if_namespace {
                    self.ctx.error(NamespaceDoesNotSupport(name.span));
                }
                let name = self.ast().new_atom(&name.to_string());
                let string_literal = StringLiteral::new(SPAN, name);
                self.ast().literal_string_expression(string_literal)
            }
        }
    }

    fn get_fragment(&self) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                if self.options.pragma_frag == "React.Fragment" {
                    let object = self.get_react_references();
                    let property = IdentifierName::new(SPAN, "Fragment".into());
                    self.ast().static_member_expression(SPAN, object, property, false)
                } else {
                    self.get_call_expression_callee(self.options.pragma_frag.as_ref())
                }
            }
            ReactJsxRuntime::Automatic => {
                if self.is_script() {
                    self.get_static_member_expression("_reactJsxRuntime", "Fragment")
                } else {
                    let ident = IdentifierReference::new(SPAN, "_Fragment".into());
                    self.ast().identifier_reference_expression(ident)
                }
            }
        }
    }

    fn get_create_element(&self, has_key_after_props_spread: bool, jsxs: bool) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                if self.options.pragma == "React.createElement" {
                    let object = self.get_react_references();
                    let property = IdentifierName::new(SPAN, "createElement".into());
                    self.ast().static_member_expression(SPAN, object, property, false)
                } else {
                    self.get_call_expression_callee(self.options.pragma.as_ref())
                }
            }
            ReactJsxRuntime::Automatic => {
                let name = if self.is_script() {
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
                if self.is_script() {
                    let object_ident_name =
                        if has_key_after_props_spread { "_react" } else { "_reactJsxRuntime" };
                    self.get_static_member_expression(object_ident_name, name)
                } else {
                    let ident = IdentifierReference::new(SPAN, name.into());
                    self.ast().identifier_reference_expression(ident)
                }
            }
        }
    }

    fn get_react_references(&self) -> Expression<'a> {
        let ident = IdentifierReference::new(SPAN, "React".into());
        self.ast().identifier_reference_expression(ident)
    }

    fn get_static_member_expression(
        &self,
        object_ident_name: &str,
        property_name: &str,
    ) -> Expression<'a> {
        let property = IdentifierName::new(SPAN, self.ast().new_atom(property_name));
        let ident = IdentifierReference::new(SPAN, self.ast().new_atom(object_ident_name));
        let object = self.ast().identifier_reference_expression(ident);
        self.ast().static_member_expression(SPAN, object, property, false)
    }

    /// Get the callee from `pragma` and `pragmaFrag`
    fn get_call_expression_callee(&self, literal_callee: &str) -> Expression<'a> {
        let mut callee = literal_callee.split('.');
        let member = callee.next().unwrap();
        let property = callee.next();
        property.map_or_else(
            || {
                let ident = IdentifierReference::new(SPAN, self.ast().new_atom(member));
                self.ast().identifier_reference_expression(ident)
            },
            |property_name| self.get_static_member_expression(member, property_name),
        )
    }

    fn transform_jsx_member_expression(&self, expr: &JSXMemberExpression<'a>) -> Expression<'a> {
        let object = match &expr.object {
            JSXMemberExpressionObject::Identifier(ident) => {
                let ident = IdentifierReference::new(SPAN, ident.name.clone());
                self.ast().identifier_reference_expression(ident)
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.transform_jsx_member_expression(expr)
            }
        };
        let property = IdentifierName::new(SPAN, expr.property.name.clone());
        self.ast().static_member_expression(SPAN, object, property, false)
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
                    self.ast().object_property(SPAN, kind, key, value, None, false, false, false);
                let object_property = ObjectPropertyKind::ObjectProperty(object_property);
                properties.push(object_property);
            }
            JSXAttributeItem::SpreadAttribute(attr) => match &attr.argument {
                Expression::ObjectExpression(expr) if !expr.has_proto() => {
                    properties.extend(self.ast().copy(&expr.properties));
                }
                expr => {
                    let argument = self.ast().copy(expr);
                    let spread_property = self.ast().spread_element(SPAN, argument);
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
                let literal = StringLiteral::new(s.span, self.ast().new_atom(&jsx_text));
                self.ast().literal_string_expression(literal)
            }
            Some(JSXAttributeValue::Element(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Element(e))
            }
            Some(JSXAttributeValue::Fragment(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e))
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => match &c.expression {
                JSXExpression::Expression(e) => self.ast().copy(e),
                JSXExpression::EmptyExpression(_e) => {
                    self.ast().literal_boolean_expression(BooleanLiteral::new(SPAN, true))
                }
            },
            None => self.ast().literal_boolean_expression(BooleanLiteral::new(SPAN, true)),
        }
    }

    fn transform_jsx_child(&mut self, child: &JSXChild<'a>) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => self.transform_jsx_text(text.value.as_str()),
            JSXChild::ExpressionContainer(e) => match &e.expression {
                JSXExpression::Expression(e) => Some(self.ast().copy(e)),
                JSXExpression::EmptyExpression(_) => None,
            },
            JSXChild::Element(e) => Some(self.transform_jsx(&JSXElementOrFragment::Element(e))),
            JSXChild::Fragment(e) => Some(self.transform_jsx(&JSXElementOrFragment::Fragment(e))),
            JSXChild::Spread(e) => {
                self.ctx.error(SpreadChildrenAreNotSupported(e.span));
                None
            }
        }
    }

    fn get_attribute_name(&self, name: &JSXAttributeName<'a>) -> PropertyKey<'a> {
        match name {
            JSXAttributeName::Identifier(ident) => {
                let name = ident.name.clone();
                if ident.name.contains('-') {
                    let expr = self.ast().literal_string_expression(StringLiteral::new(SPAN, name));
                    self.ast().property_key_expression(expr)
                } else {
                    self.ast().property_key_identifier(IdentifierName::new(SPAN, name))
                }
            }
            JSXAttributeName::NamespacedName(name) => {
                let name = self.ast().new_atom(&name.to_string());
                let expr = self.ast().literal_string_expression(StringLiteral::new(SPAN, name));
                self.ast().property_key_expression(expr)
            }
        }
    }

    fn transform_jsx_text(&self, text: &str) -> Option<Expression<'a>> {
        Self::fixup_whitespace_and_decode_entities(text).map(|s| {
            let s = StringLiteral::new(SPAN, self.ast().new_atom(&s));
            self.ast().literal_string_expression(s)
        })
    }

    /// JSX trims whitespace at the end and beginning of lines, except that the
    /// start/end of a tag is considered a start/end of a line only if that line is
    /// on the same line as the closing tag. See examples in
    /// tests/cases/conformance/jsx/tsxReactEmitWhitespace.tsx
    /// See also <https://www.w3.org/TR/html4/struct/text.html#h-9.1> and <https://www.w3.org/TR/CSS2/text.html#white-space-model>
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
        for (index, c) in text.char_indices() {
            if is_line_terminator(c) {
                if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                    acc = Some(Self::add_line_of_jsx_text(acc, &text[first..last]));
                }
                first_non_whitespace = None;
            } else if c != ' ' && !is_irregular_whitespace(c) {
                last_non_whitespace = Some(index + c.len_utf8());
                if first_non_whitespace.is_none() {
                    first_non_whitespace.replace(index);
                }
            }
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

    /// Replace entities like "&nbsp;", "&#123;", and "&#xDEADBEEF;" with the characters they encode.
    /// * See <https://en.wikipedia.org/wiki/List_of_XML_and_HTML_character_entity_references>
    /// Code adapted from <https://github.com/microsoft/TypeScript/blob/514f7e639a2a8466c075c766ee9857a30ed4e196/src/compiler/transformers/jsx.ts#L617C1-L635>
    fn decode_entities(s: &str) -> String {
        let mut buffer = vec![];
        let mut chars = s.char_indices();
        let mut prev = 0;
        while let Some((i, c)) = chars.next() {
            if c == '&' {
                let start = i;
                let mut end = None;
                for (j, c) in chars.by_ref() {
                    if c == ';' {
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
        #[allow(unsafe_code)]
        // SAFETY: The buffer is constructed from valid utf chars.
        unsafe {
            String::from_utf8_unchecked(buffer)
        }
    }
}
