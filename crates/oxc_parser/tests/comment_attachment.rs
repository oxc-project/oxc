#![cfg(not(miri))]

use std::fmt::Write;

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

#[derive(Default)]
struct Cases(String);

impl Cases {
    fn group(&mut self, name: &str) {
        writeln!(self.0, "##### {name}\n").unwrap();
    }

    fn add(&mut self, source: &str, source_type: SourceType) {
        let language = match (source_type.is_typescript(), source_type.is_jsx()) {
            (true, true) => "tsx",
            (true, false) => "ts",
            (false, true) => "jsx",
            (false, false) => "js",
        };
        let unambiguous = if source_type.is_unambiguous() { " (unambiguous)" } else { "" };
        writeln!(self.0, "=== {language}{unambiguous} ===\n{source}\n---").unwrap();
        for line in attach(source, source_type) {
            writeln!(self.0, "{line}").unwrap();
        }
        self.0.push('\n');
    }
}

#[test]
fn comment_attachment() {
    let mut cases = Cases::default();
    cases.group("statements");
    cases.add("/* only */", SourceType::mjs());
    cases.add("// a\nfoo(); // b\n/* c */ bar();\n// d", SourceType::mjs());
    cases.add("'use strict' /* d */;\n// next\nfoo", SourceType::mjs());

    cases.group("expressions");
    cases.add("foo(a, /* b */ b /* after b */, c)", SourceType::mjs());
    cases.add("let x = /* before */ 1 + /* mid */ 2;", SourceType::mjs());
    cases.add("`a${/* in */ b}c`", SourceType::mjs());

    cases.group("empty_containers");
    cases.add("function f(/* params */) { /* body */ }", SourceType::mjs());
    cases.add("const x = [/* empty */]; const y = {/* empty */};", SourceType::mjs());
    cases.add("class A { static /* s */ { /* in */ } }", SourceType::mjs());

    cases.group("jsx");
    cases.add("<div>{/* child */}<a /* attr */ b={c} /></div>", SourceType::jsx());

    cases.group("typescript");
    cases.add("type T = /* t */ string | /* n */ number;", SourceType::ts());
    cases.add("class A { /* before */ @dec /* after dec */ method() {} }", SourceType::ts());
    cases.add("new Foo /* a */ <T> /* b */ ();", SourceType::ts());
    cases.add("class A extends B/* a */<T>/* b */ {}", SourceType::ts());

    cases.group("this_parameter");
    cases.add("function f(this: /* t */ P, /* b */ a: /* c */ T) {}", SourceType::ts());
    cases.add("function f(this: P,\n// a\na) {}", SourceType::ts());
    cases.add("function f(this: P, // t\n a, // a\n ...b) {}", SourceType::ts());
    cases.add("type F = (this: P, a: /* c */ T) => void;", SourceType::ts());
    cases.add(
        "interface I { m(this: P, a: /* m */ T): void; (this: P, a: /* c */ T): void }",
        SourceType::ts(),
    );

    cases.group("speculation_and_cover_grammar");
    cases.add("let f = (/* a */ a) => a; let g = (/* p */ b);", SourceType::mjs());
    cases.add("let f = <T,>(/* a */ a: T) => a; x = a < /* b */ b > c;", SourceType::ts());
    cases.add("({ a /* a */ = 1, b: /* b */ c } = d);", SourceType::mjs());
    cases.add("({a = /* @__PURE__ */ f() });", SourceType::mjs());
    cases.add("({a = f(x, /* c */ y) });", SourceType::mjs());
    cases.add("({ a = /* @__PURE__ */ f().x } = x);", SourceType::mjs());
    cases.add("[/* x */ x, ...rest /* rest */] = y;", SourceType::mjs());
    cases.add("import /* 1 */ source /* 2 */ x from 'x';", SourceType::mjs());
    cases.add("await /* a */ x; import 'y';", SourceType::unambiguous());

    cases.group("annotations");
    cases.add(
        "/* @__PURE__ */ foo(); /* #__NO_SIDE_EFFECTS__ */ function g() {}",
        SourceType::mjs(),
    );
    cases.add("x = /* @__PURE__ */ new Foo();", SourceType::mjs());

    cases.group("annotations_among_leading_comments");
    cases.add("// c\n/* @__PURE__ */ foo();", SourceType::mjs());
    cases.add(
        "/*#__NO_SIDE_EFFECTS__*/ /* c */ /*#__NO_SIDE_EFFECTS__*/ function foo() {}",
        SourceType::mjs(),
    );

    cases.group("comments_assigned_before_checkpoint");
    cases.add("f(a, // c\n  (b, // e\n  d));", SourceType::mjs());
    cases.add("f(a, // c\n  async (b, // e\n  d));", SourceType::mjs());

    cases.group("fatal_error");
    cases.add("foo(/* a */", SourceType::mjs());

    cases.group("list_boundaries");
    cases.add("function f() { 'use strict'; // c\n}", SourceType::mjs());
    cases.add("{ foo(); /* c */ }", SourceType::mjs());
    cases.add("function f() { return /* c */ (a); }", SourceType::mjs());
    cases.add("var x = /** @class */ (function () {})();", SourceType::mjs());
    cases.add("x = { a: /* v */ 1 // c\n};", SourceType::mjs());
    cases.add("x = [/* a */\u{a0}a];", SourceType::mjs());
    cases.add("x = { a: 1, // c\n};", SourceType::mjs());

    cases.group("comma_separated_lists");
    cases.add("f(\n  a, // c\n  b,\n);", SourceType::mjs());
    cases.add("x = [\n  a, // c\n  b,\n];", SourceType::mjs());
    cases.add("x = {\n  a: 1, // c\n  b: 2,\n};", SourceType::mjs());
    cases.add("function f(\n  a, // c\n  b,\n) {}", SourceType::mjs());
    cases.add("let a = 1, // c\n  b = 2;", SourceType::mjs());
    cases.add("[\n  a, // c\n  b,\n] = x;", SourceType::mjs());
    cases.add("f(\n  a, /* x */ // c\n  b,\n);", SourceType::mjs());
    cases.add("f(a, /* c */ b);", SourceType::mjs());
    cases.add("f(\n  a,\n  // c\n  b,\n);", SourceType::mjs());

    cases.group("comment_runs");
    cases.add("f(\n  a,\n  /* x */ // c\n  b,\n);", SourceType::mjs());
    cases.add("foo(); /* x */ // c\nbar();", SourceType::mjs());
    cases.add("foo();\n/* x */ // c\nbar();", SourceType::mjs());
    cases.add("foo(); /* x */ bar();", SourceType::mjs());
    cases.add("x = a /* x */ + // c\n  b;", SourceType::mjs());

    cases.group("binding_lists");
    cases.add("const {\n  a, // c\n  b,\n} = x;", SourceType::mjs());
    cases.add("const [\n  a, // c\n  b,\n] = x;", SourceType::mjs());
    cases.add("function f(\n  a, /* x */ // c\n  b,\n) {}", SourceType::mjs());
    cases.add("declare function f(a: T, ...b: any[], // trailing b\n) {}", SourceType::ts());
    cases.add("const { a = b /* c */ + d } = x;", SourceType::mjs());
    cases.add("const { a /* c */ = b } = x;", SourceType::mjs());

    cases.group("line_comment_trailing_whitespace");
    cases.add("function f() {\n  foo(); // a \n  // b\n}", SourceType::mjs());
    cases.add("x = (\n  a // a \n  // b\n);", SourceType::mjs());

    cases.group("operators");
    cases.add("x = a + /* b */ b;\ny = c ? /* d */ d : /* e */ e;", SourceType::mjs());
    cases.add("x = a /* op */ + b;", SourceType::mjs());

    cases.group("class_elements");
    cases.add(
        "class C {\n  // a\n  a() {} // after a\n  /* b */ b = 1;\n  // end\n}",
        SourceType::mjs(),
    );
    cases.add("class C {\n  // only\n}", SourceType::mjs());

    cases.group("empty_class_elements");
    cases.add("class C {\n  a() {}\n  // before semi\n  ;\n  b() {}\n}", SourceType::mjs());
    cases.add("class C {\n  a() {};\n  // c\n  b() {}\n}", SourceType::mjs());
    cases.add("class C {\n  a() {}\n  ; /* x */ ;\n  b() {}\n}", SourceType::mjs());

    cases.group("switch_cases");
    cases.add("switch (x) {\n  // a\n  case 1:\n    foo();\n    // falls through\n  case 2: // two\n    bar();\n  // end\n}", SourceType::mjs());
    cases.add("switch (x) {\n  // only\n}", SourceType::mjs());

    cases.group("decorators_before_export");
    cases.add("// before\n@dec export class C {}", SourceType::ts());
    cases.add("foo(); // after foo\n@dec export class C {}", SourceType::ts());
    cases.add("@dec(a /* c */ + b) export class C {}", SourceType::ts());
    cases.add("namespace N {\n  // before\n  @dec export class C {}\n}", SourceType::ts());

    cases.group("top_level_await_reparse");
    let line = "await /x/u; f(a, /* c */ b); // t\n";
    cases.add(&format!("{line}{line}{line}export {{}};"), SourceType::unambiguous());
    cases.add("'use strict'; // c\nawait /x/u;\nimport 'z';", SourceType::unambiguous());
    cases.add("await /x/u; y = /* @__PURE__ */ f().x; import 'z';", SourceType::unambiguous());
    cases.add("await /* @__PURE__ */ f().x; import 'z';", SourceType::unambiguous());
    insta::assert_snapshot!(cases.0);
}
