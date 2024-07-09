use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::{
    identifier::{is_irregular_whitespace, is_line_terminator},
    reference::ReferenceFlag,
    symbol::SymbolFlags,
    xml_entities::XML_ENTITIES,
};
use oxc_traverse::TraverseCtx;

use super::{diagnostics, utils::get_line_column};
pub use super::{
    jsx_self::ReactJsxSelf,
    jsx_source::ReactJsxSource,
    options::{ReactJsxRuntime, ReactOptions},
};
use crate::{
    context::{Ctx, TransformCtx},
    helpers::{bindings::BoundIdentifier, module_imports::NamedImport},
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
    options: ReactOptions,

    ctx: Ctx<'a>,

    pub(super) jsx_self: ReactJsxSelf<'a>,
    pub(super) jsx_source: ReactJsxSource<'a>,

    // States
    bindings: Bindings<'a>,
}

/// Bindings for different import options
enum Bindings<'a> {
    Classic(ClassicBindings<'a>),
    AutomaticScript(AutomaticScriptBindings<'a>),
    AutomaticModule(AutomaticModuleBindings<'a>),
}
impl<'a> Bindings<'a> {
    #[inline]
    fn is_classic(&self) -> bool {
        matches!(self, Self::Classic(_))
    }
}

struct ClassicBindings<'a> {
    pragma: Pragma<'a>,
    pragma_frag: Pragma<'a>,
}

struct AutomaticScriptBindings<'a> {
    ctx: Ctx<'a>,
    jsx_runtime_importer: Atom<'a>,
    react_importer_len: u32,
    require_create_element: Option<BoundIdentifier<'a>>,
    require_jsx: Option<BoundIdentifier<'a>>,
    is_development: bool,
}

impl<'a> AutomaticScriptBindings<'a> {
    fn new(
        ctx: Ctx<'a>,
        jsx_runtime_importer: Atom<'a>,
        react_importer_len: u32,
        is_development: bool,
    ) -> Self {
        Self {
            ctx,
            jsx_runtime_importer,
            react_importer_len,
            require_create_element: None,
            require_jsx: None,
            is_development,
        }
    }

    fn require_create_element(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.require_create_element.is_none() {
            let source =
                get_import_source(self.jsx_runtime_importer.as_str(), self.react_importer_len);
            let id = self.add_require_statement("react", source, true, ctx);
            self.require_create_element = Some(id);
        }
        self.require_create_element.as_ref().unwrap().create_read_reference(ctx)
    }

    fn require_jsx(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.require_jsx.is_none() {
            let var_name =
                if self.is_development { "reactJsxDevRuntime" } else { "reactJsxRuntime" };
            let id =
                self.add_require_statement(var_name, self.jsx_runtime_importer.clone(), false, ctx);
            self.require_jsx = Some(id);
        };
        self.require_jsx.as_ref().unwrap().create_read_reference(ctx)
    }

    fn add_require_statement(
        &mut self,
        variable_name: &str,
        source: Atom<'a>,
        front: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let symbol_id =
            ctx.generate_uid_in_root_scope(variable_name, SymbolFlags::FunctionScopedVariable);
        let variable_name = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);

        let import = NamedImport::new(variable_name.clone(), None, symbol_id);
        self.ctx.module_imports.add_require(source, import, front);
        BoundIdentifier { name: variable_name, symbol_id }
    }
}

struct AutomaticModuleBindings<'a> {
    ctx: Ctx<'a>,
    jsx_runtime_importer: Atom<'a>,
    react_importer_len: u32,
    import_create_element: Option<BoundIdentifier<'a>>,
    import_fragment: Option<BoundIdentifier<'a>>,
    import_jsx: Option<BoundIdentifier<'a>>,
    import_jsxs: Option<BoundIdentifier<'a>>,
    is_development: bool,
}

impl<'a> AutomaticModuleBindings<'a> {
    fn new(
        ctx: Ctx<'a>,
        jsx_runtime_importer: Atom<'a>,
        react_importer_len: u32,
        is_development: bool,
    ) -> Self {
        Self {
            ctx,
            jsx_runtime_importer,
            react_importer_len,
            import_create_element: None,
            import_fragment: None,
            import_jsx: None,
            import_jsxs: None,
            is_development,
        }
    }

    fn import_create_element(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.import_create_element.is_none() {
            let source =
                get_import_source(self.jsx_runtime_importer.as_str(), self.react_importer_len);
            let id = self.add_import_statement("createElement", source, ctx);
            self.import_create_element = Some(id);
        }
        self.import_create_element.as_ref().unwrap().create_read_reference(ctx)
    }

    fn import_fragment(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.import_fragment.is_none() {
            self.import_fragment = Some(self.add_jsx_import_statement("Fragment", ctx));
        }
        self.import_fragment.as_ref().unwrap().create_read_reference(ctx)
    }

    fn import_jsx(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.import_jsx.is_none() {
            if self.is_development {
                self.add_import_jsx_dev(ctx);
            } else {
                self.import_jsx = Some(self.add_jsx_import_statement("jsx", ctx));
            };
        }
        self.import_jsx.as_ref().unwrap().create_read_reference(ctx)
    }

    fn import_jsxs(&mut self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        if self.import_jsxs.is_none() {
            if self.is_development {
                self.add_import_jsx_dev(ctx);
            } else {
                self.import_jsxs = Some(self.add_jsx_import_statement("jsxs", ctx));
            };
        }
        self.import_jsxs.as_ref().unwrap().create_read_reference(ctx)
    }

    // Inline so that compiler can see in `import_jsx` and `import_jsxs` that fields
    // are always `Some` after calling this function, and can elide the `unwrap()`s
    #[inline]
    fn add_import_jsx_dev(&mut self, ctx: &mut TraverseCtx<'a>) {
        let id = self.add_jsx_import_statement("jsxDEV", ctx);
        self.import_jsx = Some(id.clone());
        self.import_jsxs = Some(id);
    }

    fn add_jsx_import_statement(
        &mut self,
        name: &'static str,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        self.add_import_statement(name, self.jsx_runtime_importer.clone(), ctx)
    }

    fn add_import_statement(
        &mut self,
        name: &'static str,
        source: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let symbol_id = ctx.generate_uid_in_root_scope(name, SymbolFlags::FunctionScopedVariable);
        let local = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);

        let import = NamedImport::new(Atom::from(name), Some(local.clone()), symbol_id);
        self.ctx.module_imports.add_import(source, import);
        BoundIdentifier { name: local, symbol_id }
    }
}

#[inline]
fn get_import_source(jsx_runtime_importer: &str, react_importer_len: u32) -> Atom {
    Atom::from(&jsx_runtime_importer[..react_importer_len as usize])
}

/// Pragma used in classic mode
struct Pragma<'a> {
    object: Atom<'a>,
    property: Option<Atom<'a>>,
}

impl<'a> Pragma<'a> {
    /// Parse `options.pragma` or `options.pragma_frag`.
    ///
    /// If provided option is invalid, raise an error and use default.
    fn parse(
        pragma: Option<&String>,
        default_property_name: &'static str,
        ctx: &TransformCtx<'a>,
    ) -> Self {
        if let Some(pragma) = pragma {
            let mut parts = pragma.split('.');

            let object_name = parts.next().unwrap();
            if object_name.is_empty() {
                return Self::invalid(default_property_name, ctx);
            }

            let property = match parts.next() {
                Some(property_name) => {
                    if property_name.is_empty() || parts.next().is_some() {
                        return Self::invalid(default_property_name, ctx);
                    }
                    Some(ctx.ast.new_atom(property_name))
                }
                None => None,
            };

            let object = ctx.ast.new_atom(object_name);
            Self { object, property }
        } else {
            Self::default(default_property_name)
        }
    }

    fn invalid(default_property_name: &'static str, ctx: &TransformCtx<'a>) -> Self {
        ctx.error(diagnostics::invalid_pragma());
        Self::default(default_property_name)
    }

    fn default(default_property_name: &'static str) -> Self {
        Self { object: Atom::from("React"), property: Some(Atom::from(default_property_name)) }
    }

    fn create_expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let object = get_read_identifier_reference(SPAN, self.object.clone(), ctx);
        if let Some(property) = self.property.as_ref() {
            create_static_member_expression(object, property.clone(), ctx)
        } else {
            ctx.ast.expression_from_identifier_reference(object)
        }
    }
}

// Transforms
impl<'a> ReactJsx<'a> {
    pub fn new(options: ReactOptions, ctx: Ctx<'a>) -> Self {
        let bindings = match options.runtime {
            ReactJsxRuntime::Classic => {
                if options.import_source.is_some() {
                    ctx.error(diagnostics::import_source_cannot_be_set());
                }
                let pragma = Pragma::parse(options.pragma.as_ref(), "createElement", &ctx);
                let pragma_frag = Pragma::parse(options.pragma_frag.as_ref(), "Fragment", &ctx);
                Bindings::Classic(ClassicBindings { pragma, pragma_frag })
            }
            ReactJsxRuntime::Automatic => {
                if options.pragma.is_some() || options.pragma_frag.is_some() {
                    ctx.error(diagnostics::pragma_and_pragma_frag_cannot_be_set());
                }

                let is_development = options.development;
                #[allow(clippy::single_match_else, clippy::cast_possible_truncation)]
                let (jsx_runtime_importer, source_len) = match options.import_source.as_ref() {
                    Some(import_source) => {
                        let mut import_source = &**import_source;
                        let source_len = match u32::try_from(import_source.len()) {
                            Ok(0) | Err(_) => {
                                ctx.error(diagnostics::invalid_import_source());
                                import_source = "react";
                                import_source.len() as u32
                            }
                            Ok(source_len) => source_len,
                        };
                        let jsx_runtime_importer = ctx.ast.new_atom(&format!(
                            "{}/jsx-{}runtime",
                            import_source,
                            if is_development { "dev-" } else { "" }
                        ));
                        (jsx_runtime_importer, source_len)
                    }
                    None => {
                        let jsx_runtime_importer = if is_development {
                            Atom::from("react/jsx-dev-runtime")
                        } else {
                            Atom::from("react/jsx-runtime")
                        };
                        (jsx_runtime_importer, "react".len() as u32)
                    }
                };

                if ctx.source_type.is_script() {
                    Bindings::AutomaticScript(AutomaticScriptBindings::new(
                        Rc::clone(&ctx),
                        jsx_runtime_importer,
                        source_len,
                        is_development,
                    ))
                } else {
                    Bindings::AutomaticModule(AutomaticModuleBindings::new(
                        Rc::clone(&ctx),
                        jsx_runtime_importer,
                        source_len,
                        is_development,
                    ))
                }
            }
        };

        Self {
            options,
            ctx: Rc::clone(&ctx),
            jsx_self: ReactJsxSelf::new(Rc::clone(&ctx)),
            jsx_source: ReactJsxSource::new(ctx),
            bindings,
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

    fn ast(&self) -> AstBuilder<'a> {
        self.ctx.ast
    }
}

// Add imports
impl<'a> ReactJsx<'a> {
    pub fn add_runtime_imports(&mut self, program: &mut Program<'a>) {
        if self.bindings.is_classic() {
            if let Some(stmt) = self.jsx_source.get_var_file_name_statement() {
                program.body.insert(0, stmt);
            }
            return;
        }

        let imports = self.ctx.module_imports.get_import_statements();
        let mut index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ImportDeclaration(_)))
            .map_or(0, |i| i + 1);

        if let Some(stmt) = self.jsx_source.get_var_file_name_statement() {
            program.body.insert(index, stmt);
            // If source type is module then we need to add the import statement after the var file name statement
            // Follow the same behavior as babel
            if !self.is_script() {
                index += 1;
            }
        }

        program.body.splice(index..index, imports);
    }
}

enum JSXElementOrFragment<'a, 'b> {
    Element(&'b JSXElement<'a>),
    Fragment(&'b JSXFragment<'a>),
}

impl<'a, 'b> JSXElementOrFragment<'a, 'b> {
    fn span(&self) -> Span {
        match self {
            Self::Element(e) => e.span,
            Self::Fragment(e) => e.span,
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
        let is_classic = self.bindings.is_classic() || has_key_after_props_spread;
        let is_automatic = !is_classic;
        let is_development = self.options.development;

        let mut arguments = self.ast().new_vec();

        // The key prop in `<div key={true} />`
        let mut key_prop = None;

        // The object properties for the second argument of `React.createElement`
        let mut properties = self.ast().new_vec();

        let mut self_attr_span = None;
        let mut source_attr_span = None;

        if let JSXElementOrFragment::Element(e) = e {
            let attributes = &e.opening_element.attributes;
            for attribute in attributes {
                match attribute {
                    // optimize `{...prop}` to `prop` in static mode
                    JSXAttributeItem::SpreadAttribute(spread)
                        if is_classic && attributes.len() == 1 =>
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
                    self.ast().expression_array(SPAN, elements, None)
                };
                properties.push(self.ast().object_property_kind_object_property(
                    SPAN,
                    PropertyKind::Init,
                    self.ast().property_key_identifier_name(SPAN, "children"),
                    value,
                    None,
                    false,
                    false,
                    false,
                ));
            }
        }

        // React.createElement's second argument
        if !is_fragment && is_classic {
            if self.options.jsx_self_plugin && self.jsx_self.can_add_self_attribute(ctx) {
                if let Some(span) = self_attr_span {
                    self.jsx_self.report_error(span);
                } else {
                    properties.push(self.jsx_self.get_object_property_kind_for_jsx_plugin());
                }
            }

            if self.options.jsx_source_plugin {
                if let Some(span) = source_attr_span {
                    self.jsx_source.report_error(span);
                } else {
                    let (line, column) = get_line_column(e.span().start, self.ctx.source_text);
                    properties.push(
                        self.jsx_source.get_object_property_kind_for_jsx_plugin(line, column, ctx),
                    );
                }
            }
        }

        // It would be better to push to `arguments` earlier, rather than use `insert`.
        // But we have to do it here to replicate the same import order as Babel, in order to pass
        // Babel's conformance tests.
        // TODO(improve-on-babel): Change this if we can handle differing output in tests.
        let argument_expr = match e {
            JSXElementOrFragment::Element(e) => {
                self.transform_element_name(&e.opening_element.name, ctx)
            }
            JSXElementOrFragment::Fragment(_) => self.get_fragment(ctx),
        };
        arguments.insert(0, Argument::from(argument_expr));

        // If runtime is automatic that means we always to add `{ .. }` as the second argument even if it's empty
        if is_automatic || !properties.is_empty() {
            let object_expression = self.ast().expression_object(SPAN, properties, None);
            arguments.push(Argument::from(object_expression));
        } else if arguments.len() == 1 {
            // If not and second argument doesn't exist, we should add `null` as the second argument
            let null_expr = self.ast().expression_null_literal(SPAN);
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
                arguments.push(Argument::from(self.ctx.ast.expression_boolean_literal(
                    SPAN,
                    if is_fragment { false } else { children.len() > 1 },
                )));
            }

            // Fragment doesn't have source and self
            if !is_fragment {
                // { __source: { fileName, lineNumber, columnNumber } }
                if self.options.jsx_source_plugin {
                    if let Some(span) = source_attr_span {
                        self.jsx_source.report_error(span);
                    } else {
                        let (line, column) = get_line_column(e.span().start, self.ctx.source_text);
                        let expr = self.jsx_source.get_source_object(line, column, ctx);
                        arguments.push(Argument::from(expr));
                    }
                }

                // this
                if self.options.jsx_self_plugin && self.jsx_self.can_add_self_attribute(ctx) {
                    if let Some(span) = self_attr_span {
                        self.jsx_self.report_error(span);
                    } else {
                        arguments.push(Argument::from(self.ctx.ast.expression_this(SPAN)));
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
        self.ast().expression_call(
            e.span(),
            arguments,
            callee,
            Option::<TSTypeParameterInstantiation>::None,
            false,
        )
    }

    fn transform_element_name(
        &self,
        name: &JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(ident) => {
                if ident.name == "this" {
                    self.ast().expression_this(ident.span)
                } else if ident.name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                    self.ast().expression_string_literal(ident.span, &ident.name)
                } else {
                    let ident = get_read_identifier_reference(ident.span, ident.name.clone(), ctx);
                    self.ctx.ast.expression_from_identifier_reference(ident)
                }
            }
            JSXElementName::MemberExpression(member_expr) => {
                self.transform_jsx_member_expression(member_expr, ctx)
            }
            JSXElementName::NamespacedName(namespaced) => {
                if self.options.throw_if_namespace {
                    self.ctx.error(diagnostics::namespace_does_not_support(namespaced.span));
                }
                let name = self.ast().new_atom(&namespaced.to_string());
                self.ast().expression_string_literal(namespaced.span, name)
            }
        }
    }

    fn get_fragment(&mut self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match &mut self.bindings {
            Bindings::Classic(bindings) => bindings.pragma_frag.create_expression(ctx),
            Bindings::AutomaticScript(bindings) => {
                let object_ident = bindings.require_jsx(ctx);
                let property_name = Atom::from("Fragment");
                create_static_member_expression(object_ident, property_name, ctx)
            }
            Bindings::AutomaticModule(bindings) => {
                let ident = bindings.import_fragment(ctx);
                ctx.ast.expression_from_identifier_reference(ident)
            }
        }
    }

    fn get_create_element(
        &mut self,
        has_key_after_props_spread: bool,
        jsxs: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match &mut self.bindings {
            Bindings::Classic(bindings) => bindings.pragma.create_expression(ctx),
            Bindings::AutomaticScript(bindings) => {
                let (ident, property_name) = if has_key_after_props_spread {
                    (bindings.require_create_element(ctx), Atom::from("createElement"))
                } else {
                    let property_name = if bindings.is_development {
                        Atom::from("jsxDEV")
                    } else if jsxs {
                        Atom::from("jsxs")
                    } else {
                        Atom::from("jsx")
                    };
                    (bindings.require_jsx(ctx), property_name)
                };
                create_static_member_expression(ident, property_name, ctx)
            }
            Bindings::AutomaticModule(bindings) => {
                let ident = if has_key_after_props_spread {
                    bindings.import_create_element(ctx)
                } else if jsxs {
                    bindings.import_jsxs(ctx)
                } else {
                    bindings.import_jsx(ctx)
                };
                self.ast().expression_from_identifier_reference(ident)
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
                    self.ast().expression_this(ident.span)
                } else {
                    let ident = get_read_identifier_reference(ident.span, ident.name.clone(), ctx);
                    self.ast().expression_from_identifier_reference(ident)
                }
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.transform_jsx_member_expression(expr, ctx)
            }
        };
        let property = IdentifierName::new(expr.property.span, expr.property.name.clone());
        self.ast().member_expression_static(expr.span, object, property, false).into()
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
                let object_property = self.ast().object_property_kind_object_property(
                    attr.span, kind, key, value, None, false, false, false,
                );
                properties.push(object_property);
            }
            JSXAttributeItem::SpreadAttribute(attr) => match &attr.argument {
                Expression::ObjectExpression(expr) if !expr.has_proto() => {
                    properties.extend(self.ast().copy(&expr.properties));
                }
                expr => {
                    let argument = self.ast().copy(expr);
                    let object_property =
                        self.ast().object_property_kind_spread_element(attr.span, argument);
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
                self.ast().expression_string_literal(s.span, jsx_text)
            }
            Some(JSXAttributeValue::Element(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Element(e), ctx)
            }
            Some(JSXAttributeValue::Fragment(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx)
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => match &c.expression {
                e @ match_expression!(JSXExpression) => self.ast().copy(e.to_expression()),
                JSXExpression::EmptyExpression(e) => {
                    self.ast().expression_boolean_literal(e.span, true)
                }
            },
            None => self.ast().expression_boolean_literal(SPAN, true),
        }
    }

    fn transform_jsx_child(
        &mut self,
        child: &JSXChild<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => self.transform_jsx_text(text),
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
                    let expr = self.ast().expression_string_literal(ident.span, name);
                    self.ast().property_key_expression(expr)
                } else {
                    self.ast().property_key_identifier_name(ident.span, name)
                }
            }
            JSXAttributeName::NamespacedName(namespaced) => {
                let name = self.ast().new_atom(&namespaced.to_string());
                let expr = self.ast().expression_string_literal(namespaced.span, name);
                self.ast().property_key_expression(expr)
            }
        }
    }

    fn transform_jsx_text(&self, text: &JSXText<'a>) -> Option<Expression<'a>> {
        Self::fixup_whitespace_and_decode_entities(text.value.as_str())
            .map(|s| self.ast().expression_string_literal(text.span, s))
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
    IdentifierReference::new_read(span, name, Some(reference_id))
}

fn create_static_member_expression<'a>(
    object_ident: IdentifierReference<'a>,
    property_name: Atom<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let object = ctx.ast.expression_from_identifier_reference(object_ident);
    let property = ctx.ast.identifier_name(SPAN, property_name);
    ctx.ast.member_expression_static(SPAN, object, property, false).into()
}
