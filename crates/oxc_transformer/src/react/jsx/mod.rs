mod diagnostics;

use std::{cell::Cell, rc::Rc};

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, CompactStr, GetSpan, Span, SPAN};
use oxc_syntax::{
    identifier::{is_irregular_whitespace, is_line_terminator},
    reference::{ReferenceFlag, ReferenceId},
    symbol::{SymbolFlags, SymbolId},
    xml_entities::XML_ENTITIES,
};
use oxc_traverse::TraverseCtx;

use crate::{context::Ctx, helpers::module_imports::NamedImport};

use super::utils::get_line_column;
pub use super::{
    jsx_self::ReactJsxSelf,
    jsx_source::ReactJsxSource,
    options::{ReactJsxRuntime, ReactOptions},
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
    jsx_runtime_importer: Atom<'a>,

    // Doubles as var name for require react
    import_create_element: Option<BoundIdentifier<'a>>,
    // Doubles as var name for require JSX
    import_jsx: Option<BoundIdentifier<'a>>,
    import_jsxs: Option<BoundIdentifier<'a>>,
    import_fragment: Option<BoundIdentifier<'a>>,

    pragma: Option<Pragma<'a>>,
    pragma_frag: Option<Pragma<'a>>,

    can_add_filename_statement: bool,
}

pub struct BoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: SymbolId,
}

impl<'a> BoundIdentifier<'a> {
    /// Create `IdentifierReference` referencing this binding which is read from
    fn create_read_reference(&self, ctx: &mut TraverseCtx) -> IdentifierReference<'a> {
        let reference_id = ctx.create_bound_reference(
            self.name.to_compact_str(),
            self.symbol_id,
            ReferenceFlag::Read,
        );
        create_read_identifier_reference(SPAN, self.name.clone(), Some(reference_id))
    }
}

struct Pragma<'a> {
    object: Atom<'a>,
    property: Option<Atom<'a>>,
}

impl<'a> Pragma<'a> {
    fn parse(pragma: &str, ast: &AstBuilder<'a>) -> Self {
        let mut parts = pragma.split('.');
        let object = ast.new_atom(parts.next().unwrap());
        let property = parts.next().map(|property| {
            assert!(parts.next().is_none(), "Invalid pragma");
            ast.new_atom(property)
        });
        Self { object, property }
    }

    fn create_expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let object = get_read_identifier_reference(SPAN, self.object.clone(), ctx);
        if let Some(property) = self.property.as_ref() {
            create_static_member_expression(object, property.clone(), ctx)
        } else {
            ctx.ast.identifier_reference_expression(object)
        }
    }
}

// Transforms
impl<'a> ReactJsx<'a> {
    pub fn new(options: &Rc<ReactOptions>, ctx: &Ctx<'a>) -> Self {
        let default_runtime = options.runtime;
        let jsx_runtime_importer =
            if options.import_source == "react" || default_runtime.is_classic() {
                let source =
                    if options.development { "react/jsx-dev-runtime" } else { "react/jsx-runtime" };
                Atom::from(source)
            } else {
                ctx.ast.new_atom(&format!(
                    "{}/jsx-{}runtime",
                    options.import_source,
                    if options.development { "dev-" } else { "" }
                ))
            };

        // Parse pragmas
        let (pragma, pragma_frag) = match options.runtime {
            ReactJsxRuntime::Classic => {
                let pragma = Pragma::parse(&options.pragma, &ctx.ast);
                let pragma_frag = Pragma::parse(&options.pragma_frag, &ctx.ast);
                (Some(pragma), Some(pragma_frag))
            }
            ReactJsxRuntime::Automatic => (None, None),
        };

        Self {
            options: Rc::clone(options),
            ctx: Rc::clone(ctx),
            jsx_self: ReactJsxSelf::new(ctx),
            jsx_source: ReactJsxSource::new(ctx),
            jsx_runtime_importer,
            import_create_element: None,
            import_jsx: None,
            import_jsxs: None,
            import_fragment: None,
            pragma,
            pragma_frag,
            can_add_filename_statement: false,
        }
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        self.add_runtime_imports(program);
    }

    pub fn transform_jsx_element(
        &mut self,
        e: &JSXElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.transform_jsx(&JSXElementOrFragment::Element(e), ctx)
    }

    pub fn transform_jsx_fragment(
        &mut self,
        e: &JSXFragment<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx)
    }

    fn is_script(&self) -> bool {
        self.ctx.source_type.is_script()
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
                self.ctx.error(diagnostics::import_source_cannot_be_set());
            }

            if self.can_add_filename_statement {
                program.body.insert(0, self.jsx_source.get_var_file_name_statement());
            }

            return;
        }

        if self.options.pragma != "React.createElement"
            || self.options.pragma_frag != "React.Fragment"
        {
            self.ctx.error(diagnostics::pragma_and_pragma_frag_cannot_be_set());
            return;
        }

        let imports = self.ctx.module_imports.get_import_statements();
        let mut index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ImportDeclaration(_)))
            .map_or(0, |i| i + 1);

        if self.can_add_filename_statement {
            program.body.insert(index, self.jsx_source.get_var_file_name_statement());
            // If source type is module then we need to add the import statement after the var file name statement
            // Follow the same behavior as babel
            if !self.is_script() {
                index += 1;
            }
        }

        program.body.splice(index..index, imports);
    }

    fn add_import<'b>(
        &mut self,
        e: &JSXElementOrFragment<'a, 'b>,
        has_key_after_props_spread: bool,
        need_jsxs: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.runtime.is_classic() {
            return;
        }
        match e {
            JSXElementOrFragment::Element(_) if has_key_after_props_spread => {
                self.add_import_create_element(ctx);
            }
            JSXElementOrFragment::Element(_) if need_jsxs => self.add_import_jsxs(ctx),
            JSXElementOrFragment::Element(_) => self.add_import_jsx(ctx),
            JSXElementOrFragment::Fragment(_) => {
                self.add_import_fragment(ctx);
                if need_jsxs {
                    self.add_import_jsxs(ctx);
                }
            }
        }
    }

    fn add_require_jsx_runtime(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.import_jsx.is_none() {
            let var_name =
                if self.options.development { "reactJsxDevRuntime" } else { "reactJsxRuntime" };
            let id =
                self.add_require_statement(var_name, self.jsx_runtime_importer.clone(), false, ctx);
            self.import_jsx = Some(id);
        }
    }

    fn add_import_jsx(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.is_script() {
            self.add_require_jsx_runtime(ctx);
        } else if self.options.development {
            self.add_import_jsx_dev(ctx);
        } else if self.import_jsx.is_none() {
            let id = self.add_import_statement("jsx", self.jsx_runtime_importer.clone(), ctx);
            self.import_jsx = Some(id);
        }
    }

    fn add_import_jsxs(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.is_script() {
            self.add_require_jsx_runtime(ctx);
        } else if self.options.development {
            self.add_import_jsx_dev(ctx);
        } else if self.import_jsxs.is_none() {
            let id = self.add_import_statement("jsxs", self.jsx_runtime_importer.clone(), ctx);
            self.import_jsxs = Some(id);
        }
    }

    fn add_import_jsx_dev(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.is_script() {
            self.add_require_jsx_runtime(ctx);
        } else if self.import_jsx.is_none() {
            let id = self.add_import_statement("jsxDEV", self.jsx_runtime_importer.clone(), ctx);
            self.import_jsx = Some(id);
        }
    }

    fn add_import_fragment(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.is_script() {
            self.add_require_jsx_runtime(ctx);
        } else if self.import_fragment.is_none() {
            let id = self.add_import_statement("Fragment", self.jsx_runtime_importer.clone(), ctx);
            self.import_fragment = Some(id);
            self.add_import_jsx(ctx);
        }
    }

    fn add_import_create_element(&mut self, ctx: &mut TraverseCtx<'a>) {
        if self.import_create_element.is_none() {
            let source = ctx.ast.new_atom(&self.options.import_source);
            let id = if self.is_script() {
                self.add_require_statement("react", source, true, ctx)
            } else {
                self.add_import_statement("createElement", source, ctx)
            };
            self.import_create_element = Some(id);
        }
    }

    fn add_import_statement(
        &mut self,
        name: &'static str,
        source: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let root_scope_id = ctx.scopes().root_scope_id();
        let symbol_id = ctx.generate_uid(name, root_scope_id, SymbolFlags::FunctionScopedVariable);
        let local = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);

        let import = NamedImport::new(Atom::from(name), Some(local.clone()), symbol_id);
        self.ctx.module_imports.add_import(source, import);
        BoundIdentifier { name: local, symbol_id }
    }

    fn add_require_statement(
        &mut self,
        variable_name: &str,
        source: Atom<'a>,
        front: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let root_scope_id = ctx.scopes().root_scope_id();
        let symbol_id =
            ctx.generate_uid(variable_name, root_scope_id, SymbolFlags::FunctionScopedVariable);
        let variable_name = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);

        let import = NamedImport::new(variable_name.clone(), None, symbol_id);
        self.ctx.module_imports.add_require(source, import, front);
        BoundIdentifier { name: variable_name, symbol_id }
    }
}

enum JSXElementOrFragment<'a, 'b> {
    Element(&'b JSXElement<'a>),
    Fragment(&'b JSXFragment<'a>),
}

impl<'a, 'b> JSXElementOrFragment<'a, 'b> {
    fn span(&self) -> Span {
        match self {
            Self::Element(e) => e.span(),
            Self::Fragment(e) => e.span,
        }
    }

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

    fn is_fragment(&self) -> bool {
        matches!(self, Self::Fragment(_))
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
    /// ## Automatic
    /// ### Element
    /// Builds JSX into:
    /// - Production: React.jsx(type, arguments, key)
    /// - Development: React.jsxDEV(type, arguments, key, isStaticChildren, source, self)
    ///
    /// ### Fragment
    /// Builds JSX Fragment <></> into
    /// - Production: React.jsx(type, arguments)
    /// - Development: React.jsxDEV(type, { children })
    ///
    /// ## Classic
    /// ### Element
    /// - Production: React.createElement(type, arguments, children)
    /// - Development: React.createElement(type, arguments, children, source, self)
    ///
    /// ### Fragment
    /// React.createElement(React.Fragment, null, ...children)
    ///
    fn transform_jsx<'b>(
        &mut self,
        e: &JSXElementOrFragment<'a, 'b>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let is_fragment = e.is_fragment();
        let has_key_after_props_spread = e.has_key_after_props_spread();
        // If has_key_after_props_spread is true, we need to fallback to `createElement` same behavior as classic runtime
        let is_classic = self.options.runtime.is_classic() || has_key_after_props_spread;
        let is_automatic = !is_classic;
        let is_development = self.options.development;

        let mut arguments = self.ast().new_vec();
        let (argument_expr, fragment_needs_update) = match e {
            JSXElementOrFragment::Element(e) => {
                (self.transform_element_name(&e.opening_element.name, ctx), false)
            }
            JSXElementOrFragment::Fragment(_) => self.get_fragment(ctx),
        };
        arguments.push(Argument::from(argument_expr));

        // The key prop in `<div key={true} />`
        let mut key_prop = None;

        let attributes = e.attributes();
        let attributes_len = attributes.map_or(0, |attrs| attrs.len());

        // The object properties for the second argument of `React.createElement`
        let mut properties = self.ast().new_vec();

        let mut self_attr_span = None;
        let mut source_attr_span = None;

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
                            arguments.push(Argument::from(self.ast().copy(&spread.argument)));
                            continue;
                        }
                    }
                    JSXAttributeItem::Attribute(attr) => {
                        if attr.is_identifier("__self") {
                            self_attr_span = Some(attr.name.span());
                        } else if attr.is_identifier("__source") {
                            source_attr_span = Some(attr.name.span());
                        }

                        if attr.is_key() {
                            if attr.value.is_none() {
                                self.ctx.error(diagnostics::valueless_key(attr.name.span()));
                            }
                            // In automatic mode, extract the key before spread prop,
                            // and add it to the third argument later.
                            if is_automatic {
                                key_prop = attr.value.as_ref();
                                continue;
                            }
                        }
                    }
                    JSXAttributeItem::SpreadAttribute(_) => {}
                }

                // Add attribute to prop object
                self.transform_jsx_attribute_item(&mut properties, attribute, ctx);
            }
        }

        let mut need_jsxs = false;

        let children = e.children();

        // Append children to object properties in automatic mode
        if is_automatic {
            let allocator = self.ast().allocator;
            let mut children = Vec::from_iter_in(
                children.iter().filter_map(|child| self.transform_jsx_child(child, ctx)),
                allocator,
            );
            let children_len = children.len();
            if children_len != 0 {
                let value = if children_len == 1 {
                    children.pop().unwrap()
                } else {
                    let elements = Vec::from_iter_in(
                        children.into_iter().map(ArrayExpressionElement::from),
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

        // React.createElement's second argument
        if !is_fragment && is_classic {
            if self.options.is_jsx_self_plugin_enabled()
                && self.jsx_self.can_add_self_attribute(ctx)
            {
                if let Some(span) = self_attr_span {
                    self.jsx_self.report_error(span);
                } else {
                    properties.push(self.jsx_self.get_object_property_kind_for_jsx_plugin());
                }
            }

            if self.options.is_jsx_source_plugin_enabled() {
                if let Some(span) = source_attr_span {
                    self.jsx_source.report_error(span);
                } else {
                    self.can_add_filename_statement = true;
                    let (line, column) = get_line_column(e.span().start, self.ctx.source_text);
                    properties.push(
                        self.jsx_source.get_object_property_kind_for_jsx_plugin(line, column),
                    );
                }
            }
        }

        self.add_import(e, has_key_after_props_spread, need_jsxs, ctx);

        if fragment_needs_update {
            self.update_fragment(arguments.first_mut().unwrap(), ctx);
        }

        // If runtime is automatic that means we always to add `{ .. }` as the second argument even if it's empty
        if is_automatic || !properties.is_empty() {
            let object_expression = self.ast().object_expression(SPAN, properties, None);
            arguments.push(Argument::from(object_expression));
        } else if arguments.len() == 1 {
            // If not and second argument doesn't exist, we should add `null` as the second argument
            let null_expr = self.ast().literal_null_expression(NullLiteral::new(SPAN));
            arguments.push(Argument::from(null_expr));
        }

        // Only jsx and jsxDev will have more than 2 arguments
        if is_automatic {
            // key
            if key_prop.is_some() {
                arguments.push(Argument::from(self.transform_jsx_attribute_value(key_prop, ctx)));
            } else if is_development {
                arguments.push(Argument::from(self.ctx.ast.void_0()));
            }

            // isStaticChildren
            if is_development {
                let literal = self
                    .ctx
                    .ast
                    .boolean_literal(SPAN, if is_fragment { false } else { children.len() > 1 });
                arguments.push(Argument::from(self.ctx.ast.literal_boolean_expression(literal)));
            }

            // Fragment doesn't have source and self
            if !is_fragment {
                // { __source: { fileName, lineNumber, columnNumber } }
                if self.options.is_jsx_source_plugin_enabled() {
                    if let Some(span) = source_attr_span {
                        self.jsx_source.report_error(span);
                    } else {
                        self.can_add_filename_statement = true;
                        let (line, column) = get_line_column(e.span().start, self.ctx.source_text);
                        let expr = self.jsx_source.get_source_object(line, column);
                        arguments.push(Argument::from(expr));
                    }
                }

                // this
                if self.options.is_jsx_self_plugin_enabled()
                    && self.jsx_self.can_add_self_attribute(ctx)
                {
                    if let Some(span) = self_attr_span {
                        self.jsx_self.report_error(span);
                    } else {
                        arguments.push(Argument::from(self.ctx.ast.this_expression(SPAN)));
                    }
                }
            }
        } else {
            // React.createElement(type, arguments, ...children)
            //                                      ^^^^^^^^^^^
            arguments.extend(
                children
                    .iter()
                    .filter_map(|child| self.transform_jsx_child(child, ctx))
                    .map(Argument::from),
            );
        }

        let callee = self.get_create_element(has_key_after_props_spread, need_jsxs, ctx);
        self.ast().call_expression(SPAN, callee, arguments, false, None)
    }

    fn transform_element_name(
        &self,
        name: &JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(ident) => {
                if ident.name == "this" {
                    self.ast().this_expression(SPAN)
                } else if ident.name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                    let string = StringLiteral::new(SPAN, ident.name.clone());
                    self.ast().literal_string_expression(string)
                } else {
                    let ident = get_read_identifier_reference(ident.span, ident.name.clone(), ctx);
                    self.ctx.ast.identifier_reference_expression(ident)
                }
            }
            JSXElementName::MemberExpression(member_expr) => {
                self.transform_jsx_member_expression(member_expr, ctx)
            }
            JSXElementName::NamespacedName(name) => {
                if self.options.throw_if_namespace {
                    self.ctx.error(diagnostics::namespace_does_not_support(name.span));
                }
                let name = self.ast().new_atom(&name.to_string());
                let string_literal = StringLiteral::new(SPAN, name);
                self.ast().literal_string_expression(string_literal)
            }
        }
    }

    /// Create fragment expression.
    /// `bool` returned is flag for whether identifier is temporary and `update_fragment`
    /// needs to be called later.
    fn get_fragment(&mut self, ctx: &mut TraverseCtx<'a>) -> (Expression<'a>, bool) {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                let expr = self.pragma_frag.as_ref().unwrap().create_expression(ctx);
                (expr, false) // false = does not need update
            }
            ReactJsxRuntime::Automatic => {
                // Use existing import if exists. Otherwise create temporary identifiers,
                // and signal to over-write them later in `update_fragment` after import is added
                // and correct var name is known. Correct `reference_id` will also be set then.
                // We have to do like this so that imports are in same order as Babel's output,
                // in order to pass Babel's tests.
                // TODO(improve-on-babel): Remove this workaround if output doesn't need to match
                // Babel's exactly.
                if self.is_script() {
                    if let Some(id) = self.import_jsx.as_ref() {
                        let expr = create_static_member_expression(
                            id.create_read_reference(ctx),
                            Atom::from("Fragment"),
                            ctx,
                        );
                        (expr, false) // false = does not need update
                    } else {
                        let expr = create_static_member_expression(
                            create_read_identifier_reference(SPAN, Atom::empty(), None),
                            Atom::from("Fragment"),
                            ctx,
                        );
                        (expr, true) // true = needs update
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(id) = self.import_fragment.as_ref() {
                        let ident = id.create_read_reference(ctx);
                        let expr = ctx.ast.identifier_reference_expression(ident);
                        (expr, false) // false = does not need update
                    } else {
                        let ident = create_read_identifier_reference(SPAN, Atom::empty(), None);
                        let expr = ctx.ast.identifier_reference_expression(ident);
                        (expr, true) // true = needs update
                    }
                }
            }
        }
    }

    fn update_fragment(&self, arg: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
        let (id, local_id) = if self.is_script() {
            let Argument::StaticMemberExpression(member_expr) = arg else { unreachable!() };
            let Expression::Identifier(id) = &mut member_expr.object else {
                unreachable!();
            };
            (id, self.import_jsx.as_ref().unwrap())
        } else {
            let Argument::Identifier(id) = arg else { unreachable!() };
            (id, self.import_fragment.as_ref().unwrap())
        };

        id.name = local_id.name.clone();
        id.reference_id = Cell::new(Some(ctx.create_bound_reference(
            CompactStr::from(local_id.name.as_str()),
            local_id.symbol_id,
            ReferenceFlag::Read,
        )));
    }

    fn get_create_element(
        &self,
        has_key_after_props_spread: bool,
        jsxs: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => self.pragma.as_ref().unwrap().create_expression(ctx),
            ReactJsxRuntime::Automatic => {
                if self.is_script() {
                    let (object_id, property_name) = if has_key_after_props_spread {
                        (self.import_create_element.as_ref().unwrap(), Atom::from("createElement"))
                    } else {
                        let property_name = if self.options.development {
                            Atom::from("jsxDEV")
                        } else if jsxs {
                            Atom::from("jsxs")
                        } else {
                            Atom::from("jsx")
                        };
                        (self.import_jsx.as_ref().unwrap(), property_name)
                    };
                    let ident = object_id.create_read_reference(ctx);
                    create_static_member_expression(ident, property_name, ctx)
                } else {
                    let id = if has_key_after_props_spread {
                        self.import_create_element.as_ref().unwrap()
                    } else if jsxs && !self.options.development {
                        self.import_jsxs.as_ref().unwrap()
                    } else {
                        self.import_jsx.as_ref().unwrap()
                    };
                    let ident = id.create_read_reference(ctx);
                    self.ast().identifier_reference_expression(ident)
                }
            }
        }
    }

    fn transform_jsx_member_expression(
        &self,
        expr: &JSXMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let object = match &expr.object {
            JSXMemberExpressionObject::Identifier(ident) => {
                if ident.name == "this" {
                    self.ast().this_expression(SPAN)
                } else {
                    let ident = get_read_identifier_reference(ident.span, ident.name.clone(), ctx);
                    self.ast().identifier_reference_expression(ident)
                }
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.transform_jsx_member_expression(expr, ctx)
            }
        };
        let property = IdentifierName::new(SPAN, expr.property.name.clone());
        self.ast().static_member_expression(SPAN, object, property, false)
    }

    fn transform_jsx_attribute_item(
        &mut self,
        properties: &mut Vec<'a, ObjectPropertyKind<'a>>,
        attribute: &JSXAttributeItem<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match attribute {
            JSXAttributeItem::Attribute(attr) => {
                let kind = PropertyKind::Init;
                let key = self.get_attribute_name(&attr.name);
                let value = self.transform_jsx_attribute_value(attr.value.as_ref(), ctx);
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
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match value {
            Some(JSXAttributeValue::StringLiteral(s)) => {
                let jsx_text = Self::decode_entities(s.value.as_str());
                let literal = StringLiteral::new(s.span, self.ast().new_atom(&jsx_text));
                self.ast().literal_string_expression(literal)
            }
            Some(JSXAttributeValue::Element(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Element(e), ctx)
            }
            Some(JSXAttributeValue::Fragment(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx)
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => match &c.expression {
                e @ match_expression!(JSXExpression) => self.ast().copy(e.to_expression()),
                JSXExpression::EmptyExpression(_e) => {
                    self.ast().literal_boolean_expression(BooleanLiteral::new(SPAN, true))
                }
            },
            None => self.ast().literal_boolean_expression(BooleanLiteral::new(SPAN, true)),
        }
    }

    fn transform_jsx_child(
        &mut self,
        child: &JSXChild<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => self.transform_jsx_text(text.value.as_str()),
            JSXChild::ExpressionContainer(e) => match &e.expression {
                e @ match_expression!(JSXExpression) => Some(self.ast().copy(e.to_expression())),
                JSXExpression::EmptyExpression(_) => None,
            },
            JSXChild::Element(e) => {
                Some(self.transform_jsx(&JSXElementOrFragment::Element(e), ctx))
            }
            JSXChild::Fragment(e) => {
                Some(self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx))
            }
            JSXChild::Spread(e) => {
                self.ctx.error(diagnostics::spread_children_are_not_supported(e.span));
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

/// Create `IdentifierReference` for var name in current scope which is read from
fn get_read_identifier_reference<'a>(
    span: Span,
    name: Atom<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> IdentifierReference<'a> {
    let reference_id =
        ctx.create_reference_in_current_scope(name.to_compact_str(), ReferenceFlag::Read);
    create_read_identifier_reference(span, name, Some(reference_id))
}

/// Create `IdentifierReference` which is read from
#[inline]
fn create_read_identifier_reference(
    span: Span,
    name: Atom,
    reference_id: Option<ReferenceId>,
) -> IdentifierReference {
    IdentifierReference {
        span,
        name,
        reference_id: Cell::new(reference_id),
        reference_flag: ReferenceFlag::Read,
    }
}

fn create_static_member_expression<'a>(
    object_ident: IdentifierReference<'a>,
    property_name: Atom<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let object = ctx.ast.identifier_reference_expression(object_ident);
    let property = IdentifierName::new(SPAN, property_name);
    ctx.ast.static_member_expression(SPAN, object, property, false)
}
