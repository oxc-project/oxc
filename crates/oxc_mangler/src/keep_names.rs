use rustc_hash::FxHashSet;

use oxc_ast::{AstKind, ast::*};
use oxc_semantic::{AstNode, AstNodes, ReferenceId, Scoping, SymbolId};

#[derive(Debug, Clone, Copy, Default)]
pub struct MangleOptionsKeepNames {
    /// Preserve `name` property for functions.
    ///
    /// Default `false`
    pub function: bool,

    /// Preserve `name` property for classes.
    ///
    /// Default `false`
    pub class: bool,
}

impl MangleOptionsKeepNames {
    pub fn all_false() -> Self {
        Self { function: false, class: false }
    }

    pub fn all_true() -> Self {
        Self { function: true, class: true }
    }
}

impl From<bool> for MangleOptionsKeepNames {
    fn from(keep_names: bool) -> Self {
        if keep_names { Self::all_true() } else { Self::all_false() }
    }
}

pub fn collect_name_symbols(
    options: MangleOptionsKeepNames,
    scoping: &Scoping,
    ast_nodes: &AstNodes,
) -> FxHashSet<SymbolId> {
    let collector = NameSymbolCollector::new(options, scoping, ast_nodes);
    collector.collect()
}

/// Collects symbols that are used to set `name` properties of functions and classes.
struct NameSymbolCollector<'a, 'b> {
    options: MangleOptionsKeepNames,
    scoping: &'b Scoping,
    ast_nodes: &'b AstNodes<'a>,
}

impl<'a, 'b: 'a> NameSymbolCollector<'a, 'b> {
    fn new(
        options: MangleOptionsKeepNames,
        scoping: &'b Scoping,
        ast_nodes: &'b AstNodes<'a>,
    ) -> Self {
        Self { options, scoping, ast_nodes }
    }

    fn collect(self) -> FxHashSet<SymbolId> {
        if !self.options.function && !self.options.class {
            return FxHashSet::default();
        }

        self.scoping
            .symbol_ids()
            .filter(|symbol_id| {
                let decl_node =
                    self.ast_nodes.get_node(self.scoping.symbol_declaration(*symbol_id));
                self.is_name_set_declare_node(decl_node, *symbol_id)
                    || self.has_name_set_reference_node(*symbol_id)
            })
            .collect()
    }

    fn has_name_set_reference_node(&self, symbol_id: SymbolId) -> bool {
        self.scoping
            .get_resolved_reference_ids(symbol_id)
            .iter()
            .any(|&reference_id| self.is_name_set_reference_node(reference_id))
    }

    fn is_name_set_declare_node(&self, node: &'a AstNode, symbol_id: SymbolId) -> bool {
        match node.kind() {
            AstKind::Function(function) => {
                self.options.function
                    && function.id.as_ref().is_some_and(|id| id.symbol_id() == symbol_id)
            }
            AstKind::Class(cls) => {
                self.options.class && cls.id.as_ref().is_some_and(|id| id.symbol_id() == symbol_id)
            }
            AstKind::VariableDeclarator(decl) => {
                if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                    if id.symbol_id() == symbol_id {
                        return decl.init.as_ref().is_some_and(|init| {
                            self.is_expression_whose_name_needs_to_be_kept(init)
                        });
                    }
                }
                if let Some(assign_pattern) =
                    Self::find_assign_binding_pattern_kind_of_specific_symbol(
                        &decl.id.kind,
                        symbol_id,
                    )
                {
                    return self.is_expression_whose_name_needs_to_be_kept(&assign_pattern.right);
                }
                false
            }
            _ => false,
        }
    }

    fn is_name_set_reference_node(&self, reference_id: ReferenceId) -> bool {
        let node_id = self.scoping.get_reference(reference_id).node_id();
        let parent_node_id = self.ast_nodes.parent_id(node_id);
        match self.ast_nodes.kind(parent_node_id) {
            // Check for direct assignment: foo = function() {}
            AstKind::AssignmentExpression(assign_expr) => {
                Self::is_assignment_target_id_of_specific_reference(&assign_expr.left, reference_id)
                    && self.is_expression_whose_name_needs_to_be_kept(&assign_expr.right)
            }
            // Check for assignments within assignment targets with defaults: [foo = function() {}] = []
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                Self::is_assignment_target_id_of_specific_reference(
                    &assign_target.binding,
                    reference_id,
                ) && self.is_expression_whose_name_needs_to_be_kept(&assign_target.init)
            }
            AstKind::IdentifierReference(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::StaticMemberExpression(_) => {
                let grand_parent_node_kind = self.ast_nodes.parent_kind(parent_node_id);

                match grand_parent_node_kind {
                    AstKind::AssignmentExpression(assign_expr) => {
                        Self::is_assignment_target_id_of_specific_reference(
                            &assign_expr.left,
                            reference_id,
                        ) && self.is_expression_whose_name_needs_to_be_kept(&assign_expr.right)
                    }
                    AstKind::AssignmentTargetWithDefault(assign_target) => {
                        Self::is_assignment_target_id_of_specific_reference(
                            &assign_target.binding,
                            reference_id,
                        ) && self.is_expression_whose_name_needs_to_be_kept(&assign_target.init)
                    }
                    _ => false,
                }
            }
            AstKind::AssignmentTargetPropertyIdentifier(ident) => {
                if ident.binding.reference_id() == reference_id {
                    return ident
                        .init
                        .as_ref()
                        .is_some_and(|init| self.is_expression_whose_name_needs_to_be_kept(init));
                }
                false
            }
            _ => false,
        }
    }

    fn find_assign_binding_pattern_kind_of_specific_symbol(
        kind: &'a BindingPatternKind,
        symbol_id: SymbolId,
    ) -> Option<&'a AssignmentPattern<'a>> {
        match kind {
            BindingPatternKind::BindingIdentifier(_) => None,
            BindingPatternKind::ObjectPattern(object_pattern) => {
                for property in &object_pattern.properties {
                    if let Some(value) = Self::find_assign_binding_pattern_kind_of_specific_symbol(
                        &property.value.kind,
                        symbol_id,
                    ) {
                        return Some(value);
                    }
                }
                None
            }
            BindingPatternKind::ArrayPattern(array_pattern) => {
                for element in &array_pattern.elements {
                    let Some(binding) = element else { continue };

                    if let Some(value) = Self::find_assign_binding_pattern_kind_of_specific_symbol(
                        &binding.kind,
                        symbol_id,
                    ) {
                        return Some(value);
                    }
                }
                None
            }
            BindingPatternKind::AssignmentPattern(assign_pattern) => {
                if Self::is_binding_id_of_specific_symbol(&assign_pattern.left.kind, symbol_id) {
                    return Some(assign_pattern);
                }
                Self::find_assign_binding_pattern_kind_of_specific_symbol(
                    &assign_pattern.left.kind,
                    symbol_id,
                )
            }
        }
    }

    fn is_binding_id_of_specific_symbol(
        pattern_kind: &BindingPatternKind,
        symbol_id: SymbolId,
    ) -> bool {
        if let BindingPatternKind::BindingIdentifier(id) = pattern_kind {
            id.symbol_id() == symbol_id
        } else {
            false
        }
    }

    fn is_assignment_target_id_of_specific_reference(
        target_kind: &AssignmentTarget,
        reference_id: ReferenceId,
    ) -> bool {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = target_kind {
            id.reference_id() == reference_id
        } else {
            false
        }
    }

    fn is_expression_whose_name_needs_to_be_kept(&self, expr: &Expression) -> bool {
        let is_anonymous = expr.is_anonymous_function_definition();
        if !is_anonymous {
            return false;
        }

        if self.options.class && self.options.function {
            return true;
        }

        let is_class = matches!(expr, Expression::ClassExpression(_));
        (self.options.class && is_class) || (self.options.function && !is_class)
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use rustc_hash::FxHashSet;

    use super::{MangleOptionsKeepNames, collect_name_symbols};

    fn collect(opts: MangleOptionsKeepNames, source_text: &str) -> FxHashSet<String> {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
        assert!(!ret.panicked, "{source_text}");
        assert!(ret.errors.is_empty(), "{source_text}");
        let ret = SemanticBuilder::new().build(&ret.program);
        assert!(ret.errors.is_empty(), "{source_text}");
        let semantic = ret.semantic;
        let symbols = collect_name_symbols(opts, semantic.scoping(), semantic.nodes());
        symbols
            .into_iter()
            .map(|symbol_id| semantic.scoping().symbol_name(symbol_id).to_string())
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
        assert_eq!(collect(function_only(), "var foo = () => {}"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo = class {}"), data("Foo"));
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
