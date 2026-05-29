//! Collects the symbols whose `name` must be preserved when `keep_names` is enabled.
//!
//! This is the only part of the mangler that needs to inspect AST node parents, so it is done with
//! a single [`Visit`] pass (maintaining a parent-kind stack) instead of reading `oxc_semantic`'s
//! flattened node vec. It runs only when `keep_names` is on; the default mangling path does not
//! traverse the AST here (the scope/function-expression data it needs lives in [`Scoping`]).

use rustc_hash::FxHashSet;

use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::Visit;
use oxc_semantic::{ReferenceId, Scoping, SymbolId};

use crate::MangleOptionsKeepNames;

/// Walk `program` once and collect the symbols whose `name` must be preserved.
pub fn collect_keep_name_symbols(
    options: MangleOptionsKeepNames,
    scoping: &Scoping,
    program: &Program,
) -> FxHashSet<SymbolId> {
    let mut collector = KeepNameCollector {
        options,
        scoping,
        ancestors: Vec::new(),
        symbols: FxHashSet::default(),
    };
    collector.visit_program(program);
    collector.symbols
}

struct KeepNameCollector<'a, 's> {
    options: MangleOptionsKeepNames,
    scoping: &'s Scoping,
    /// Ancestors of the node being visited (does not include the node itself while in `enter_node`).
    ancestors: Vec<AstKind<'a>>,
    symbols: FxHashSet<SymbolId>,
}

impl<'a> KeepNameCollector<'a, '_> {
    /// Mark symbols whose `name` is set by a declaration (`var foo = function () {}`,
    /// `var [foo = function () {}] = []`, ...).
    fn collect_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        // `var foo = function () {}`
        if let BindingPattern::BindingIdentifier(id) = &declarator.id
            && let Some(init) = &declarator.init
            && is_expression_whose_name_needs_to_be_kept(self.options, init)
        {
            self.symbols.insert(id.symbol_id());
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
                        self.options,
                        &assignment_pattern.right,
                    )
                {
                    self.symbols.insert(id.symbol_id());
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
                    && is_expression_whose_name_needs_to_be_kept(self.options, &assign_expr.right)
            }
            // `[foo = function () {}] = []`
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                is_assignment_target_id_of_specific_reference(&assign_target.binding, reference_id)
                    && is_expression_whose_name_needs_to_be_kept(self.options, &assign_target.init)
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
                            self.options,
                            &assign_expr.right,
                        )
                    }
                    Some(AstKind::AssignmentTargetWithDefault(assign_target)) => {
                        is_assignment_target_id_of_specific_reference(
                            &assign_target.binding,
                            reference_id,
                        ) && is_expression_whose_name_needs_to_be_kept(
                            self.options,
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
                        is_expression_whose_name_needs_to_be_kept(self.options, init)
                    })
            }
            _ => false,
        };

        if needs_keep && let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id()
        {
            self.symbols.insert(symbol_id);
        }
    }
}

impl<'a> Visit<'a> for KeepNameCollector<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(func) => {
                if self.options.function
                    && let Some(id) = &func.id
                {
                    self.symbols.insert(id.symbol_id());
                }
            }
            AstKind::Class(class) => {
                if self.options.class
                    && let Some(id) = &class.id
                {
                    self.symbols.insert(id.symbol_id());
                }
            }
            AstKind::VariableDeclarator(declarator) => self.collect_declarator(declarator),
            AstKind::IdentifierReference(reference) => self.collect_reference(reference),
            _ => {}
        }
        self.ancestors.push(kind);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.ancestors.pop();
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

    use super::collect_keep_name_symbols;
    use crate::MangleOptionsKeepNames;

    fn collect(opts: MangleOptionsKeepNames, source_text: &str) -> FxHashSet<String> {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
        assert!(!parser_ret.panicked, "{source_text}");
        assert!(parser_ret.errors.is_empty(), "{source_text}");
        let program = parser_ret.program;
        let semantic_ret = SemanticBuilder::new().build(&program);
        assert!(semantic_ret.errors.is_empty(), "{source_text}");
        let semantic = semantic_ret.semantic;
        collect_keep_name_symbols(opts, semantic.scoping(), &program)
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
        assert_eq!(collect(function_only(), "function foo() {}"), data("foo"));
        assert_eq!(collect(class_only(), "class Foo {}"), data("Foo"));
    }

    #[test]
    fn test_simple_declare_init() {
        assert_eq!(collect(function_only(), "var foo = function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = (function() {})"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = () => {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = (() => {})"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo = class {}"), data("Foo"));
        assert_eq!(collect(class_only(), "var Foo = (class {})"), data("Foo"));
    }

    #[test]
    fn test_simple_assign() {
        assert_eq!(collect(function_only(), "var foo; foo = function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; foo = () => {}"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo; Foo = class {}"), data("Foo"));

        assert_eq!(collect(function_only(), "var foo; foo ||= function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = 1; foo &&= function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; foo ??= function() {}"), data("foo"));
    }

    #[test]
    fn test_default_declarations() {
        assert_eq!(collect(function_only(), "var [foo = function() {}] = []"), data("foo"));
        assert_eq!(collect(function_only(), "var [foo = () => {}] = []"), data("foo"));
        assert_eq!(collect(class_only(), "var [Foo = class {}] = []"), data("Foo"));
        assert_eq!(collect(function_only(), "var { foo = function() {} } = {}"), data("foo"));
    }

    #[test]
    fn test_default_assign() {
        assert_eq!(collect(function_only(), "var foo; [foo = function() {}] = []"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; [foo = () => {}] = []"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo; [Foo = class {}] = []"), data("Foo"));
        assert_eq!(
            collect(function_only(), "var foo; ({ foo = function() {} } = {})"),
            data("foo")
        );
    }

    #[test]
    fn test_for_in_declaration() {
        assert_eq!(collect(function_only(), "for (var foo = function() {} in []) {}"), data("foo"));
        assert_eq!(collect(function_only(), "for (var foo = () => {} in []) {}"), data("foo"));
        assert_eq!(collect(class_only(), "for (var Foo = class {} in []) {}"), data("Foo"));
    }
}
