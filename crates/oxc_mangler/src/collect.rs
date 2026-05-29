//! Collects, in a single AST traversal, the information the mangler needs that is otherwise only
//! available from `oxc_semantic`'s flattened `AstNodes` vec:
//!
//! - the lexical declaration scopes of hoisted symbols (`var` / Annex B functions), which differ
//!   from the symbol's registered scope and are needed for slot liveness;
//! - the name symbol of each named function expression, keyed by its own scope, used to repair
//!   orphaned slots;
//! - (when `keep_names` is enabled) the symbols whose `name` must be preserved.
//!
//! This keeps the mangler self-contained - it does not read `Semantic::nodes()`. The traversal
//! maintains a scope stack (so the current lexical scope is known) and, for `keep_names`, a
//! parent-kind stack. Results are stored in pre-sized [`IndexVec`]s for `O(1)` lookup, avoiding
//! the per-node hashing that would otherwise dominate.

use std::cell::Cell;

use rustc_hash::FxHashSet;

use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::Visit;
use oxc_index::IndexVec;
use oxc_semantic::{ReferenceId, ScopeFlags, ScopeId, Scoping, SymbolFlags, SymbolId};

use crate::MangleOptionsKeepNames;

/// Information gathered from a single AST traversal, replacing reads of `Semantic::nodes()`.
///
/// `hoisted_declaration_scopes` is read once per symbol in the (hot) slot-assignment loop, so it is
/// kept as an `IndexVec` for `O(1)` indexing - the common, non-hoisted case is just an empty vec.
pub struct CollectedInfo {
    /// For each symbol, the lexical declaration scope(s) that differ from its registered scope
    /// (i.e. it was hoisted - `var` / Annex B function). Empty for the common, non-hoisted case.
    pub hoisted_declaration_scopes: IndexVec<SymbolId, Vec<ScopeId>>,
    /// For each scope, the name symbol of the named function expression that creates it, if any.
    pub fn_expr_name_symbols: IndexVec<ScopeId, Option<SymbolId>>,
    /// Symbols whose `name` must be preserved. Empty unless `keep_names` is enabled.
    pub keep_name_symbols: FxHashSet<SymbolId>,
}

/// Walk `program` once and collect the information the mangler needs.
pub fn collect(
    keep_names: MangleOptionsKeepNames,
    scoping: &Scoping,
    program: &Program,
) -> CollectedInfo {
    let info = CollectedInfo {
        hoisted_declaration_scopes: std::iter::repeat_with(Vec::new)
            .take(scoping.symbols_len())
            .collect(),
        fn_expr_name_symbols: std::iter::repeat_n(None, scoping.scopes_len()).collect(),
        keep_name_symbols: FxHashSet::default(),
    };
    let mut collector = Collector {
        keep_names,
        scoping,
        scope_stack: Vec::with_capacity(64),
        ancestors: Vec::new(),
        info,
    };
    collector.visit_program(program);
    collector.info
}

struct Collector<'a, 's> {
    keep_names: MangleOptionsKeepNames,
    scoping: &'s Scoping,
    /// Current lexical scope chain; the top is the scope of the node being visited.
    scope_stack: Vec<ScopeId>,
    /// Ancestors of the node being visited (only maintained when `keep_names` is enabled).
    ancestors: Vec<AstKind<'a>>,
    info: CollectedInfo,
}

impl<'a> Collector<'a, '_> {
    #[inline]
    fn keep_names_enabled(&self) -> bool {
        self.keep_names.function || self.keep_names.class
    }

    fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().expect("not inside a scope")
    }

    /// Record `symbol_id`'s lexical declaration scope when it differs from the registered scope -
    /// i.e. it was hoisted. The mangler needs these (and only these) for slot liveness. `scope` is
    /// the scope of the declaration *node* (which, for functions/classes, is the enclosing scope
    /// where the `Function`/`Class` node sits, not the scope its binding identifier is visited in).
    fn record_declaration_scope(&mut self, symbol_id: SymbolId, scope: ScopeId) {
        if scope != self.scoping.symbol_scope_id(symbol_id) {
            self.info.hoisted_declaration_scopes[symbol_id].push(scope);
        }
    }

    /// Mark symbols whose `name` is set by a declaration (`var foo = function () {}`,
    /// `var [foo = function () {}] = []`, ...).
    fn collect_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        // `var foo = function () {}`
        if let BindingPattern::BindingIdentifier(id) = &declarator.id
            && let Some(init) = &declarator.init
            && is_expression_whose_name_needs_to_be_kept(self.keep_names, init)
        {
            self.info.keep_name_symbols.insert(id.symbol_id());
        }
        // Default values in destructuring patterns: `var [foo = function () {}] = []`
        self.collect_pattern_defaults(&declarator.id);
    }

    fn collect_pattern_defaults(&mut self, pattern: &BindingPattern<'a>) {
        match pattern {
            BindingPattern::BindingIdentifier(_) => {}
            BindingPattern::ObjectPattern(object_pattern) => {
                for property in &object_pattern.properties {
                    self.collect_pattern_defaults(&property.value);
                }
            }
            BindingPattern::ArrayPattern(array_pattern) => {
                for element in array_pattern.elements.iter().flatten() {
                    self.collect_pattern_defaults(element);
                }
            }
            BindingPattern::AssignmentPattern(assignment_pattern) => {
                if let BindingPattern::BindingIdentifier(id) = &assignment_pattern.left
                    && is_expression_whose_name_needs_to_be_kept(
                        self.keep_names,
                        &assignment_pattern.right,
                    )
                {
                    self.info.keep_name_symbols.insert(id.symbol_id());
                }
                self.collect_pattern_defaults(&assignment_pattern.left);
            }
        }
    }

    /// Mark a symbol whose `name` is set via assignment to one of its references
    /// (`foo = function () {}`, `[foo = function () {}] = []`, `{ foo = function () {} } = {}`).
    fn collect_reference(&mut self, reference: &IdentifierReference<'a>) {
        let reference_id = reference.reference_id();
        let Some(parent) = self.ancestors.last().copied() else { return };
        let needs_keep = match parent {
            // `foo = function () {}`
            AstKind::AssignmentExpression(assign_expr) => {
                is_assignment_target_id_of_specific_reference(&assign_expr.left, reference_id)
                    && is_expression_whose_name_needs_to_be_kept(
                        self.keep_names,
                        &assign_expr.right,
                    )
            }
            // `[foo = function () {}] = []`
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                is_assignment_target_id_of_specific_reference(&assign_target.binding, reference_id)
                    && is_expression_whose_name_needs_to_be_kept(
                        self.keep_names,
                        &assign_target.init,
                    )
            }
            // The reference may be wrapped; walk one level up to the assignment.
            AstKind::IdentifierReference(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::StaticMemberExpression(_) => {
                match self.ancestors.iter().rev().nth(1).copied() {
                    Some(AstKind::AssignmentExpression(assign_expr)) => {
                        is_assignment_target_id_of_specific_reference(
                            &assign_expr.left,
                            reference_id,
                        ) && is_expression_whose_name_needs_to_be_kept(
                            self.keep_names,
                            &assign_expr.right,
                        )
                    }
                    Some(AstKind::AssignmentTargetWithDefault(assign_target)) => {
                        is_assignment_target_id_of_specific_reference(
                            &assign_target.binding,
                            reference_id,
                        ) && is_expression_whose_name_needs_to_be_kept(
                            self.keep_names,
                            &assign_target.init,
                        )
                    }
                    _ => false,
                }
            }
            // `({ foo = function () {} } = {})`
            AstKind::AssignmentTargetPropertyIdentifier(ident) => {
                ident.binding.reference_id() == reference_id
                    && ident.init.as_ref().is_some_and(|init| {
                        is_expression_whose_name_needs_to_be_kept(self.keep_names, init)
                    })
            }
            _ => false,
        };

        if needs_keep && let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id()
        {
            self.info.keep_name_symbols.insert(symbol_id);
        }
    }
}

impl<'a> Visit<'a> for Collector<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.scope_stack.push(scope_id.get().expect("scope id should be set by semantic analysis"));
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::BindingIdentifier(id) => {
                // Only `var`-scoped bindings (`FunctionScopedVariable`) can be hoisted out of their
                // lexical block, so only they can have a declaration scope differing from the
                // registered one - `let`/`const`/imports/etc. never do, so skip them. Function and
                // class names (which can also carry this flag when a `var` merges with a function)
                // are recorded at their `Function`/`Class` node, where the enclosing declaration
                // scope is current; their binding identifier is visited inside the function/class
                // scope instead, so skip it here.
                if let Some(symbol_id) = id.symbol_id.get() {
                    let flags = self.scoping.symbol_flags(symbol_id);
                    if flags.contains(SymbolFlags::FunctionScopedVariable)
                        && !flags.intersects(SymbolFlags::Function | SymbolFlags::Class)
                    {
                        self.record_declaration_scope(symbol_id, self.current_scope());
                    }
                }
            }
            AstKind::Function(func) => {
                // The `Function` node is visited before its own scope is entered, so `current_scope`
                // is the enclosing scope - matching where `symbol_declaration` points. This records
                // the hoisted scope for Annex B block functions (and is a no-op otherwise).
                if let Some(id) = &func.id {
                    self.record_declaration_scope(id.symbol_id(), self.current_scope());
                }
                if func.is_expression()
                    && let Some(id) = &func.id
                    && let Some(scope_id) = func.scope_id.get()
                {
                    self.info.fn_expr_name_symbols[scope_id] = Some(id.symbol_id());
                }
                if self.keep_names.function
                    && let Some(id) = &func.id
                {
                    self.info.keep_name_symbols.insert(id.symbol_id());
                }
            }
            AstKind::Class(class) => {
                if let Some(id) = &class.id {
                    self.record_declaration_scope(id.symbol_id(), self.current_scope());
                }
                if self.keep_names.class
                    && let Some(id) = &class.id
                {
                    self.info.keep_name_symbols.insert(id.symbol_id());
                }
            }
            AstKind::VariableDeclarator(declarator) if self.keep_names_enabled() => {
                self.collect_declarator(declarator);
            }
            AstKind::IdentifierReference(reference) if self.keep_names_enabled() => {
                self.collect_reference(reference);
            }
            _ => {}
        }

        // The ancestor stack is only needed for the `keep_names` reference checks.
        if self.keep_names_enabled() {
            self.ancestors.push(kind);
        }
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        if self.keep_names_enabled() {
            self.ancestors.pop();
        }
    }
}

fn is_assignment_target_id_of_specific_reference(
    target: &AssignmentTarget,
    reference_id: ReferenceId,
) -> bool {
    if let AssignmentTarget::AssignmentTargetIdentifier(id) = target {
        id.reference_id() == reference_id
    } else {
        false
    }
}

fn is_expression_whose_name_needs_to_be_kept(
    options: MangleOptionsKeepNames,
    expr: &Expression,
) -> bool {
    if !expr.is_anonymous_function_definition() {
        return false;
    }

    if options.class && options.function {
        return true;
    }

    let is_class = matches!(expr.without_parentheses(), Expression::ClassExpression(_));
    (options.class && is_class) || (options.function && !is_class)
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use rustc_hash::FxHashSet;

    use super::collect;
    use crate::MangleOptionsKeepNames;

    fn collect_keep_names(opts: MangleOptionsKeepNames, source_text: &str) -> FxHashSet<String> {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
        assert!(!parser_ret.panicked, "{source_text}");
        assert!(parser_ret.errors.is_empty(), "{source_text}");
        let program = parser_ret.program;
        let semantic_ret = SemanticBuilder::new().build(&program);
        assert!(semantic_ret.errors.is_empty(), "{source_text}");
        let semantic = semantic_ret.semantic;
        collect(opts, semantic.scoping(), &program)
            .keep_name_symbols
            .iter()
            .map(|&symbol_id| semantic.scoping().symbol_name(symbol_id).to_string())
            .collect()
    }

    fn data(s: &str) -> FxHashSet<String> {
        FxHashSet::from_iter([s.to_string()])
    }

    fn function_only() -> MangleOptionsKeepNames {
        MangleOptionsKeepNames { function: true, class: false }
    }

    fn class_only() -> MangleOptionsKeepNames {
        MangleOptionsKeepNames { function: false, class: true }
    }

    #[test]
    fn test_declarations() {
        assert_eq!(collect_keep_names(function_only(), "function foo() {}"), data("foo"));
        assert_eq!(collect_keep_names(class_only(), "class Foo {}"), data("Foo"));
    }

    #[test]
    fn test_simple_declare_init() {
        assert_eq!(collect_keep_names(function_only(), "var foo = function() {}"), data("foo"));
        assert_eq!(collect_keep_names(function_only(), "var foo = (function() {})"), data("foo"));
        assert_eq!(collect_keep_names(function_only(), "var foo = () => {}"), data("foo"));
        assert_eq!(collect_keep_names(function_only(), "var foo = (() => {})"), data("foo"));
        assert_eq!(collect_keep_names(class_only(), "var Foo = class {}"), data("Foo"));
        assert_eq!(collect_keep_names(class_only(), "var Foo = (class {})"), data("Foo"));
    }

    #[test]
    fn test_simple_assign() {
        assert_eq!(
            collect_keep_names(function_only(), "var foo; foo = function() {}"),
            data("foo")
        );
        assert_eq!(collect_keep_names(function_only(), "var foo; foo = () => {}"), data("foo"));
        assert_eq!(collect_keep_names(class_only(), "var Foo; Foo = class {}"), data("Foo"));

        assert_eq!(
            collect_keep_names(function_only(), "var foo; foo ||= function() {}"),
            data("foo")
        );
        assert_eq!(
            collect_keep_names(function_only(), "var foo = 1; foo &&= function() {}"),
            data("foo")
        );
        assert_eq!(
            collect_keep_names(function_only(), "var foo; foo ??= function() {}"),
            data("foo")
        );
    }

    #[test]
    fn test_default_declarations() {
        assert_eq!(
            collect_keep_names(function_only(), "var [foo = function() {}] = []"),
            data("foo")
        );
        assert_eq!(collect_keep_names(function_only(), "var [foo = () => {}] = []"), data("foo"));
        assert_eq!(collect_keep_names(class_only(), "var [Foo = class {}] = []"), data("Foo"));
        assert_eq!(
            collect_keep_names(function_only(), "var { foo = function() {} } = {}"),
            data("foo")
        );
    }

    #[test]
    fn test_default_assign() {
        assert_eq!(
            collect_keep_names(function_only(), "var foo; [foo = function() {}] = []"),
            data("foo")
        );
        assert_eq!(
            collect_keep_names(function_only(), "var foo; [foo = () => {}] = []"),
            data("foo")
        );
        assert_eq!(collect_keep_names(class_only(), "var Foo; [Foo = class {}] = []"), data("Foo"));
        assert_eq!(
            collect_keep_names(function_only(), "var foo; ({ foo = function() {} } = {})"),
            data("foo")
        );
    }

    #[test]
    fn test_for_in_declaration() {
        assert_eq!(
            collect_keep_names(function_only(), "for (var foo = function() {} in []) {}"),
            data("foo")
        );
        assert_eq!(
            collect_keep_names(function_only(), "for (var foo = () => {} in []) {}"),
            data("foo")
        );
        assert_eq!(
            collect_keep_names(class_only(), "for (var Foo = class {} in []) {}"),
            data("Foo")
        );
    }
}
