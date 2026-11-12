use oxc_allocator::{Allocator, BitSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{Scoping, SymbolId};
use oxc_syntax::scope::ScopeFlags;

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

pub fn collect_name_symbols<'t>(
    options: MangleOptionsKeepNames,
    allocator: &'t Allocator,
    scoping: &Scoping,
    program: &Program<'_>,
) -> BitSet<'t> {
    let mut collector = NameSymbolCollector {
        options,
        scoping,
        symbol_ids: BitSet::new_in(scoping.symbols_len(), allocator),
    };
    collector.visit_program(program);
    collector.symbol_ids
}

/// Collects symbols that are used to set `name` properties of functions and classes.
struct NameSymbolCollector<'s, 't> {
    options: MangleOptionsKeepNames,
    scoping: &'s Scoping,
    symbol_ids: BitSet<'t>,
}

impl<'a> Visit<'a> for NameSymbolCollector<'_, '_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        if self.options.function
            && let Some(id) = &func.id
        {
            self.keep_symbol(id.symbol_id());
        }
        walk::walk_function(self, func, flags);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if self.options.class
            && let Some(id) = &class.id
        {
            self.keep_symbol(id.symbol_id());
        }
        walk::walk_class(self, class);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &decl.id {
            if decl
                .init
                .as_ref()
                .is_some_and(|init| self.is_expression_whose_name_needs_to_be_kept(init))
            {
                self.keep_symbol(id.symbol_id());
            }
        } else {
            self.collect_assignment_patterns(&decl.id);
        }
        walk::walk_variable_declarator(self, decl);
    }

    // Check for direct assignment: foo = function() {}
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &expr.left
            && self.is_expression_whose_name_needs_to_be_kept(&expr.right)
        {
            self.keep_reference(id);
        }
        walk::walk_assignment_expression(self, expr);
    }

    // Check for assignments within assignment targets with defaults: [foo = function() {}] = []
    fn visit_assignment_target_with_default(&mut self, target: &AssignmentTargetWithDefault<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &target.binding
            && self.is_expression_whose_name_needs_to_be_kept(&target.init)
        {
            self.keep_reference(id);
        }
        walk::walk_assignment_target_with_default(self, target);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        prop: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        if prop
            .init
            .as_ref()
            .is_some_and(|init| self.is_expression_whose_name_needs_to_be_kept(init))
        {
            self.keep_reference(&prop.binding);
        }
        walk::walk_assignment_target_property_identifier(self, prop);
    }
}

impl NameSymbolCollector<'_, '_> {
    fn keep_symbol(&mut self, symbol_id: SymbolId) {
        self.symbol_ids.set_bit(symbol_id.index());
    }

    fn keep_reference(&mut self, ident: &IdentifierReference) {
        if let Some(symbol_id) = self.scoping.get_reference(ident.reference_id()).symbol_id() {
            self.keep_symbol(symbol_id);
        }
    }

    /// Collects `foo` in `var [foo = function() {}] = []` and `var { foo = function() {} } = {}`.
    fn collect_assignment_patterns(&mut self, pattern: &BindingPattern) {
        match pattern {
            BindingPattern::BindingIdentifier(_) => {}
            BindingPattern::ObjectPattern(object_pattern) => {
                for property in &object_pattern.properties {
                    self.collect_assignment_patterns(&property.value);
                }
            }
            BindingPattern::ArrayPattern(array_pattern) => {
                for element in array_pattern.elements.iter().flatten() {
                    self.collect_assignment_patterns(element);
                }
            }
            BindingPattern::AssignmentPattern(assign_pattern) => {
                if let BindingPattern::BindingIdentifier(id) = &assign_pattern.left {
                    if self.is_expression_whose_name_needs_to_be_kept(&assign_pattern.right) {
                        self.keep_symbol(id.symbol_id());
                    }
                } else {
                    self.collect_assignment_patterns(&assign_pattern.left);
                }
            }
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

        let is_class = matches!(expr.without_parentheses(), Expression::ClassExpression(_));
        (self.options.class && is_class) || (self.options.function && !is_class)
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::{SemanticBuilder, SymbolId};
    use oxc_span::SourceType;
    use rustc_hash::FxHashSet;

    use super::{MangleOptionsKeepNames, collect_name_symbols};

    fn collect(opts: MangleOptionsKeepNames, source_text: &str) -> FxHashSet<String> {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
        assert!(!parser_ret.fatal_error, "{source_text}");
        assert!(parser_ret.diagnostics.is_empty(), "{source_text}");
        let program = parser_ret.program;
        let ret = SemanticBuilder::new().build(&program);
        assert!(ret.diagnostics.is_empty(), "{source_text}");
        let semantic = ret.semantic;
        let symbols = collect_name_symbols(opts, &allocator, semantic.scoping(), &program);
        symbols
            .ones()
            .map(|symbol_id| {
                semantic.scoping().symbol_name(SymbolId::from_usize(symbol_id)).to_string()
            })
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

    #[test]
    fn test_redeclarations() {
        assert_eq!(collect(function_only(), "var foo = 1; var foo = function() {}"), data("foo"));
        assert_eq!(
            collect(function_only(), "(function() { var foo; function foo() {} })"),
            data("foo")
        );
        assert_eq!(collect(class_only(), "var Foo = 1; var Foo = class {}"), data("Foo"));
    }
}
