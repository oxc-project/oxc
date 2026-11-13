use oxc_allocator::{Allocator, BitSet};
use oxc_semantic::{Scoping, SymbolFlags};

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

/// Collects symbols whose names become the `name` property of a function or class: function and
/// class names, and bindings that name an anonymous function or class.
pub fn collect_name_symbols<'t>(
    options: MangleOptionsKeepNames,
    allocator: &'t Allocator,
    scoping: &Scoping,
) -> BitSet<'t> {
    let mut keep_flags = SymbolFlags::empty();
    if options.function {
        keep_flags |= SymbolFlags::Function | SymbolFlags::AnonymousFunctionName;
    }
    if options.class {
        keep_flags |= SymbolFlags::Class | SymbolFlags::AnonymousClassName;
    }
    let mut symbol_ids = BitSet::new_in(scoping.symbols_len(), allocator);
    for symbol_id in scoping.symbol_ids() {
        if scoping.symbol_flags(symbol_id).intersects(keep_flags) {
            symbol_ids.set_bit(symbol_id.index());
        }
    }
    symbol_ids
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
        let symbols = collect_name_symbols(opts, &allocator, semantic.scoping());
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

    fn none() -> FxHashSet<String> {
        FxHashSet::default()
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
        assert_eq!(collect(function_only(), "(function foo() {})"), data("foo"));
        assert_eq!(collect(class_only(), "(class Foo {})"), data("Foo"));
        assert_eq!(collect(class_only(), "function foo() {}"), none());
        assert_eq!(collect(function_only(), "class Foo {}"), none());
    }

    #[test]
    fn test_simple_declare_init() {
        assert_eq!(collect(function_only(), "var foo = function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = (function() {})"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = () => {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = (() => {})"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo = class {}"), data("Foo"));
        assert_eq!(collect(class_only(), "var Foo = (class {})"), data("Foo"));
        assert_eq!(collect(class_only(), "var foo = function() {}"), none());
        assert_eq!(collect(function_only(), "var Foo = class {}"), none());
        let all = MangleOptionsKeepNames::all_true();
        assert_eq!(collect(all, "var foo = function bar() {}"), data("bar"));
        assert_eq!(collect(all, "var foo = 1"), none());
    }

    #[test]
    fn test_simple_assign() {
        assert_eq!(collect(function_only(), "var foo; foo = function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; foo = () => {}"), data("foo"));
        assert_eq!(collect(class_only(), "var Foo; Foo = class {}"), data("Foo"));

        assert_eq!(collect(function_only(), "var foo; foo ||= function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo = 1; foo &&= function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; foo ??= function() {}"), data("foo"));
        assert_eq!(collect(function_only(), "var foo; foo += function() {}"), none());
        assert_eq!(collect(function_only(), "foo = function() {}"), none());
        assert_eq!(
            collect(function_only(), "var foo; (function() { foo = function() {} })"),
            data("foo")
        );
    }

    #[test]
    fn test_default_declarations() {
        assert_eq!(collect(function_only(), "var [foo = function() {}] = []"), data("foo"));
        assert_eq!(collect(function_only(), "var [foo = () => {}] = []"), data("foo"));
        assert_eq!(collect(class_only(), "var [Foo = class {}] = []"), data("Foo"));
        assert_eq!(collect(function_only(), "var { foo = function() {} } = {}"), data("foo"));
        assert_eq!(collect(function_only(), "var { bar: foo = function() {} } = {}"), data("foo"));
        assert_eq!(collect(function_only(), "var [[foo = () => {}] = []] = []"), data("foo"));
        assert_eq!(collect(function_only(), "var [...[foo = () => {}]] = []"), data("foo"));
        assert_eq!(collect(class_only(), "var [foo = function() {}] = []"), none());
    }

    #[test]
    fn test_default_parameters() {
        assert_eq!(collect(function_only(), "(function(foo = () => {}) {})"), data("foo"));
        assert_eq!(collect(class_only(), "((Foo = class {}) => {})"), data("Foo"));
        assert_eq!(collect(function_only(), "(function([foo = () => {}]) {})"), data("foo"));
        assert_eq!(collect(function_only(), "(function(...[foo = () => {}]) {})"), data("foo"));
        assert_eq!(collect(function_only(), "try {} catch ({ foo = () => {} }) {}"), data("foo"));
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
        assert_eq!(
            collect(function_only(), "var foo; ({ bar: foo = function() {} } = {})"),
            data("foo")
        );
        assert_eq!(collect(class_only(), "var foo; [foo = function() {}] = []"), none());
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
        assert_eq!(collect(class_only(), "var Foo = 1; Foo = class {}"), data("Foo"));
    }
}
