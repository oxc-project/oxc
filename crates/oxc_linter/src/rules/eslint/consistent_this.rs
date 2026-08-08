use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, BindingPattern, Expression, ForStatementInit, ForStatementLeft,
        match_simple_assignment_target,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use oxc_syntax::{operator::AssignmentOperator, scope::ScopeId, symbol::SymbolId};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::unique_non_empty_string_spread_schema,
};

fn alias_not_assigned_to_this(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Designated alias '{name}' is not assigned to 'this'."))
        .with_label(span)
}

fn unexpected_alias(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected alias '{name}' for 'this'.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentThis(Box<ConsistentThisConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(try_from = "Vec<CompactStr>")]
#[schemars(transparent)]
struct ConsistentThisConfig(
    #[schemars(schema_with = "unique_non_empty_string_spread_schema")] Box<[CompactStr]>,
);

impl ConsistentThisConfig {
    #[inline]
    fn aliases(&self) -> &[CompactStr] {
        &self.0
    }
}

impl Default for ConsistentThisConfig {
    fn default() -> Self {
        Self(vec![CompactStr::new_const("that")].into_boxed_slice())
    }
}

impl TryFrom<Vec<CompactStr>> for ConsistentThisConfig {
    type Error = &'static str;

    fn try_from(aliases: Vec<CompactStr>) -> Result<Self, Self::Error> {
        if aliases.is_empty() {
            return Ok(Self::default());
        }

        for (index, alias) in aliases.iter().enumerate() {
            if alias.is_empty() {
                return Err("alias names must not be empty");
            }

            if aliases[..index].iter().any(|existing| existing == alias) {
                return Err("alias names must be unique");
            }
        }

        Ok(Self(aliases.into_boxed_slice()))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent naming when capturing the current execution context.
    ///
    /// This rule requires variables with configured alias names to be initialized
    /// or assigned to `this` in the same scope. It also reports initializing or
    /// assigning `this` to variables whose names are not configured aliases.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript code sometimes captures `this` in a variable so callbacks can
    /// refer to the original execution context. Mixing aliases such as `that`,
    /// `self`, and `me` makes this pattern harder to read consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"that"` alias:
    /// ```js
    /// var self = this;
    /// var that = window;
    ///
    /// function outer() {
    ///   var that;
    ///   function inner() {
    ///     that = this;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"that"` alias:
    /// ```js
    /// var that = this;
    /// var self = window;
    ///
    /// function f() {
    ///   var that;
    ///   that = this;
    /// }
    ///
    /// var foo = {};
    /// foo.bar = this;
    /// ```
    ///
    /// ### Differences compared to `eslint/consistent-this`
    ///
    /// On TypeScript files this rule looks through type-only wrappers — `as`,
    /// `satisfies`, `<T>` and `!` — on both sides of an assignment, because they
    /// do not change which value is captured.
    ///
    /// ESLint tests `value.type === "ThisExpression"` literally, so it reports
    /// `var self = this as any`, where the alias really is assigned `this`, and stays
    /// silent on `var context = this as any`, where `this` is captured under a name
    /// that is not a configured alias. Its rule is marked `frozen: true` upstream,
    /// so neither will be fixed there.
    ///
    /// ```ts
    /// /* consistent-this: ["error", "self"] */
    ///
    /// var self = this as any;    // allowed here, reported by ESLint
    /// var context = this!;       // reported here, allowed by ESLint
    /// ```
    ///
    /// JavaScript behaviour is unchanged and reports exactly the same code as ESLint.
    ConsistentThis,
    eslint,
    style,
    none,
    config = ConsistentThisConfig,
    version = "next",
    short_description = "Enforce consistent naming when capturing the current execution context.",
);

impl ConsistentThis {
    #[inline]
    fn is_alias(&self, name: &str) -> bool {
        self.0.aliases().iter().any(|alias| alias == name)
    }

    fn check_assignment(
        &self,
        ctx: &LintContext<'_>,
        span: Span,
        name: &str,
        rhs: &Expression<'_>,
        operator: Option<AssignmentOperator>,
    ) {
        let is_this = is_this_expression(rhs);

        if self.is_alias(name) {
            if !is_this || operator.is_some_and(|operator| operator != AssignmentOperator::Assign) {
                ctx.diagnostic(alias_not_assigned_to_this(span, name));
            }
        } else if is_this {
            ctx.diagnostic(unexpected_alias(span, name));
        }
    }

    fn check_was_assigned(&self, ctx: &LintContext<'_>, scope_id: ScopeId) {
        let scoping = ctx.scoping();

        for alias in self.0.aliases() {
            let alias = alias.as_str();
            let Some(symbol_id) = scoping.get_binding(scope_id, alias.into()) else {
                continue;
            };

            if is_named_function_expression_symbol(ctx, symbol_id) {
                continue;
            }

            if scoping
                .symbol_declarations(symbol_id)
                .any(|node_id| is_initialized_variable_declarator(ctx, node_id))
            {
                continue;
            }

            if has_same_scope_this_assignment(ctx, symbol_id, scope_id) {
                continue;
            }

            for node_id in scoping.symbol_declarations(symbol_id) {
                if is_unchecked_block_function_declaration(ctx, node_id, scope_id) {
                    continue;
                }

                let node = ctx.nodes().get_node(node_id);
                ctx.diagnostic(alias_not_assigned_to_this(node.kind().span(), alias));
            }
        }
    }
}

impl Rule for ConsistentThis {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<ConsistentThisConfig>>(value)
            .map(TupleRuleConfig::into_inner)
            .map(Box::new)
            .map(Self)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                let Some(init) = &decl.init else { return };
                let BindingPattern::BindingIdentifier(id) = &decl.id else {
                    return;
                };

                self.check_assignment(ctx, decl.span, id.name.as_str(), init, None);
            }
            AstKind::AssignmentExpression(assignment) => {
                let Some(name) = assignment_target_name(&assignment.left) else { return };

                self.check_assignment(
                    ctx,
                    assignment.span,
                    name,
                    &assignment.right,
                    Some(assignment.operator),
                );
            }
            AstKind::Program(program) => self.check_was_assigned(ctx, program.scope_id()),
            AstKind::Function(func) if func.body.is_some() => {
                self.check_was_assigned(ctx, func.scope_id());
            }
            _ => {}
        }
    }
}

#[inline]
fn is_this_expression(expr: &Expression<'_>) -> bool {
    matches!(expr.get_inner_expression(), Expression::ThisExpression(_))
}

fn assignment_target_name<'t>(target: &'t AssignmentTarget<'_>) -> Option<&'t str> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => Some(id.name.as_str()),
        target @ match_simple_assignment_target!(AssignmentTarget) => target
            .to_simple_assignment_target()
            .get_expression()?
            .get_identifier_reference()
            .map(|id| id.name.as_str()),
        _ => None,
    }
}

/// escope keeps a named function expression's own name in a separate scope that
/// `ensureWasAssigned` never reads; oxc binds it in the function scope, so skip it.
fn is_named_function_expression_symbol(ctx: &LintContext<'_>, symbol_id: SymbolId) -> bool {
    ctx.scoping().symbol_declarations(symbol_id).all(|node_id| {
        let node = ctx.nodes().get_node(node_id);
        match node.kind() {
            AstKind::Function(func) => func.is_expression(),
            _ => false,
        }
    })
}

fn is_initialized_variable_declarator(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    matches!(
        ctx.nodes().kind(node_id),
        AstKind::VariableDeclarator(decl) if decl.init.is_some()
    )
}

/// oxc models Annex-B hoisting, so a block-level `function` has its binding moved into the
/// enclosing var scope; escope does not, and ESLint never checks the owning block scope.
fn is_unchecked_block_function_declaration(
    ctx: &LintContext<'_>,
    declaration_id: NodeId,
    checked_scope_id: ScopeId,
) -> bool {
    if !matches!(
        ctx.nodes().kind(declaration_id),
        AstKind::Function(function) if function.is_declaration()
    ) {
        return false;
    }

    let checked_node_id = ctx.scoping().get_node_id(checked_scope_id);
    for (ancestor_id, ancestor) in ctx.nodes().ancestors_enumerated(declaration_id) {
        if ancestor_id == checked_node_id {
            return false;
        }

        match ancestor.kind() {
            AstKind::BlockStatement(_) | AstKind::SwitchStatement(_) => return true,
            AstKind::Program(_) | AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                return false;
            }
            _ => {}
        }
    }

    false
}

fn has_same_scope_this_assignment(
    ctx: &LintContext<'_>,
    symbol_id: SymbolId,
    checked_scope_id: ScopeId,
) -> bool {
    ctx.scoping().get_resolved_references(symbol_id).any(|reference| {
        reference.is_write()
            && reference_matches_eslint_scope(ctx, reference.scope_id(), checked_scope_id)
            && reference_write_is_plain_this_assignment(ctx, reference.node_id())
    })
}

fn reference_matches_eslint_scope(
    ctx: &LintContext<'_>,
    reference_scope_id: ScopeId,
    checked_scope_id: ScopeId,
) -> bool {
    if reference_scope_id == checked_scope_id {
        return true;
    }

    let scoping = ctx.scoping();
    let mut scope_id = reference_scope_id;

    while is_eslint_transparent_scope(ctx, scope_id) {
        let Some(parent_scope_id) = scoping.scope_parent_id(scope_id) else {
            return false;
        };

        if parent_scope_id == checked_scope_id {
            return true;
        }

        scope_id = parent_scope_id;
    }

    false
}

/// oxc creates a scope for every `for`; escope only for lexical heads (`let`, `const`,
/// `using`).
/// Non-lexical `for` scopes are therefore transparent when matching `reference.from`.
fn is_eslint_transparent_scope(ctx: &LintContext<'_>, scope_id: ScopeId) -> bool {
    match ctx.nodes().kind(ctx.scoping().get_node_id(scope_id)) {
        AstKind::ForStatement(stmt) => {
            !stmt.init.as_ref().is_some_and(for_statement_init_has_lexical_declaration)
        }
        AstKind::ForInStatement(stmt) => !for_statement_left_has_lexical_declaration(&stmt.left),
        AstKind::ForOfStatement(stmt) => !for_statement_left_has_lexical_declaration(&stmt.left),
        _ => false,
    }
}

fn for_statement_init_has_lexical_declaration(init: &ForStatementInit<'_>) -> bool {
    matches!(init, ForStatementInit::VariableDeclaration(decl) if decl.kind.is_lexical())
}

fn for_statement_left_has_lexical_declaration(left: &ForStatementLeft<'_>) -> bool {
    matches!(left, ForStatementLeft::VariableDeclaration(decl) if decl.kind.is_lexical())
}

fn reference_write_is_plain_this_assignment(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    let mut node = ctx.nodes().get_node(node_id);

    loop {
        let parent = ctx.nodes().parent_node(node.id());

        match parent.kind() {
            AstKind::AssignmentExpression(assignment) => {
                return assignment.operator == AssignmentOperator::Assign
                    && is_this_expression(&assignment.right);
            }
            // escope never credits these writes to an enclosing assignment: `writeExpr` is
            // null for updates, and for `for-in`/`for-of` heads it is the loop RHS, whose
            // parent is the loop and therefore has no `=` operator.
            AstKind::UpdateExpression(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::Program(_)
            | AstKind::Function(_) => return false,
            _ => node = parent,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let self_config = Some(serde_json::json!(["self"]));
    let that_config = Some(serde_json::json!(["that"]));
    let multi_alias_config = Some(serde_json::json!(["self", "vm"]));

    let pass = vec![
        ("var foo = 42, that = this", None),
        ("var foo = 42, self = this", self_config.clone()),
        ("var self = 42", that_config.clone()),
        ("var self", that_config.clone()),
        ("var self; self = this", self_config.clone()),
        ("var foo, self; self = this", self_config.clone()),
        ("var foo, self; foo = 42; self = this", self_config.clone()),
        ("self = 42", that_config.clone()),
        ("var foo = {}; foo.bar = this", self_config.clone()),
        ("var self = this; var vm = this;", multi_alias_config),
        ("var {foo, bar} = this", self_config.clone()),
        ("({foo, bar} = this)", self_config.clone()),
        ("var [foo, bar] = this", self_config.clone()),
        ("[foo, bar] = this", self_config.clone()),
        ("var self = (this)", self_config.clone()),
        ("var context = this.foo", self_config.clone()),
        ("self = this", self_config.clone()),
        ("self = (this)", self_config.clone()),
        ("obj.self = this", self_config.clone()),
        ("this.self = this", self_config.clone()),
        ("self++", self_config.clone()),
        ("self = this; var self", self_config.clone()),
        ("var self; var self = this", self_config.clone()),
        ("function f(){ var self; self = this }", self_config.clone()),
        ("var self; if (a) self = this", self_config.clone()),
        ("(() => { var self; })()", self_config.clone()),
        ("(() => { let self; })()", self_config.clone()),
        ("(() => { var self; self = this; })()", self_config.clone()),
        ("(self) => {}", self_config.clone()),
        ("(self) => { self = this }", self_config.clone()),
        ("(self = this) => {}", self_config.clone()),
        ("({ self }) => {}", self_config.clone()),
        ("([self]) => {}", self_config.clone()),
        ("function f(self){ self = this }", self_config.clone()),
        ("function f(self = this){ self = this }", self_config.clone()),
        ("function f({ self }){ self = this }", self_config.clone()),
        ("function f([self]){ self = this }", self_config.clone()),
        ("function self(){} self = this", self_config.clone()),
        ("var f = function self(){}", self_config.clone()),
        ("var f = function self(){ self = this }", self_config.clone()),
        ("var f = function self(self){ self = this }", self_config.clone()),
        ("var f = function self(){ var self; self = this }", self_config.clone()),
        ("class self {} self = this", self_config.clone()),
        ("var C = class self {}", self_config.clone()),
        ("var o = { m(){ var self; self = this } }", self_config.clone()),
        ("class C { m(){ var self; self = this } }", self_config.clone()),
        ("try{} catch(self){}", self_config.clone()),
        ("try{} catch(self){ self = this }", self_config.clone()),
        ("if (a) { let self; }", self_config.clone()),
        ("for (let self;;) {}", self_config.clone()),
        ("for (let self of xs) {}", self_config.clone()),
        ("var self; for (self = this;;) break", self_config.clone()),
        ("var self; for (;;) self = this", self_config.clone()),
        ("var self; for (; self = this;) break", self_config.clone()),
        ("var self; for (;; self = this) break", self_config.clone()),
        ("var self; for (;; self = this) {}", self_config.clone()),
        ("var self; for (var i = 0; self = this;) break", self_config.clone()),
        ("var self; for (var i = 0;; self = this) {}", self_config.clone()),
        ("var self; for (var i = (self = this);;) {}", self_config.clone()),
        ("var self; for (;;) for (;;) self = this", self_config.clone()),
        ("var self; for (key in obj) self = this", self_config.clone()),
        ("var self; for (var key in obj) self = this", self_config.clone()),
        ("var self; for (x of xs) self = this", self_config.clone()),
        ("var self; for (var x of xs) self = this", self_config.clone()),
        ("var self; while (foo) self = this", self_config.clone()),
        ("var self; do self = this; while (foo)", self_config.clone()),
        ("label: { function self() {} }", self_config.clone()),
        ("if (foo) { function self() {} }", self_config.clone()),
        ("function f() { if (foo) { function self() {} } }", self_config.clone()),
        ("switch (foo) { case 1: function self() {} }", self_config.clone()),
        ("for (;;) { function self() {} }", self_config.clone()),
        ("(() => { if (foo) { function self() {} } })()", self_config.clone()),
        ("var { self } = obj", self_config.clone()),
        ("var { self } = this", self_config.clone()),
        ("var [self] = arr", self_config.clone()),
        ("({ self } = this)", self_config.clone()),
        ("var self; ({ self } = this)", self_config.clone()),
        ("var self; [self] = this", self_config.clone()),
        ("var self; ({ self = obj } = this)", self_config.clone()),
        ("var self; ({ self = this } = this)", self_config.clone()),
        ("var self; ({ x: self } = this)", self_config.clone()),
        ("var self; ({ x: self = obj } = this)", self_config.clone()),
        ("var self; ([, self] = this)", self_config.clone()),
        ("({ context } = this)", self_config.clone()),
        ("var self; if (a) { self = this } self = this", self_config.clone()),
        ("var self; if (a) { self = this } else self = this", self_config.clone()),
        ("function f(){ return function(){ var self; self = this } }", self_config.clone()),
        ("class C { field = (() => { var self; })() }", self_config.clone()),
        ("class C { field = function(){ var self; self = this } }", self_config.clone()),
        ("class C { static { var self } }", self_config.clone()),
        ("class C { static { let self; } }", self_config.clone()),
        ("class C { static { self = this; } }", self_config.clone()),
        ("class C { field = this }", self_config.clone()),
        ("class C { field = self = this }", self_config.clone()),
        ("switch (a) { default: function self() {} }", self_config.clone()),
        ("function f() { { function self() {} } }", self_config.clone()),
        ("try {} catch ({ self }) {}", self_config.clone()),
        ("try {} catch ({ self }) { self = this }", self_config.clone()),
        ("var self; a ? (self = this) : (self = this);", self_config.clone()),
        ("var self; (0, self = this);", self_config.clone()),
        ("var self; a && (self = this);", self_config.clone()),
        ("var self = this; var self = this;", self_config.clone()),
        ("var self = this; var self;", self_config.clone()),
        ("function self() {} var self = this;", self_config.clone()),
        ("self = this;", self_config.clone()),
        ("self.foo = this;", self_config.clone()),
        ("var self; (self) = this;", self_config.clone()),
        ("var self; ((self)) = this;", self_config.clone()),
        ("var o = { self: this };", self_config.clone()),
        ("var o = { self() { return this } };", self_config.clone()),
        ("class C { static self = this; }", self_config.clone()),
        ("class C { #self = this; }", self_config.clone()),
        ("class C { ['self'] = this; }", self_config.clone()),
        ("self: for (;;) { break self; }", self_config.clone()),
        ("var self; ({ ...self } = this);", self_config.clone()),
        ("var self; [...self] = this;", self_config.clone()),
        ("var self = this; { var self; }", self_config.clone()),
        ("if (0) { var self = this; }", self_config.clone()),
        ("var { self = this } = {};", self_config.clone()),
        ("var self; (self = this, foo());", self_config.clone()),
        ("var self = this; self = this;", self_config.clone()),
        ("let self = this;", self_config.clone()),
        ("const self = this;", self_config.clone()),
        ("var self; for (var i in (self = this, {})) {}", self_config.clone()),
        ("var self; ({ a: { self } } = this);", self_config.clone()),
        ("var self; [[self]] = this;", self_config.clone()),
        ("async function f() { var self; self = this; }", self_config.clone()),
        ("function* g() { var self; self = this; }", self_config.clone()),
        ("var o = { get self() { return this } };", self_config.clone()),
        ("globalThis.self = this;", self_config.clone()),
        ("var self; { var self = this; }", self_config.clone()),
        ("var self; x = self = this;", self_config.clone()),
        ("var self; obj[self = this] = 1;", self_config.clone()),
        ("var self; obj[self = this]++;", self_config.clone()),
        ("var self; obj[self++] = this; self = this;", self_config.clone()),
    ];

    let fail = vec![
        ("var context = this", None),
        ("var that = this", self_config.clone()),
        ("var foo = 42, self = this", that_config.clone()),
        ("var self = 42", self_config.clone()),
        ("var self", self_config.clone()),
        ("var self; self = 42", self_config.clone()),
        ("context = this", that_config.clone()),
        ("that = this", self_config.clone()),
        ("self = this", that_config),
        ("self += this", self_config.clone()),
        ("var self; (function() { self = this; }())", self_config.clone()),
        ("var context = (this)", self_config.clone()),
        ("var self = this.foo", self_config.clone()),
        ("var self = this || foo", self_config.clone()),
        ("var self = () => this", self_config.clone()),
        ("self = 42", self_config.clone()),
        ("self = this.foo", self_config.clone()),
        ("self = this || foo", self_config.clone()),
        ("self ||= this", self_config.clone()),
        ("foo += this", self_config.clone()),
        ("var self = this; self = 42", self_config.clone()),
        ("var self; self = 42; self = this", self_config.clone()),
        ("var self; self += this", self_config.clone()),
        ("var self; var self", self_config.clone()),
        ("var self; var self = 42", self_config.clone()),
        ("var self; function f(){ self = this }", self_config.clone()),
        ("function f(){ var self; function g(){ self = this } }", self_config.clone()),
        ("var self; if (a) { self = this }", self_config.clone()),
        ("var self; { self = this }", self_config.clone()),
        ("var self; class C { static { self = this } }", self_config.clone()),
        ("(() => { var self = 42; })()", self_config.clone()),
        ("(() => { var self; self = 42; })()", self_config.clone()),
        ("(self) => { self = 42 }", self_config.clone()),
        ("({ self }) => { self = 42 }", self_config.clone()),
        ("([self]) => { self = 42 }", self_config.clone()),
        ("function f(self){}", self_config.clone()),
        ("function f(self = this){}", self_config.clone()),
        ("function f(self = 42){}", self_config.clone()),
        ("function f({ self }){}", self_config.clone()),
        ("function f([self]){}", self_config.clone()),
        ("function f(self){ self = 42 }", self_config.clone()),
        ("function f({ self }){ self = 42 }", self_config.clone()),
        ("function f(self){ var self; }", self_config.clone()),
        ("function f(self){ var self = 42; }", self_config.clone()),
        ("function self(){}", self_config.clone()),
        ("function self(){ self = this }", self_config.clone()),
        ("var f = function self(){ self = 42 }", self_config.clone()),
        ("var f = function self(){ var self; }", self_config.clone()),
        ("var f = function self(self){}", self_config.clone()),
        ("var f = function self(self){ self = 42 }", self_config.clone()),
        ("class self {}", self_config.clone()),
        ("var o = { m(){ var self; } }", self_config.clone()),
        ("class C { m(){ var self; } }", self_config.clone()),
        ("try{} catch(self){ self = 42 }", self_config.clone()),
        ("if (a) { let self = 42; }", self_config.clone()),
        ("if (a) { var self; }", self_config.clone()),
        ("for (let self = 0;;) {}", self_config.clone()),
        ("for (var self;;) {}", self_config.clone()),
        ("for (var self of xs) {}", self_config.clone()),
        ("var self; for (;;) { self = this; break }", self_config.clone()),
        ("var self; for (let i = 0; self = this;) break", self_config.clone()),
        ("var self; for (let i = 0;; self = this) {}", self_config.clone()),
        ("var self; for (let i = (self = this);;) {}", self_config.clone()),
        ("var self; for (let x of xs) self = this", self_config.clone()),
        ("var self; for (const x in xs) self = this", self_config.clone()),
        ("var self; for (let key in (self = this, obj)) {}", self_config.clone()),
        ("var self; for (using x of xs) self = this", self_config.clone()),
        ("var self; switch(x){ case 1: self = this }", self_config.clone()),
        ("var self; try { self = this } catch(e){}", self_config.clone()),
        ("var self; try {} catch { self = this; }", self_config.clone()),
        ("var self; try {} catch (e) { self = this; }", self_config.clone()),
        ("var self; with (obj) { self = this }", self_config.clone()),
        ("function f() { var self; (() => { self = this; })(); }", self_config.clone()),
        ("if (foo) { function self() {} } var self;", self_config.clone()),
        ("if (foo) function self() {}", self_config.clone()),
        ("label: function self() {}", self_config.clone()),
        ("var self; ({ self } = obj)", self_config.clone()),
        ("var self; [self] = obj", self_config.clone()),
        ("var self; ({ self = this } = obj)", self_config.clone()),
        ("var self; ({ x: self = this } = obj)", self_config.clone()),
        ("var self; [self = this] = obj", self_config.clone()),
        ("var self; ([, self] = obj)", self_config.clone()),
        ("var self; while (foo) { self = this; break }", self_config.clone()),
        ("var self; do { self = this; break } while (foo)", self_config.clone()),
        ("var self; if (a) { self = this } else { self = this }", self_config.clone()),
        ("var self; try { self = this } finally { self = this }", self_config.clone()),
        ("function f(){ return function(){ var self; } }", self_config.clone()),
        ("var self; (() => self = this)()", self_config.clone()),
        ("var self; (() => ({ self } = this))()", self_config.clone()),
        ("var self; class C { method(){ self = this } }", self_config.clone()),
        ("function f(){ var self; class C { m(){ self = this } } }", self_config.clone()),
        ("class C { static { var self = 42; } }", self_config.clone()),
        ("class C { static { self = 42; } }", self_config.clone()),
        ("class C { field = self = 42 }", self_config.clone()),
        ("class C { field = (() => { var self; self = 42; })() }", self_config.clone()),
        ("class C { field = function(){ var self; } }", self_config.clone()),
        ("var self; class C { field = self = this }", self_config.clone()),
        ("var self; class C { field = (self = this); }", self_config.clone()),
        ("var self; class C { [self = this] = 1; }", self_config.clone()),
        ("var self; class C extends (self = this) {}", self_config.clone()),
        (
            "function f() { var self; class C { field = (() => { self = this; })(); } }",
            self_config.clone(),
        ),
        ("var self; function f(){ var self; }", self_config.clone()),
        ("var self; with (o) { self = this; }", self_config.clone()),
        ("var self; with (o) { var self; }", self_config.clone()),
        ("function f() { var self; with (o) { self = this; } }", self_config.clone()),
        ("label: label2: function self() {}", self_config.clone()),
        ("function f() { { function self() {} } var self; }", self_config.clone()),
        ("var self; try { self = this } catch (self) { self = this }", self_config.clone()),
        ("var self; do { self = this } while (0)", self_config.clone()),
        ("var self; for (const [self] of xs) {}", self_config.clone()),
        ("var self; for (let { self } in o) {}", self_config.clone()),
        ("var self; for (var self of xs) {}", self_config.clone()),
        ("var self = this; function f() { var self; }", self_config.clone()),
        ("function f() { var self; { let self = 1; } }", self_config.clone()),
        ("function f() { let self; { var other; } }", self_config.clone()),
        ("var self; (() => (() => { self = this })())();", self_config.clone()),
        ("function f() { var self; return () => { self = this; }; }", self_config.clone()),
        ("var self; new Function('self = this');", self_config.clone()),
        ("var self; eval('self = this');", self_config.clone()),
        ("function self() {} function self() {}", self_config.clone()),
        ("var self; function self() {}", self_config.clone()),
        ("var self; import('x');", self_config.clone()),
        ("var self; self.foo = this;", self_config.clone()),
        ("var self; this.self = this;", self_config.clone()),
        ("var self = this?.foo;", self_config.clone()),
        ("var self = (this, this);", self_config.clone()),
        ("var self = void this;", self_config.clone()),
        ("var self = !this;", self_config.clone()),
        ("var self = this, other = this;", self_config.clone()),
        ("var self; self ??= this;", self_config.clone()),
        ("function f() { 'use strict'; var self; }", self_config.clone()),
        ("var self = this ? a : b;", self_config.clone()),
        ("class self { static { self = this; } }", self_config.clone()),
        ("var self; delete self;", self_config.clone()),
        ("for (var self in this) {}", self_config.clone()),
        ("var self; queueMicrotask(function () { self = this; });", self_config.clone()),
        ("var self = globalThis;", self_config.clone()),
        ("var self; obj[self++] = this;", self_config.clone()),
        ("var self; obj[--self] = this;", self_config.clone()),
        ("var self; ({ [self++]: x } = this);", self_config.clone()),
        ("var self; [obj[self++]] = this;", self_config.clone()),
        ("var self; self++;", self_config.clone()),
        ("var self; for (self in obj) {}", self_config.clone()),
        ("var self; for (self of obj) {}", self_config.clone()),
        ("var self; for (self of this) {}", self_config.clone()),
        ("var self; for (self in this) {}", self_config.clone()),
        ("var self; for ([self] of [this]) {}", self_config.clone()),
        ("var self; for (obj[self++] of [this]) {}", self_config.clone()),
        ("var self; obj[self] = this;", self_config.clone()),
        ("var self; self &&= this;", self_config.clone()),
        ("var self; self = x = this;", self_config.clone()),
        ("var self; with (o) self = this;", self_config.clone()),
    ];

    Tester::new(ConsistentThis::NAME, ConsistentThis::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .test_and_snapshot();

    let pass = vec![
        ("var self = this as any", self_config.clone()),
        ("var self = <any>this", self_config.clone()),
        ("var self = this satisfies unknown", self_config.clone()),
        ("var self = this!", self_config.clone()),
        ("var self = (this as any)!", self_config.clone()),
        ("var self = this as unknown as any;", self_config.clone()),
        ("var self = this satisfies unknown satisfies unknown;", self_config.clone()),
        ("self = this as any", self_config.clone()),
        ("var self; self = this as any", self_config.clone()),
        ("var self; (self as any) = this", self_config.clone()),
        ("var self; (self!) = this", self_config.clone()),
        ("var self: unknown; self = this", self_config.clone()),
        ("var context = this.foo as any", self_config.clone()),
        ("function f(self: unknown){ self = this }", self_config.clone()),
        ("function f({ self }: { self: unknown }){ self = this }", self_config.clone()),
        ("(self: unknown) => {}", self_config.clone()),
        ("class C<self> {}", self_config.clone()),
        ("type T<self> = self;", self_config.clone()),
        ("type self = number; var self = this;", self_config.clone()),
        ("interface self {} var self = this;", self_config.clone()),
        ("declare global { var self: any; }", self_config.clone()),
        ("enum E { self }", self_config.clone()),
        ("class C { self = this; }", self_config.clone()),
        ("class C { accessor self = this; }", self_config.clone()),
        ("declare module 'x' { const self: number; }", self_config.clone()),
        ("namespace N { var self; }", self_config.clone()),
        ("let self!: any; self = this;", self_config.clone()),
        ("function f() { using self = this; }", self_config.clone()),
        ("async function f() { await using self = this; }", self_config.clone()),
        ("declare function f(self: any): void;", self_config.clone()),
        ("declare class C { m(self: any): void; }", self_config.clone()),
        ("declare namespace N { function f(self: any): void; }", self_config.clone()),
        ("declare const f: (self: any) => void;", self_config.clone()),
        ("abstract class C { abstract m(self: any): void; }", self_config.clone()),
        ("class C { m(self: any): void; m(): void {} }", self_config.clone()),
        ("interface I { m(self: any): void; }", self_config.clone()),
        ("type F = (self: any) => void;", self_config.clone()),
    ];

    let fail = vec![
        // Deliberate divergence: type-only wrappers are transparent, see the rule docs.
        ("var context = this as any", self_config.clone()),
        ("var context = <any>this", self_config.clone()),
        ("var context = this satisfies unknown", self_config.clone()),
        ("var context = this!", self_config.clone()),
        ("context = this as any", self_config.clone()),
        ("(context as any) = this", self_config.clone()),
        ("var self; (self as any) = 42", self_config.clone()),
        // Below: verified identical to eslint@9 + @typescript-eslint/parser@8.
        ("var self = this.foo as any", self_config.clone()),
        ("var self = foo as any", self_config.clone()),
        ("function f(self: unknown){}", self_config.clone()),
        ("function f(self?: unknown){}", self_config.clone()),
        ("function f(self: unknown){ self = 42 }", self_config.clone()),
        ("function f(self: any): void {}", self_config.clone()),
        ("type self = number;", self_config.clone()),
        ("interface self {}", self_config.clone()),
        ("enum self { A }", self_config.clone()),
        ("const enum self { A }", self_config.clone()),
        ("namespace self { export const x = 1; }", self_config.clone()),
        ("declare namespace self { const x: number; }", self_config.clone()),
        ("declare var self: any;", self_config.clone()),
        ("declare let self: any;", self_config.clone()),
        ("declare const self: any;", self_config.clone()),
        ("declare function self(): void;", self_config.clone()),
        ("declare class self {}", self_config.clone()),
        ("abstract class self {}", self_config.clone()),
        ("function f<self>(): void {}", self_config.clone()),
        ("import type self from 'x';", self_config.clone()),
        ("import { type x as self } from 'x';", self_config.clone()),
        ("function self(): void; function self(): void {}", self_config.clone()),
        ("class C { constructor(private self: any) {} }", self_config.clone()),
        ("export type self = number;", self_config.clone()),
        ("function f(this: any, self: number) {}", self_config.clone()),
        ("class C { m<self>(): void {} }", self_config.clone()),
        ("let self: any;", self_config.clone()),
        ("function f(): void { type self = number; }", self_config.clone()),
        ("function f(): void { interface self {} }", self_config.clone()),
        ("function f(): void { enum self { A } }", self_config.clone()),
    ];

    Tester::new(ConsistentThis::NAME, ConsistentThis::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();

    let pass = vec![
        ("import self from 'x'; self = this", self_config.clone()),
        ("import { x as self } from 'x'; self = this", self_config.clone()),
        ("import * as self from 'x'; self = this", self_config.clone()),
        ("if (foo) { function self() {} }", self_config.clone()),
        ("export const self = this;", self_config.clone()),
    ];

    let fail = vec![
        ("var self; (function() { self = this; }())", self_config.clone()),
        ("import self from 'x'; self = 42", self_config.clone()),
        ("import { x as self } from 'x'; self = 42", self_config.clone()),
        ("import self from 'x'", self_config.clone()),
        ("import { x as self } from 'x'", self_config.clone()),
        ("import * as self from 'x'", self_config.clone()),
        ("export var self;", self_config.clone()),
        ("export default function self() {}", self_config.clone()),
        ("export function self() {}", self_config.clone()),
        ("import 'x'; var self;", self_config.clone()),
    ];

    Tester::new(ConsistentThis::NAME, ConsistentThis::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .with_snapshot_suffix("module")
        .test_and_snapshot();

    let pass = vec![
        ("var self = this as any;", self_config.clone()),
        ("class C { @dec self = this; }", self_config.clone()),
        ("var self = this!;", self_config.clone()),
        ("var self; (self as any)! = this;", self_config.clone()),
        ("var self; (self satisfies any) = this;", self_config.clone()),
    ];

    let fail = vec![
        ("const self = <div />;", self_config.clone()),
        ("var context = this as any;", self_config.clone()),
        ("function f<self,>(): void {}", self_config.clone()),
        ("@dec class self {}", self_config),
    ];

    Tester::new(ConsistentThis::NAME, ConsistentThis::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .with_snapshot_suffix("tsx")
        .test_and_snapshot();
}

#[test]
fn test_config() {
    assert_eq!(ConsistentThis::default().0.aliases(), [CompactStr::new("that")]);

    let configured = ConsistentThis::from_configuration(serde_json::json!(["self", "vm"]))
        .expect("valid aliases");
    assert_eq!(configured.0.aliases(), [CompactStr::new("self"), CompactStr::new("vm")]);

    assert!(ConsistentThis::from_configuration(serde_json::Value::Null).is_ok());
    assert!(ConsistentThis::from_configuration(serde_json::json!([])).is_ok());
    assert!(ConsistentThis::from_configuration(serde_json::json!([""])).is_err());
    assert!(ConsistentThis::from_configuration(serde_json::json!(["self", "self"])).is_err());
    assert!(ConsistentThis::from_configuration(serde_json::json!([42])).is_err());
    assert!(ConsistentThis::from_configuration(serde_json::json!({"alias": "self"})).is_err());
}
