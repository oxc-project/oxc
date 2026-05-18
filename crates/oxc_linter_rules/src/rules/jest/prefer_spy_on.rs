use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_spy_on::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferSpyOn;

declare_oxc_lint!(PreferSpyOn, jest, style, suggestion, docs = DOCUMENTATION, version = "0.2.14",);

impl Rule for PreferSpyOn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(_) = node.kind() else {
            return;
        };

        run(node, ctx);
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("Date.now = () => 10", None),
        ("window.fetch = jest.fn", None),
        ("Date.now = fn()", None),
        ("obj.mock = jest.something()", None),
        ("const mock = jest.fn()", None),
        ("mock = jest.fn()", None),
        ("const mockObj = { mock: jest.fn() }", None),
        ("mockObj = { mock: jest.fn() }", None),
        ("window[`${name}`] = jest[`fn${expression}`]()", None),
    ];

    let fail = vec![
        ("obj.a = jest.fn(); const test = 10;", None),
        ("Date['now'] = jest['fn']()", None),
        ("window[`${name}`] = jest[`fn`]()", None),
        ("obj['prop' + 1] = jest['fn']()", None),
        ("obj.one.two = jest.fn(); const test = 10;", None),
        ("obj.a = jest.fn(() => 10,)", None),
        (
            "obj.a.b = jest.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        ("window.fetch = jest.fn(() => ({})).one.two().three().four", None),
        ("foo[bar] = jest.fn().mockReturnValue(undefined)", None),
        (
            "
                foo.bar = jest.fn().mockImplementation(baz => baz)
                foo.bar = jest.fn(a => b).mockImplementation(baz => baz)
            ",
            None,
        ),
    ];

    let fix = vec![
        (
            "obj.a = jest.fn(); const test = 10;",
            "jest.spyOn(obj, 'a').mockImplementation(); const test = 10;",
            None,
        ),
        ("Date['now'] = jest['fn']()", "jest.spyOn(Date, 'now').mockImplementation()", None),
        (
            "window[`${name}`] = jest[`fn`]()",
            "jest.spyOn(window, `${name}`).mockImplementation()",
            None,
        ),
        (
            "obj['prop' + 1] = jest['fn']()",
            "jest.spyOn(obj, 'prop' + 1).mockImplementation()",
            None,
        ),
        (
            "obj.one.two = jest.fn(); const test = 10;",
            "jest.spyOn(obj.one, 'two').mockImplementation(); const test = 10;",
            None,
        ),
        ("obj.a = jest.fn(() => 10,)", "jest.spyOn(obj, 'a').mockImplementation(() => 10)", None),
        (
            "obj.a.b = jest.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            "jest.spyOn(obj.a, 'b').mockImplementation(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        (
            "window.fetch = jest.fn(() => ({})).one.two().three().four",
            "jest.spyOn(window, 'fetch').mockImplementation(() => ({})).one.two().three().four",
            None,
        ),
        (
            "foo[bar] = jest.fn().mockReturnValue(undefined)",
            "jest.spyOn(foo, bar).mockImplementation().mockReturnValue(undefined)",
            None,
        ),
        (
            "
                foo.bar = jest.fn().mockImplementation(baz => baz)
                foo.bar = jest.fn(a => b).mockImplementation(baz => baz)
            ",
            "
                jest.spyOn(foo, 'bar').mockImplementation(baz => baz)
                jest.spyOn(foo, 'bar').mockImplementation(baz => baz)
            ",
            None,
        ),
    ];

    Tester::new(PreferSpyOn::NAME, PreferSpyOn::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
