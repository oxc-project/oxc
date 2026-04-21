use oxc_allocator::Allocator;
use oxc_ast::ast::{ChainElement, Expression, Statement};
use oxc_ecmascript::side_effects::is_pure_function;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[track_caller]
fn test(source: &str, pure_functions: &[&str], expected: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source}");
    assert!(ret.errors.is_empty(), "{source}");

    let Some(Statement::ExpressionStatement(stmt)) = ret.program.body.first() else {
        panic!("should have an expression statement body: {source}");
    };
    let callee = get_callee(stmt.expression.without_parentheses(), source);

    let pure_fns: Vec<String> = pure_functions.iter().map(ToString::to_string).collect();
    assert_eq!(
        is_pure_function(callee, &pure_fns),
        expected,
        "{source} with pure_functions={pure_functions:?}"
    );
}

fn get_callee<'a>(expr: &'a Expression<'a>, source: &str) -> &'a Expression<'a> {
    match expr {
        Expression::CallExpression(call) => &call.callee,
        Expression::NewExpression(new_expr) => &new_expr.callee,
        Expression::TaggedTemplateExpression(tagged) => &tagged.tag,
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => &call.callee,
            _ => panic!("should have a call expression inside chain: {source}"),
        },
        _ => panic!("should have a call, new, or tagged template expression: {source}"),
    }
}

#[test]
fn test_simple_identifiers() {
    test("foo()", &["foo"], true);
    test("bar()", &["foo"], false);
    test("Foo()", &["foo"], false);
    test("foo()", &["Foo"], false);
    test("Foo()", &["Foo"], true);
    test("foo()", &["foo", "bar"], true);
    test("bar()", &["foo", "bar"], true);
    test("baz()", &["foo", "bar"], false);
}

#[test]
fn test_member_expressions() {
    test("console.log()", &["console"], true);
    test("console['log']()", &["console"], true);
    test("console.warn()", &["console"], true);
    test("other.log()", &["console"], false);
    test("other.console.log()", &["console"], false);
    test("other['console'].log()", &["console"], false);
    test("console.log()", &["console.log"], true);
    test("console['log']()", &["console.log"], true);
    test("console.warn()", &["console.log"], false);
    test("console['warn']()", &["console.log"], false);
    test("console.console.console()", &["console"], true);
    test("console.foo.console()", &["console"], true);
    test("console.other.log()", &["console.log"], false);

    test("a.b.c()", &["a"], true);
    test("a.b.c()", &["a.b"], true);
    test("a.b.c()", &["a.b.c"], true);
    test("a['b'].c()", &["a.b.c"], true);
    test("a.b['c']()", &["a.b.c"], true);
    test("a['b']['c']()", &["a.b.c"], true);
    test("a.b.d()", &["a.b.c"], false);
    test("a.b()", &["a.b.c"], false);
}

#[test]
fn test_chained_calls() {
    test("styled()()", &["styled"], true);
    test("styled()('div')", &["styled"], true);
    test("other()()", &["styled"], false);
    test("console.log()()", &["console.log"], true);
    test("console.log().foo()", &["console.log"], true);

    test("styled().div()", &["styled"], true);
    test("styled().button()", &["styled"], true);
    test("other().div()", &["styled"], false);
    test("console.log.foo()", &["console.log"], true);
    test("console.log.bar.baz()", &["console.log"], true);

    test("a()()()", &["a"], true);
    test("a()().b()", &["a"], true);
    test("a().b().c()", &["a"], true);
}

#[test]
fn test_optional_chaining() {
    test("styled?.()", &["styled"], true);
    test("other?.()", &["styled"], false);
    test("styled?.div()", &["styled"], true);
    test("console?.log()", &["console"], true);
    test("console?.log()", &["console.log"], true);
    test("console?.warn()", &["console.log"], false);
}

#[test]
fn test_new_expressions() {
    test("new Foo()", &["Foo"], true);
    test("new Bar()", &["Foo"], false);

    test("new styled.div()", &["styled"], true);
    test("new styled.div()", &["styled.div"], true);
    test("new styled.button()", &["styled.div"], false);
}

#[test]
fn test_tagged_template_expressions() {
    test("foo``", &["foo"], true);
    test("bar``", &["foo"], false);

    test("styled.div``", &["styled"], true);
    test("other.div``", &["styled"], false);
    test("styled.div``", &["styled.div"], true);
}

#[test]
fn test_edge_cases() {
    test("foo()", &[], false);
    test("console.log()", &[], false);
    test("styled()()", &[], false);

    test("(foo)()", &["foo"], true);
    test("(bar)()", &["foo"], false);
    test("((foo))()", &["foo"], true);
    test("(((console.log)))()", &["console.log"], true);

    test("foo()", &[""], false); // should not match anything
    test("console.log()", &["console."], false); // should not match anything
    test("console()", &["console."], false); // should not match anything
    test("console.log()", &["."], false); // should not match anything
}

/// Based on https://github.com/rollup/rollup/blob/v4.53.3/test/form/samples/manual-pure-functions
#[test]
fn test_rollup_manual_pure_functions() {
    test("foo()", &["foo", "bar.a"], true);
    test("foo.a()", &["foo", "bar.a"], true);
    test("foo.a()()", &["foo", "bar.a"], true);
    test("foo.a().a()", &["foo", "bar.a"], true);
    test("foo.a().a()()", &["foo", "bar.a"], true);
    test("foo.a().a().a()", &["foo", "bar.a"], true);

    test("bar()", &["foo", "bar.a"], false);
    test("bar.b()", &["foo", "bar.a"], false);

    test("bar.a()", &["foo", "bar.a"], true);
    test("bar?.a()", &["foo", "bar.a"], true);
    test("bar.a.a()", &["foo", "bar.a"], true);
    test("bar.a()()", &["foo", "bar.a"], true);
    test("bar.a().a()", &["foo", "bar.a"], true);
    test("bar.a()()()", &["foo", "bar.a"], true);
    test("bar.a()().a()", &["foo", "bar.a"], true);
}
