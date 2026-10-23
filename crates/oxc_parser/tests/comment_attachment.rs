use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::node::NodeId;

#[derive(Default)]
struct Nodes(FxHashMap<NodeId, (AstType, Span)>);

impl<'a> Visit<'a> for Nodes {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.0.insert(kind.node_id(), (kind.ty(), kind.span()));
    }
}

fn attach(source: &str, source_type: SourceType) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let mut nodes = Nodes::default();
    nodes.visit_program(&parsed.program);
    parsed
        .program
        .comments
        .iter()
        .map(|comment| {
            let (ty, span) = nodes.0[&comment.node_id.get()];
            format!(
                "{} -> {:?} {ty:?} `{}`",
                comment.span.source_text(source),
                comment.placement,
                span.source_text(source)
            )
        })
        .collect()
}

#[track_caller]
fn check(source: &str, source_type: SourceType, expected: &[&str]) {
    assert_eq!(attach(source, source_type), expected, "{source}");
}

#[test]
fn statements() {
    check("/* only */", SourceType::mjs(), &["/* only */ -> Dangling Program `/* only */`"]);
    check(
        "// a\nfoo(); // b\n/* c */ bar();\n// d",
        SourceType::mjs(),
        &[
            "// a -> Leading ExpressionStatement `foo();`",
            "// b -> Trailing ExpressionStatement `foo();`",
            "/* c */ -> Leading ExpressionStatement `bar();`",
            "// d -> Trailing ExpressionStatement `bar();`",
        ],
    );
    check(
        "'use strict' /* d */;\n// next\nfoo",
        SourceType::mjs(),
        &[
            "/* d */ -> Trailing StringLiteral `'use strict'`",
            "// next -> Leading ExpressionStatement `foo`",
        ],
    );
}

#[test]
fn expressions() {
    check(
        "foo(a, /* b */ b /* after b */, c)",
        SourceType::mjs(),
        &[
            "/* b */ -> Leading IdentifierReference `b`",
            "/* after b */ -> Trailing IdentifierReference `b`",
        ],
    );
    check(
        "let x = /* before */ 1 + /* mid */ 2;",
        SourceType::mjs(),
        &[
            "/* before */ -> Leading BinaryExpression `1 + /* mid */ 2`",
            "/* mid */ -> Leading NumericLiteral `2`",
        ],
    );
    check("`a${/* in */ b}c`", SourceType::mjs(), &["/* in */ -> Leading IdentifierReference `b`"]);
}

#[test]
fn empty_containers() {
    check(
        "function f(/* params */) { /* body */ }",
        SourceType::mjs(),
        &[
            "/* params */ -> Dangling FormalParameters `(/* params */)`",
            "/* body */ -> Dangling FunctionBody `{ /* body */ }`",
        ],
    );
    check(
        "const x = [/* empty */]; const y = {/* empty */};",
        SourceType::mjs(),
        &[
            "/* empty */ -> Dangling ArrayExpression `[/* empty */]`",
            "/* empty */ -> Dangling ObjectExpression `{/* empty */}`",
        ],
    );
    check(
        "class A { static /* s */ { /* in */ } }",
        SourceType::mjs(),
        &[
            "/* s */ -> Dangling StaticBlock `static /* s */ { /* in */ }`",
            "/* in */ -> Dangling StaticBlock `static /* s */ { /* in */ }`",
        ],
    );
}

#[test]
fn jsx() {
    check(
        "<div>{/* child */}<a /* attr */ b={c} /></div>",
        SourceType::jsx(),
        &[
            "/* child */ -> Dangling JSXEmptyExpression `/* child */`",
            "/* attr */ -> Leading JSXAttribute `b={c}`",
        ],
    );
}

#[test]
fn typescript() {
    check(
        "type T = /* t */ string | /* n */ number;",
        SourceType::ts(),
        &[
            "/* t */ -> Leading TSUnionType `string | /* n */ number`",
            "/* n */ -> Leading TSNumberKeyword `number`",
        ],
    );
    check(
        "class A { /* before */ @dec /* after dec */ method() {} }",
        SourceType::ts(),
        &[
            "/* before */ -> Leading MethodDefinition `@dec /* after dec */ method() {}`",
            "/* after dec */ -> Leading IdentifierName `method`",
        ],
    );
    check(
        "new Foo /* a */ <T> /* b */ ();",
        SourceType::ts(),
        &[
            "/* a */ -> Leading TSTypeParameterInstantiation `<T>`",
            "/* b */ -> Trailing TSTypeParameterInstantiation `<T>`",
        ],
    );
    check(
        "class A extends B/* a */<T>/* b */ {}",
        SourceType::ts(),
        &[
            "/* a */ -> Leading TSTypeParameterInstantiation `<T>`",
            "/* b */ -> Leading ClassBody `{}`",
        ],
    );
}

#[test]
fn speculation_and_cover_grammar() {
    check(
        "let f = (/* a */ a) => a; let g = (/* p */ b);",
        SourceType::mjs(),
        &["/* a */ -> Leading FormalParameter `a`", "/* p */ -> Leading IdentifierReference `b`"],
    );
    check(
        "let f = <T,>(/* a */ a: T) => a; x = a < /* b */ b > c;",
        SourceType::ts(),
        &[
            "/* a */ -> Leading FormalParameter `a: T`",
            "/* b */ -> Leading IdentifierReference `b`",
        ],
    );
    check(
        "({ a /* a */ = 1, b: /* b */ c } = d);",
        SourceType::mjs(),
        &[
            "/* a */ -> Trailing IdentifierReference `a`",
            "/* b */ -> Leading IdentifierReference `c`",
        ],
    );
    check(
        "[/* x */ x, ...rest /* rest */] = y;",
        SourceType::mjs(),
        &[
            "/* x */ -> Leading IdentifierReference `x`",
            "/* rest */ -> Trailing AssignmentTargetRest `...rest`",
        ],
    );
    check(
        "import /* 1 */ source /* 2 */ x from 'x';",
        SourceType::mjs(),
        &[
            "/* 1 */ -> Dangling ImportDeclaration `import /* 1 */ source /* 2 */ x from 'x';`",
            "/* 2 */ -> Leading ImportDefaultSpecifier `x`",
        ],
    );
    check(
        "await /* a */ x; import 'y';",
        SourceType::unambiguous(),
        &["/* a */ -> Leading IdentifierReference `x`"],
    );
}

#[test]
fn annotations() {
    check(
        "/* @__PURE__ */ foo(); /* #__NO_SIDE_EFFECTS__ */ function g() {}",
        SourceType::mjs(),
        &[
            "/* @__PURE__ */ -> Leading CallExpression `foo()`",
            "/* #__NO_SIDE_EFFECTS__ */ -> Leading Function `function g() {}`",
        ],
    );
    check(
        "x = /* @__PURE__ */ new Foo();",
        SourceType::mjs(),
        &["/* @__PURE__ */ -> Leading NewExpression `new Foo()`"],
    );
}

#[test]
fn fatal_error() {
    check("foo(/* a */", SourceType::mjs(), &["/* a */ -> Dangling Program ``"]);
}

#[test]
fn list_boundaries() {
    check(
        "function f() { 'use strict'; // c\n}",
        SourceType::mjs(),
        &["// c -> Trailing Directive `'use strict';`"],
    );
    check(
        "{ foo(); /* c */ }",
        SourceType::mjs(),
        &["/* c */ -> Trailing ExpressionStatement `foo();`"],
    );
    check(
        "function f() { return /* c */ (a); }",
        SourceType::mjs(),
        &["/* c */ -> Leading ParenthesizedExpression `(a)`"],
    );
    check(
        "var x = /** @class */ (function () {})();",
        SourceType::mjs(),
        &["/** @class */ -> Leading CallExpression `(function () {})()`"],
    );
    check(
        "x = { a: /* v */ 1 // c\n};",
        SourceType::mjs(),
        &[
            "/* v */ -> Leading NumericLiteral `1`",
            "// c -> Trailing ObjectProperty `a: /* v */ 1`",
        ],
    );
    check(
        "x = [/* a */\u{a0}a];",
        SourceType::mjs(),
        &["/* a */ -> Leading IdentifierReference `a`"],
    );
    check("x = { a: 1, // c\n};", SourceType::mjs(), &["// c -> Trailing ObjectProperty `a: 1`"]);
}

#[test]
fn comma_separated_lists() {
    check(
        "f(\n  a, // c\n  b,\n);",
        SourceType::mjs(),
        &["// c -> Trailing IdentifierReference `a`"],
    );
    check(
        "x = [\n  a, // c\n  b,\n];",
        SourceType::mjs(),
        &["// c -> Trailing IdentifierReference `a`"],
    );
    check(
        "x = {\n  a: 1, // c\n  b: 2,\n};",
        SourceType::mjs(),
        &["// c -> Trailing ObjectProperty `a: 1`"],
    );
    check(
        "function f(\n  a, // c\n  b,\n) {}",
        SourceType::mjs(),
        &["// c -> Trailing FormalParameter `a`"],
    );
    check(
        "let a = 1, // c\n  b = 2;",
        SourceType::mjs(),
        &["// c -> Trailing VariableDeclarator `a = 1`"],
    );
    check(
        "[\n  a, // c\n  b,\n] = x;",
        SourceType::mjs(),
        &["// c -> Trailing IdentifierReference `a`"],
    );
    check(
        "f(\n  a, /* x */ // c\n  b,\n);",
        SourceType::mjs(),
        &["/* x */ -> Leading IdentifierReference `b`", "// c -> Trailing IdentifierReference `a`"],
    );
    check("f(a, /* c */ b);", SourceType::mjs(), &["/* c */ -> Leading IdentifierReference `b`"]);
    check(
        "f(\n  a,\n  // c\n  b,\n);",
        SourceType::mjs(),
        &["// c -> Leading IdentifierReference `b`"],
    );
}
