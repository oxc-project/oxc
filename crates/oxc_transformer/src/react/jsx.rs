//! React JSX
//!
//! This plugin transforms React JSX to JS.
//!
//! > This plugin is included in `preset-react`.
//!
//! Has two modes which create different output:
//! 1. Automatic
//! 2. Classic
//!
//! And also prod/dev modes:
//! 1. Production
//! 2. Development
//!
//! ## Example
//!
//! ### Automatic
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
//! // Production mode
//! import { jsx as _jsx, Fragment as _Fragment } from "react/jsx-runtime";
//! _jsx("div", { children: "foo" });
//! _jsx(Bar, { children: "foo" });
//! _jsx(_Fragment, { children: "foo" });
//! ```
//!
//! ```js
//! // Development mode
//! var _jsxFileName = "<CWD>/test.js";
//! import { jsxDEV as _jsxDEV, Fragment as _Fragment } from "react/jsx-dev-runtime";
//! _jsxDEV(
//!     "div", { children: "foo" }, void 0, false,
//!     { fileName: _jsxFileName, lineNumber: 1, columnNumber: 1 },
//!     this
//! );
//! _jsxDEV(
//!     Bar, { children: "foo" }, void 0, false,
//!     { fileName: _jsxFileName, lineNumber: 2, columnNumber: 1 },
//!     this
//! );
//! _jsxDEV(_Fragment, { children: "foo" }, void 0, false);
//! ```
//!
//! ### Classic
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
//! // Production mode
//! React.createElement("div", null, "foo");
//! React.createElement(Bar, null, "foo");
//! React.createElement(React.Fragment, null, "foo");
//! ```
//!
//! ```js
//! // Development mode
//! var _jsxFileName = "<CWD>/test.js";
//! React.createElement("div", {
//!     __self: this,
//!     __source: { fileName: _jsxFileName, lineNumber: 1, columnNumber: 1 }
//! }, "foo");
//! React.createElement(Bar, {
//!     __self: this,
//!     __source: { fileName: _jsxFileName, lineNumber: 2, columnNumber: 1 }
//! }, "foo");
//! React.createElement(React.Fragment, null, "foo");
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx).
//!
//! ## References:
//!
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder, NONE};
use oxc_ecmascript::PropName;
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::{
    identifier::{is_irregular_whitespace, is_line_terminator},
    reference::ReferenceFlags,
    symbol::SymbolFlags,
    xml_entities::XML_ENTITIES,
};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

use super::diagnostics;

pub use super::{
    jsx_self::ReactJsxSelf,
    jsx_source::ReactJsxSource,
    options::{JsxOptions, JsxRuntime},
};

pub struct ReactJsx<'a, 'ctx> {
    options: JsxOptions,

    ctx: &'ctx TransformCtx<'a>,

    pub(super) jsx_self: ReactJsxSelf<'a, 'ctx>,
    pub(super) jsx_source: ReactJsxSource<'a, 'ctx>,

    // States
    bindings: Bindings<'a, 'ctx>,
}

/// Bindings for different import options
enum Bindings<'a, 'ctx> {
    Classic(ClassicBindings<'a>),
    AutomaticScript(AutomaticScriptBindings<'a, 'ctx>),
    AutomaticModule(AutomaticModuleBindings<'a, 'ctx>),
}
impl<'a, 'ctx> Bindings<'a, 'ctx> {
    #[inline]
    fn is_classic(&self) -> bool {
        matches!(self, Self::Classic(_))
    }
}

struct ClassicBindings<'a> {
    pragma: Pragma<'a>,
    pragma_frag: Pragma<'a>,
}

struct AutomaticScriptBindings<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
    jsx_runtime_importer: Atom<'a>,
    react_importer_len: u32,
    require_create_element: Option<BoundIdentifier<'a>>,
    require_jsx: Option<BoundIdentifier<'a>>,
    is_development: bool,
}

impl<'a, 'ctx> AutomaticScriptBindings<'a, 'ctx> {
    fn new(
        ctx: &'ctx TransformCtx<'a>,
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
            // We have to insert this `require` above `require("react/jsx-runtime")`
            // just to pass one of Babel's tests, but the order doesn't actually matter.
            // TODO(improve-on-babel): Remove this once we don't need our output to match Babel exactly.
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
        let binding =
            ctx.generate_uid_in_root_scope(variable_name, SymbolFlags::FunctionScopedVariable);
        self.ctx.module_imports.add_default_import(source, binding.clone(), front);
        binding
    }
}

struct AutomaticModuleBindings<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
    jsx_runtime_importer: Atom<'a>,
    react_importer_len: u32,
    import_create_element: Option<BoundIdentifier<'a>>,
    import_fragment: Option<BoundIdentifier<'a>>,
    import_jsx: Option<BoundIdentifier<'a>>,
    import_jsxs: Option<BoundIdentifier<'a>>,
    is_development: bool,
}

impl<'a, 'ctx> AutomaticModuleBindings<'a, 'ctx> {
    fn new(
        ctx: &'ctx TransformCtx<'a>,
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
        let binding = ctx.generate_uid_in_root_scope(name, SymbolFlags::Import);
        self.ctx.module_imports.add_named_import(source, Atom::from(name), binding.clone(), false);
        binding
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
        ast: AstBuilder<'a>,
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
                    Some(ast.atom(property_name))
                }
                None => None,
            };

            let object = ast.atom(object_name);
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

impl<'a, 'ctx> ReactJsx<'a, 'ctx> {
    pub fn new(options: JsxOptions, ast: AstBuilder<'a>, ctx: &'ctx TransformCtx<'a>) -> Self {
        let bindings = match options.runtime {
            JsxRuntime::Classic => {
                if options.import_source.is_some() {
                    ctx.error(diagnostics::import_source_cannot_be_set());
                }
                let pragma = Pragma::parse(options.pragma.as_ref(), "createElement", ast, ctx);
                let pragma_frag = Pragma::parse(options.pragma_frag.as_ref(), "Fragment", ast, ctx);
                Bindings::Classic(ClassicBindings { pragma, pragma_frag })
            }
            JsxRuntime::Automatic => {
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
                        let jsx_runtime_importer = ast.atom(&format!(
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
                        ctx,
                        jsx_runtime_importer,
                        source_len,
                        is_development,
                    ))
                } else {
                    Bindings::AutomaticModule(AutomaticModuleBindings::new(
                        ctx,
                        jsx_runtime_importer,
                        source_len,
                        is_development,
                    ))
                }
            }
        };

        Self {
            options,
            ctx,
            jsx_self: ReactJsxSelf::new(ctx),
            jsx_source: ReactJsxSource::new(ctx),
            bindings,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ReactJsx<'a, 'ctx> {
    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.insert_filename_var_statement(ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        *expr = match expr {
            Expression::JSXElement(e) => self.transform_jsx(&JSXElementOrFragment::Element(e), ctx),
            Expression::JSXFragment(e) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx)
            }
            _ => return,
        };
    }
}

impl<'a, 'ctx> ReactJsx<'a, 'ctx> {
    fn is_script(&self) -> bool {
        self.ctx.source_type.is_script()
    }

    fn insert_filename_var_statement(&mut self, ctx: &mut TraverseCtx<'a>) {
        let Some(declarator) = self.jsx_source.get_filename_var_declarator(ctx) else { return };

        // If is a module, add filename statements before `import`s. If script, then after `require`s.
        // This is the same behavior as Babel.
        // If in classic mode, then there are no import statements, so it doesn't matter either way.
        // TODO(improve-on-babel): Simplify this once we don't need to follow Babel exactly.
        if self.bindings.is_classic() || !self.is_script() {
            // Insert before imports - add to `top_level_statements` immediately
            let stmt = Statement::VariableDeclaration(ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                ctx.ast.vec1(declarator),
                false,
            ));
            self.ctx.top_level_statements.insert_statement(stmt);
        } else {
            // Insert after imports - add to `var_declarations`, which are inserted after `require` statements
            self.ctx.var_declarations.insert_declarator(declarator, ctx);
        }
    }

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

        let mut arguments = ctx.ast.vec();

        // The key prop in `<div key={true} />`
        let mut key_prop = None;

        // The object properties for the second argument of `React.createElement`
        let mut properties = ctx.ast.vec();

        let mut self_attr_span = None;
        let mut source_attr_span = None;

        if let JSXElementOrFragment::Element(e) = e {
            let attributes = &e.opening_element.attributes;
            for attribute in attributes {
                match attribute {
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

                        // Add attribute to prop object
                        let kind = PropertyKind::Init;
                        let key = Self::get_attribute_name(&attr.name, ctx);
                        let value = self.transform_jsx_attribute_value(attr.value.as_ref(), ctx);
                        let object_property = ctx.ast.object_property_kind_object_property(
                            attr.span, kind, key, value, None, false, false, false,
                        );
                        properties.push(object_property);
                    }
                    // optimize `{...prop}` to `prop` in static mode
                    JSXAttributeItem::SpreadAttribute(spread) => {
                        if is_classic && attributes.len() == 1 {
                            // deopt if spreading an object with `__proto__` key
                            if !matches!(&spread.argument, Expression::ObjectExpression(o) if has_proto(o))
                            {
                                arguments.push(Argument::from({
                                    // SAFETY: `ast.copy` is unsound! We need to fix.
                                    unsafe { ctx.ast.copy(&spread.argument) }
                                }));
                                continue;
                            }
                        }

                        // Add attribute to prop object
                        match &spread.argument {
                            Expression::ObjectExpression(expr) if !has_proto(expr) => {
                                // SAFETY: `ast.copy` is unsound! We need to fix.
                                properties.extend(unsafe { ctx.ast.copy(&expr.properties) });
                            }
                            expr => {
                                // SAFETY: `ast.copy` is unsound! We need to fix.
                                let argument = unsafe { ctx.ast.copy(expr) };
                                let object_property = ctx
                                    .ast
                                    .object_property_kind_spread_element(spread.span, argument);
                                properties.push(object_property);
                            }
                        }
                    }
                }
            }
        }

        let mut need_jsxs = false;

        let children = e.children();
        let mut children_len = children.len();

        // Append children to object properties in automatic mode
        if is_automatic {
            let mut children = ctx.ast.vec_from_iter(
                children.iter().filter_map(|child| self.transform_jsx_child(child, ctx)),
            );
            children_len = children.len();
            if children_len != 0 {
                let value = if children_len == 1 {
                    children.pop().unwrap()
                } else {
                    let elements = ctx
                        .ast
                        .vec_from_iter(children.into_iter().map(ArrayExpressionElement::from));
                    need_jsxs = true;
                    ctx.ast.expression_array(SPAN, elements, None)
                };
                properties.push(ctx.ast.object_property_kind_object_property(
                    SPAN,
                    PropertyKind::Init,
                    ctx.ast.property_key_identifier_name(SPAN, "children"),
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
                    properties.push(ReactJsxSelf::get_object_property_kind_for_jsx_plugin(ctx));
                }
            }

            if self.options.jsx_source_plugin {
                if let Some(span) = source_attr_span {
                    self.jsx_source.report_error(span);
                } else {
                    let (line, column) = self.jsx_source.get_line_column(e.span().start);
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
                if let Some(closing_element) = &e.closing_element {
                    if let Some(ident) = closing_element.name.get_identifier() {
                        ctx.delete_reference_for_identifier(ident);
                    }
                }

                self.transform_element_name(&e.opening_element.name, ctx)
            }
            JSXElementOrFragment::Fragment(_) => self.get_fragment(ctx),
        };
        arguments.insert(0, Argument::from(argument_expr));

        // If runtime is automatic that means we always to add `{ .. }` as the second argument even if it's empty
        if is_automatic || !properties.is_empty() {
            let object_expression = ctx.ast.expression_object(SPAN, properties, None);
            arguments.push(Argument::from(object_expression));
        } else if arguments.len() == 1 {
            // If not and second argument doesn't exist, we should add `null` as the second argument
            let null_expr = ctx.ast.expression_null_literal(SPAN);
            arguments.push(Argument::from(null_expr));
        }

        // Only jsx and jsxDev will have more than 2 arguments
        if is_automatic {
            // key
            if key_prop.is_some() {
                arguments.push(Argument::from(self.transform_jsx_attribute_value(key_prop, ctx)));
            } else if is_development {
                arguments.push(Argument::from(ctx.ast.void_0(SPAN)));
            }

            // isStaticChildren
            if is_development {
                arguments.push(Argument::from(ctx.ast.expression_boolean_literal(
                    SPAN,
                    if is_fragment { false } else { children_len > 1 },
                )));
            }

            // Fragment doesn't have source and self
            if !is_fragment {
                // { __source: { fileName, lineNumber, columnNumber } }
                if self.options.jsx_source_plugin {
                    if let Some(span) = source_attr_span {
                        self.jsx_source.report_error(span);
                    } else {
                        let (line, column) = self.jsx_source.get_line_column(e.span().start);
                        let expr = self.jsx_source.get_source_object(line, column, ctx);
                        arguments.push(Argument::from(expr));
                    }
                }

                // this
                if self.options.jsx_self_plugin && self.jsx_self.can_add_self_attribute(ctx) {
                    if let Some(span) = self_attr_span {
                        self.jsx_self.report_error(span);
                    } else {
                        arguments.push(Argument::from(ctx.ast.expression_this(SPAN)));
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
        ctx.ast.expression_call(e.span(), callee, NONE, arguments, false)
    }

    fn transform_element_name(
        &self,
        name: &JSXElementName<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(ident) => {
                ctx.ast.expression_string_literal(ident.span, ident.name.clone())
            }
            JSXElementName::IdentifierReference(ident) => {
                ctx.ast.expression_from_identifier_reference(ident.as_ref().clone())
            }
            JSXElementName::MemberExpression(member_expr) => {
                Self::transform_jsx_member_expression(member_expr, ctx)
            }
            JSXElementName::NamespacedName(namespaced) => {
                if self.options.throw_if_namespace {
                    self.ctx.error(diagnostics::namespace_does_not_support(namespaced.span));
                }
                ctx.ast.expression_string_literal(namespaced.span, namespaced.to_string())
            }
            JSXElementName::ThisExpression(expr) => ctx.ast.expression_this(expr.span),
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
                ctx.ast.expression_from_identifier_reference(ident)
            }
        }
    }

    fn transform_jsx_member_expression(
        expr: &JSXMemberExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let object = match &expr.object {
            JSXMemberExpressionObject::IdentifierReference(ident) => {
                ctx.ast.expression_from_identifier_reference(ident.as_ref().clone())
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                Self::transform_jsx_member_expression(expr, ctx)
            }
            JSXMemberExpressionObject::ThisExpression(expr) => ctx.ast.expression_this(expr.span),
        };
        let property = IdentifierName::new(expr.property.span, expr.property.name.clone());
        ctx.ast.member_expression_static(expr.span, object, property, false).into()
    }

    fn transform_jsx_attribute_value(
        &mut self,
        value: Option<&JSXAttributeValue<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match value {
            Some(JSXAttributeValue::StringLiteral(s)) => {
                let jsx_text = Self::decode_entities(s.value.as_str());
                ctx.ast.expression_string_literal(s.span, jsx_text)
            }
            Some(JSXAttributeValue::Element(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Element(e), ctx)
            }
            Some(JSXAttributeValue::Fragment(e)) => {
                self.transform_jsx(&JSXElementOrFragment::Fragment(e), ctx)
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => match &c.expression {
                e @ match_expression!(JSXExpression) => {
                    // SAFETY: `ast.copy` is unsound! We need to fix.
                    unsafe { ctx.ast.copy(e.to_expression()) }
                }
                JSXExpression::EmptyExpression(e) => {
                    ctx.ast.expression_boolean_literal(e.span, true)
                }
            },
            None => ctx.ast.expression_boolean_literal(SPAN, true),
        }
    }

    fn transform_jsx_child(
        &mut self,
        child: &JSXChild<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => Self::transform_jsx_text(text, ctx),
            JSXChild::ExpressionContainer(e) => match &e.expression {
                e @ match_expression!(JSXExpression) => {
                    // SAFETY: `ast.copy` is unsound! We need to fix.
                    Some(unsafe { ctx.ast.copy(e.to_expression()) })
                }
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

    fn get_attribute_name(name: &JSXAttributeName<'a>, ctx: &TraverseCtx<'a>) -> PropertyKey<'a> {
        match name {
            JSXAttributeName::Identifier(ident) => {
                let name = ident.name.clone();
                if ident.name.contains('-') {
                    let expr = ctx.ast.expression_string_literal(ident.span, name);
                    ctx.ast.property_key_expression(expr)
                } else {
                    ctx.ast.property_key_identifier_name(ident.span, name)
                }
            }
            JSXAttributeName::NamespacedName(namespaced) => {
                let name = ctx.ast.atom(&namespaced.to_string());
                let expr = ctx.ast.expression_string_literal(namespaced.span, name);
                ctx.ast.property_key_expression(expr)
            }
        }
    }

    fn transform_jsx_text(text: &JSXText<'a>, ctx: &TraverseCtx<'a>) -> Option<Expression<'a>> {
        Self::fixup_whitespace_and_decode_entities(text.value.as_str())
            .map(|s| ctx.ast.expression_string_literal(text.span, s))
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
        let mut buffer = String::new();
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
                    buffer.push_str(&s[prev..start]);
                    prev = end + 1;
                    let word = &s[start + 1..end];
                    if let Some(decimal) = word.strip_prefix('#') {
                        if let Some(hex) = decimal.strip_prefix('x') {
                            if let Some(c) =
                                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                            {
                                // &x0123;
                                buffer.push(c);
                                continue;
                            }
                        } else if let Some(c) = decimal.parse::<u32>().ok().and_then(char::from_u32)
                        {
                            // &#0123;
                            buffer.push(c);
                            continue;
                        }
                    } else if let Some(c) = XML_ENTITIES.get(word) {
                        // &quote;
                        buffer.push(*c);
                        continue;
                    }
                    // fallback
                    buffer.push('&');
                    buffer.push_str(word);
                    buffer.push(';');
                }
            }
        }
        buffer.push_str(&s[prev..]);
        buffer
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

/// Create `IdentifierReference` for var name in current scope which is read from
fn get_read_identifier_reference<'a>(
    span: Span,
    name: Atom<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> IdentifierReference<'a> {
    let reference_id =
        ctx.create_reference_in_current_scope(name.to_compact_str(), ReferenceFlags::Read);
    IdentifierReference::new_with_reference_id(span, name, Some(reference_id))
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

fn has_proto(e: &ObjectExpression<'_>) -> bool {
    e.properties.iter().any(|p| p.prop_name().is_some_and(|name| name.0 == "__proto__"))
}
