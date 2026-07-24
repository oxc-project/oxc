//! Shared AST walk driving lowering's two pre-passes in a single traversal.
//!
//! Before BuildHIR, lowering runs two pre-passes over the compiled function:
//!
//! * [`identifier_loc_index`]: builds the location index for resolved
//!   identifier references and binding declarations.
//! * [`find_context_identifiers`]: finds bindings that cross function
//!   boundaries and need StoreContext/LoadContext semantics.
//!
//! Every type that implements an `oxc_ast_visit` visitor trait monomorphizes
//! the entire generated AST walk, so driving these passes separately paid for
//! two full walk instantiations in the binary (and traversed the function
//! twice). [`PrePassVisitor`] owns both passes' state and implements [`Visit`]
//! once, forwarding each node event to both, so they share one walk.
//!
//! Each pass must keep observing exactly the nodes its own walk observed:
//!
//! * The identifier-loc pass used the full [`Visit`] walk. It must keep seeing
//!   identifiers inside TS type subtrees: `find_context_identifiers`'s
//!   post-walk supplement loop and BuildHIR's `gather_captured_context` both
//!   rely on type-space references being present in the index.
//! * The context-identifier pass used the JS-only [`VisitJs`] walk, which
//!   prunes pure type grammar (but walks TS constructs carrying runtime JS —
//!   decorators, `x as T` casts, `import x = require(..)` — exactly like
//!   [`Visit`] does). Under the full walk, pure type grammar is entered only
//!   via `visit_ts_type`, `visit_ts_type_annotation`,
//!   `visit_ts_type_parameter_declaration` and
//!   `visit_ts_type_parameter_instantiation` — each of which bumps
//!   `type_depth` below — plus the TS type alias / interface declarations
//!   stubbed to no-ops. Forwarding context events only while `type_depth == 0`
//!   therefore reproduces the `VisitJs` event stream exactly.
//!
//! The per-node behavior (and the comments explaining it) is carried over
//! unchanged from the two separate visitor impls this replaces; see the module
//! docs of [`identifier_loc_index`] and [`find_context_identifiers`] for what
//! each pass records and deliberately skips.
//!
//! [`VisitJs`]: oxc_ast_visit::VisitJs
//! [`identifier_loc_index`]: super::identifier_loc_index
//! [`find_context_identifiers`]: super::find_context_identifiers

use rustc_hash::FxHashSet;

use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_syntax::scope::ScopeFlags;

use crate::scope::{ScopeResolver, SymbolId};

use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::find_context_identifiers::{
    ContextIdentifierVisitor, find_context_identifiers,
};
use crate::react_compiler_lowering::identifier_loc_index::{
    IdentifierLocIndex, IdentifierLocVisitor,
};

/// Driver owning both pre-passes' state. Its [`Visit`] impl is lowering's only
/// full-AST walk instantiation: each overridden method forwards the node to
/// the identifier-loc pass and — when outside TS type subtrees — to the
/// context-identifier pass, preserving both original event streams.
struct PrePassVisitor<'a> {
    loc: IdentifierLocVisitor,
    ctx: ContextIdentifierVisitor<'a>,
}

impl PrePassVisitor<'_> {
    /// True while inside a TS type subtree. The context-identifier pass's
    /// original `VisitJs` walk never entered these, so its events are
    /// suppressed while this holds.
    fn in_type_space(&self) -> bool {
        self.loc.type_depth > 0
    }
}

impl<'a> PrePassVisitor<'a> {
    /// Visit the JSX element name identifiers (and only those) while the
    /// loc pass's `current_opening_element_span` is set, mirroring the
    /// original `walk_jsx_element_name` / `walk_jsx_member_expression`.
    /// Lowercase tag names, `this`, and member-property parts carry no
    /// reference and are never looked up, so only `IdentifierReference` names
    /// are visited — which is also exactly the set of name nodes that fired
    /// the context pass's capture check under `walk_js`.
    fn visit_jsx_element_name_refs(&mut self, name: &JSXElementName<'a>) {
        match name {
            JSXElementName::IdentifierReference(id) => self.visit_identifier_reference(id),
            JSXElementName::MemberExpression(m) => self.visit_jsx_member_expression_refs(m),
            JSXElementName::Identifier(_)
            | JSXElementName::ThisExpression(_)
            | JSXElementName::NamespacedName(_) => {}
        }
    }

    fn visit_jsx_member_expression_refs(&mut self, expr: &JSXMemberExpression<'a>) {
        match &expr.object {
            JSXMemberExpressionObject::IdentifierReference(id) => {
                self.visit_identifier_reference(id);
            }
            JSXMemberExpressionObject::ThisExpression(_) => {}
            JSXMemberExpressionObject::MemberExpression(inner) => {
                self.visit_jsx_member_expression_refs(inner);
            }
        }
    }
}

impl<'a> Visit<'a> for PrePassVisitor<'a> {
    // ---- identifiers (both passes) ----

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.loc.record_reference(it);
        if !self.in_type_space() {
            self.ctx.enter_identifier_reference(it);
        }
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        // loc: `collect_type_idents` only collected IdentifierReference /
        // IdentifierName, never BindingIdentifier, so type-parameter declaration
        // names (`<T>`) and other binding positions inside type subtrees must not
        // be recorded. ctx: `VisitJs` never walked type subtrees at all.
        if self.in_type_space() {
            return;
        }
        self.loc.record_declaration(it);
        self.ctx.enter_binding_identifier(it);
    }

    // ---- function scopes (ctx pushes BOTH the generic scope and the function stack) ----

    fn visit_function(&mut self, it: &Function<'a>, _flags: ScopeFlags) {
        let scope_pushed = self.ctx.enter_scope(it.scope_id.get());
        let fn_pushed = self.ctx.push_function_scope(it.scope_id.get());
        // The original Babel walker driving the context pass never visited the
        // function NAME identifier (`it.id`); visiting it — with the inner
        // function already on `function_stack` — would spuriously mark a hoisted
        // nested-function name as referenced_by_inner_fn. The loc pass DOES
        // record it (function declaration & expression names go into the
        // declaration map), so the name is forwarded to the loc pass only.
        // Functions cannot sit inside pure type grammar, so no type-depth check
        // is needed (the previous loc walk fired `visit_binding_identifier` here
        // with `type_depth == 0` always).
        if let Some(id) = &it.id {
            self.loc.record_declaration(id);
        }
        // Same field order as the generated `walk_function`, which the loc pass
        // previously used unmodified.
        if let Some(type_parameters) = &it.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(this_param) = &it.this_param {
            self.visit_ts_this_parameter(this_param);
        }
        self.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            self.visit_ts_type_annotation(return_type);
        }
        if let Some(body) = &it.body {
            self.visit_function_body(body);
        }
        self.ctx.pop_function_scope(fn_pushed);
        self.ctx.exit_scope(scope_pushed);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        let scope_pushed = self.ctx.enter_scope(it.scope_id.get());
        let fn_pushed = self.ctx.push_function_scope(it.scope_id.get());
        // Same field order as the generated `walk_arrow_function_expression`.
        if let Some(type_parameters) = &it.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        self.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            self.visit_ts_type_annotation(return_type);
        }
        self.visit_function_body(&it.body);
        self.ctx.pop_function_scope(fn_pushed);
        self.ctx.exit_scope(scope_pushed);
    }

    // ---- non-function scope-creating nodes (ctx pushes only the generic scope) ----

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_block_statement(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_for_statement(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_for_in_statement(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_for_of_statement(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_switch_statement(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_catch_clause(self, it);
        self.ctx.exit_scope(pushed);
    }

    fn visit_static_block(&mut self, it: &StaticBlock<'a>) {
        let pushed = self.ctx.enter_scope(it.scope_id.get());
        walk::walk_static_block(self, it);
        self.ctx.exit_scope(pushed);
    }

    // ---- reassignment tracking (ctx) ----

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if !self.in_type_space() && !self.ctx.has_error() {
            let current_scope = self.ctx.current_scope();
            self.ctx.walk_assignment_target_for_reassignment(&it.left, current_scope);
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if !self.in_type_space()
            && let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &it.argument
        {
            let current_scope = self.ctx.current_scope();
            self.ctx.handle_reassignment_identifier(&ident.name, current_scope);
        }
        walk::walk_update_expression(self, it);
    }

    // ---- positions deliberately NOT visited, matching BOTH original walkers ----

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        // Both originals walked the property only when computed; a static member
        // is non-computed, so the property name is never visited.
        self.visit_expression(&it.object);
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        // Both originals walked the key only when computed.
        if it.computed {
            self.visit_property_key(&it.key);
        }
        self.visit_expression(&it.value);
    }

    fn visit_class(&mut self, it: &Class<'a>) {
        // loc: the original immutable walker recorded only the class name and
        // then the class's type-bearing parts (decorators / implements / type
        // params) as RawNodes (type idents only). It did NOT walk `super_class`
        // (the extends clause) nor the class body's method/property members.
        // ctx: the original walker did not recurse into the class at all — the
        // type-bearing parts carried no `enter_identifier` calls, so the class
        // contributes nothing to the walker-based capture analysis. Everything
        // below is therefore loc-only (type subtrees suppress ctx events).
        if let Some(id) = &it.id {
            self.loc.record_declaration(id);
        }
        if let Some(type_parameters) = &it.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_type_arguments) = &it.super_type_arguments {
            self.visit_ts_type_parameter_instantiation(super_type_arguments);
        }
        self.loc.type_depth += 1;
        self.visit_ts_class_implements_list(&it.implements);
        self.loc.type_depth -= 1;
    }

    // ---- JSX ----

    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        // loc: the opening element's span is active only while walking the
        // element name; it is cleared before the type arguments, attributes,
        // and children.
        self.loc.current_opening_element_span = Some(it.opening_element.span);
        self.visit_jsx_element_name_refs(&it.opening_element.name);
        self.loc.current_opening_element_span = None;
        if let Some(type_args) = &it.opening_element.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_args);
        }

        // Both originals visited only attribute VALUES and spread arguments,
        // never attribute names (nor namespaced names).
        for attr in &it.opening_element.attributes {
            match attr {
                JSXAttributeItem::Attribute(a) => {
                    if let Some(value) = &a.value {
                        match value {
                            JSXAttributeValue::ExpressionContainer(c) => {
                                self.visit_jsx_expression_container(c);
                            }
                            JSXAttributeValue::Element(el) => self.visit_jsx_element(el),
                            JSXAttributeValue::Fragment(f) => self.visit_jsx_fragment(f),
                            JSXAttributeValue::StringLiteral(_) => {}
                        }
                    }
                }
                JSXAttributeItem::SpreadAttribute(a) => {
                    self.visit_expression(&a.argument);
                }
            }
        }
        for child in &it.children {
            self.visit_jsx_child(child);
        }
        // The loc pass had no closing-element handling, but the context pass's
        // `walk_js` walk DID visit the closing element name — keep firing its
        // capture check for `</Foo>` references (context-only).
        if let Some(closing_element) = &it.closing_element {
            self.ctx.check_jsx_element_name(&closing_element.name);
        }
    }

    // ---- TS type subtrees (loc-only; ctx events are suppressed inside) ----

    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        self.loc.type_depth += 1;
        walk::walk_ts_type(self, it);
        self.loc.type_depth -= 1;
    }

    fn visit_ts_type_annotation(&mut self, it: &TSTypeAnnotation<'a>) {
        self.loc.type_depth += 1;
        walk::walk_ts_type_annotation(self, it);
        self.loc.type_depth -= 1;
    }

    fn visit_ts_type_parameter_instantiation(&mut self, it: &TSTypeParameterInstantiation<'a>) {
        self.loc.type_depth += 1;
        walk::walk_ts_type_parameter_instantiation(self, it);
        self.loc.type_depth -= 1;
    }

    fn visit_ts_type_parameter_declaration(&mut self, it: &TSTypeParameterDeclaration<'a>) {
        self.loc.type_depth += 1;
        walk::walk_ts_type_parameter_declaration(self, it);
        self.loc.type_depth -= 1;
    }

    // The original loc walker treated these TS declaration statements as no-ops
    // (nothing inside them was recorded), and the original ctx walker treated
    // enum/namespace bodies — runtime JS which `VisitJs` WOULD walk — as opaque
    // RawNodes (alias/interface it never walked at all). Skip entirely.

    fn visit_ts_type_alias_declaration(&mut self, _it: &TSTypeAliasDeclaration<'a>) {}

    fn visit_ts_interface_declaration(&mut self, _it: &TSInterfaceDeclaration<'a>) {}

    fn visit_ts_enum_declaration(&mut self, _it: &TSEnumDeclaration<'a>) {}

    fn visit_ts_module_declaration(&mut self, _it: &TSModuleDeclaration<'a>) {}
}

/// Run lowering's two AST pre-passes over a function in one traversal:
/// build the identifier location index and find the context identifiers
/// (variables captured across function boundaries).
///
/// Walks the function's params (`FormalParameters`) and body, mirroring the
/// original Babel walks (like Babel's `func.traverse()`): the function node
/// itself is not re-entered — its own name, if any, is recorded explicitly
/// (loc-only), and it is never pushed onto the context pass's `function_stack`.
pub fn run_pre_passes(
    func: &FunctionNode<'_, '_>,
    scope: &ScopeResolver<'_, '_>,
) -> Result<(IdentifierLocIndex, FxHashSet<SymbolId>), OxcDiagnostic> {
    let func_scope = func.scope_id().unwrap_or_else(|| scope.program_scope());

    let mut visitor = PrePassVisitor {
        loc: IdentifierLocVisitor::default(),
        ctx: ContextIdentifierVisitor::new(scope, func_scope),
    };

    match func {
        FunctionNode::Function(f) => {
            // The function's own name is a declaration name (loc-only).
            if let Some(id) = &f.id {
                visitor.loc.record_declaration(id);
            }
            if let Some(type_parameters) = &f.type_parameters {
                visitor.visit_ts_type_parameter_declaration(type_parameters);
            }
            if let Some(this_param) = &f.this_param {
                visitor.visit_ts_this_parameter(this_param);
            }
            visitor.visit_formal_parameters(&f.params);
            if let Some(return_type) = &f.return_type {
                visitor.visit_ts_type_annotation(return_type);
            }
            if let Some(body) = &f.body {
                visitor.visit_function_body(body);
            }
        }
        FunctionNode::Arrow(arrow) => {
            if let Some(type_parameters) = &arrow.type_parameters {
                visitor.visit_ts_type_parameter_declaration(type_parameters);
            }
            visitor.visit_formal_parameters(&arrow.params);
            if let Some(return_type) = &arrow.return_type {
                visitor.visit_ts_type_annotation(return_type);
            }
            if arrow.expression {
                if let Some(Statement::ExpressionStatement(es)) = arrow.body.statements.first() {
                    visitor.visit_expression(&es.expression);
                } else {
                    visitor.visit_function_body(&arrow.body);
                }
            } else {
                visitor.visit_function_body(&arrow.body);
            }
        }
    }

    let PrePassVisitor { loc, ctx } = visitor;
    let identifier_spans = loc.index;
    let context_identifiers = find_context_identifiers(ctx, &identifier_spans)?;
    Ok((identifier_spans, context_identifiers))
}
