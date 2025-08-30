use javascript_globals::GLOBALS;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, Statement};
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
};
use oxc_minifier::PropertyReadSideEffects;
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

struct Ctx {
    global_variable_names: FxHashSet<&'static str>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            global_variable_names: GLOBALS["builtin"]
                .keys()
                .copied()
                .chain(["arguments", "URL"])
                .collect::<FxHashSet<_>>(),
        }
    }
}

impl<'a> GlobalContext<'a> for Ctx {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.global_variable_names.contains(ident.name.as_str())
    }
}

impl MayHaveSideEffectsContext<'_> for Ctx {
    fn annotations(&self) -> bool {
        true
    }

    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }

    fn unknown_global_side_effects(&self) -> bool {
        true
    }
}

#[track_caller]
fn test(source_text: &str, expected: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let stmt = ret.program.body.first().unwrap();
    assert_eq!(stmt.may_have_side_effects(&Ctx::default()), expected, "{source_text}");
}

#[track_caller]
fn test_in_function(source_text: &str, expected: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::FunctionDeclaration(stmt)) = &ret.program.body.first() else {
        panic!("should have a function declaration: {source_text}");
    };
    let stmt = stmt.body.as_ref().expect("should have a body").statements.first().unwrap();
    assert_eq!(stmt.may_have_side_effects(&Ctx::default()), expected, "{source_text}");
}

#[test]
fn test_block() {
    test("{}", false);
    test("{ ; ; }", false);
    test("{ foo() }", true);
    test("{ ; ; foo() }", true);
}

#[test]
fn test_do_while() {
    test("do { foo() } while (true)", true);
    test("do {} while (foo())", true);
    test("do {} while (true)", false);
}

#[test]
fn test_expr() {
    test("1", false);
    test("foo()", true);
}

#[test]
fn test_if() {
    test("if (foo()) {}", true);
    test("if (true) { foo() }", true);
    test("if (true) {}", false);
}

#[test]
fn test_labeled() {
    test("label: foo()", true);
    test("label: 1", false);
}

#[test]
fn test_return() {
    test_in_function("function _() { return foo() }", true);
    test_in_function("function _() { return 1 }", false);
}

#[test]
fn test_switch() {
    test("switch (foo()) {}", true);
    test("switch (true) { case foo(): }", true);
    test("switch (true) { case true: foo() }", true);
    test("switch (true) { case true: true }", false);
}

#[test]
fn test_try() {
    test("try { foo() } catch {}", true);
    test("try { true } catch ({}) {}", true);
    test("try { true } catch { foo() }", true);
    test("try { true } finally { foo() }", true);
    test("try { true } catch (e) { true } finally { true }", false);
}

#[test]
fn test_while() {
    test("while (true) { foo() }", true);
    test("while (foo()) {}", true);
    test("while (true) {}", false);
}

#[test]
fn test_declarations() {
    test("await using a = null", true);
    test("await using a = true", true);
    test("using a = null", false);
    test("using a = void 0", false);
    test("using a = null, b = 1", true);
    test("using a = void foo()", true);
    test("var a = foo()", true);
    test("var a = true", false);
    test("let a = foo()", true);
    test("let a = true", false);
    test("const a = foo()", true);
    test("const a = true", false);

    test("var [a] = []", true);
    test("var [a = foo()] = []", true);
    test("var [[a] = [foo()]] = []", true);
    test("var [a] = foo", true);
    test("var {a} = {}", true);
    test("var {a = foo()} = {}", true);
    test("var {a} = foo", true);
}

#[test]
fn test_others() {
    test("for (var a in b) {}", true);
    test("for (var a of b) {}", true);
    test("for (;;) {}", true);
    test("throw 1", true);
    test("with (a) {}", true);
    test("debugger", true);

    test("import 'a'", true);
    test("export * from 'a'", true);
    test("export { a }", true);
}
