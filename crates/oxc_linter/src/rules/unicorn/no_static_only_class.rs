use oxc_ast::{AstKind, ast::ClassElement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_static_only_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use an object instead of a class with only static members.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoStaticOnlyClass;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow classes that only have static members.
    ///
    /// ### Why is this bad?
    ///
    /// A class with only static members could just be an object instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A {
    ///     static a() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class A {
    ///     static a() {}
    ///
    ///     constructor() {}
    /// }
    /// ```
    /// ```javascript
    /// const X = {
    ///     foo: false,
    ///     bar() {}
    /// };
    /// ```
    /// ```javascript
    /// class X {
    ///     static #foo = false; // private field
    ///     static bar() {}
    /// }
    /// ```
    NoStaticOnlyClass,
    unicorn,
    pedantic,
    fix_dangerous
);

impl Rule for NoStaticOnlyClass {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        if class.super_class.is_some() {
            return;
        }
        if !class.decorators.is_empty() {
            return;
        }
        if class.body.body.is_empty() {
            return;
        }
        let mut ele_arr: Vec<&ClassElement> = vec![];
        let mut with_type_annotation = false;
        if class.body.body.iter().any(|node| {
            match node {
                ClassElement::MethodDefinition(v) => {
                    if v.accessibility.is_some() {
                        return true;
                    }
                }
                ClassElement::PropertyDefinition(v) => {
                    if v.accessibility.is_some() || v.readonly || v.declare {
                        return true;
                    }
                }
                ClassElement::AccessorProperty(_)
                | ClassElement::StaticBlock(_)
                | ClassElement::TSIndexSignature(_)
                 => {
                    return true;
                }
            }

            if node.r#static() {
                ele_arr.push(node);
                if !with_type_annotation && matches!(node, ClassElement::PropertyDefinition(v) if v.type_annotation.is_some()) {
                    with_type_annotation = true;
                }
                if let Some(k) = node.property_key() {
                    return k.is_private_identifier();
                }
                return false;
            }
            true
        }) {
            return;
        }
        #[expect(clippy::cast_possible_truncation)]
        ctx.diagnostic_with_dangerous_fix(no_static_only_class_diagnostic(class.span), |fixer| {
            if with_type_annotation
                || class.is_typescript_syntax()
                || (class.is_expression() && class.id.is_some())
                || !class.implements.is_empty()
            {
                return fixer.noop();
            }

            if (matches!(
                ctx.nodes().parent_kind(node.id()),
                AstKind::ExportDefaultDeclaration(_) | AstKind::ReturnStatement(_)
            ) && class.id.is_some())
            {
                return fixer.noop();
            }

            let fixer = fixer.for_multifix();
            let len = ele_arr.len();
            let mut rule_fixes = fixer.new_fix_with_capacity(len);
            for (idx, item) in ele_arr.iter().enumerate() {
                match item {
                    ClassElement::MethodDefinition(v) => {
                        let not_last = idx != len - 1;
                        let (key, value) = (&v.key, &v.value);
                        let name = if v.computed {
                            format!("[{}]", ctx.source_range(key.span()))
                        } else {
                            key.static_name().unwrap().to_string()
                        };

                        // we need to check is there have a trailing semicolon
                        // ```javascript
                        // class A {
                        //     static a() {}; // <-- trailing semicolon
                        // }
                        // ```
                        let next_start = if not_last {
                            ele_arr[idx + 1].span().start
                        } else {
                            class.body.span.end
                        };
                        let mut search_start = item.span().end;
                        let mut target_semicolon_pos = None;
                        while search_start < next_start {
                            if let Some(pos) = ctx
                                .source_range(Span::new(search_start, next_start))
                                .find(';')
                                .map(|p| search_start + (p as u32))
                            {
                                let comments = ctx.comments_range(item.span().end..next_start);
                                let mut is_in_comment = false;
                                for comment in comments {
                                    if comment.span.start < pos && comment.span.end > pos {
                                        is_in_comment = true;
                                        break;
                                    }
                                }
                                if !is_in_comment {
                                    target_semicolon_pos = Some(pos);
                                    break;
                                }
                                search_start = pos + 1;
                            } else {
                                break;
                            }
                        }
                        if let Some(pos) = target_semicolon_pos {
                            rule_fixes.push(fixer.delete_range(Span::sized(pos, 1)));
                        }
                        let replacement = format!("{name}{},", ctx.source_range(value.span()));
                        rule_fixes.push(fixer.replace(v.span, replacement));
                    }
                    ClassElement::PropertyDefinition(v) => {
                        if v.type_annotation.is_some() {
                            return fixer.noop();
                        }
                        let (key, value) = (&v.key, &v.value);
                        let name = if v.computed {
                            format!("[{}]", ctx.source_range(key.span()))
                        } else {
                            key.static_name().unwrap().to_string()
                        };
                        let value_str = if value.is_none() {
                            "undefined"
                        } else {
                            ctx.source_range(value.as_ref().unwrap().span())
                        };

                        let replacement = format!("{name}: {value_str},");
                        rule_fixes.push(fixer.replace(v.span, replacement));
                    }
                    _ => {}
                }
            }

            let start = class.span.start;
            if class.id.is_none() {
                // just remove the class keyword
                rule_fixes.push(fixer.delete_range(Span::sized(start, 5)));
            } else {
                let id = class.id.as_ref().unwrap();
                let target = Span::new(start, id.span.end);
                let replacement = format!("const {} =", id.name.as_str());
                rule_fixes.push(fixer.replace(target, replacement));
            }
            rule_fixes
                .with_message("Convert to an object instead of a class with only static members.")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"class A {}",
        r"const A = class {}",
        r"class A extends B { static a() {}; }",
        r"const A = class extends B { static a() {}; }",
        r"class A { a() {} }",
        r"class A { constructor() {} }",
        r"class A { get a() {} }",
        r"class A { set a(value) {} }",
        r"class A3 { static #a() {}; }",
        r"class A3 { static #a = 1; }",
        r"const A3 = class { static #a() {}; }",
        r"const A3 = class { static #a = 1; }",
        r"class A2 { static {}; }",
        r"class A { static #a() {}; }",
        r"class A { static #a = 1; }",
        r"const A = class { static #a() {}; }",
        r"const A = class { static #a = 1; }",
        r"@decorator class A { static  a = 1; }",
        r"class A { static public a = 1; }",
        r"class A { static private a = 1; }",
        r"class A { static readonly a = 1; }",
        r"class A { static declare a = 1; }",
        r"class A { static {}; }",
        r"class A2 { static #a() {}; }",
        r"class A2 { static #a = 1; }",
        r"const A2 = class { static #a() {}; }",
        r"const A2 = class { static #a = 1; }",
        r"class A2 { static {}; }",
        r"class X { static foo = 2; accessor y: string = 'hello'; }",
        r"class X { static foo = 2; static { } }",
    ];

    let fail = vec![
        r"class A { static a() {}; }",
        r"class A { static a() {} }",
        r"const A = class A { static a() {}; }",
        r"const A = class { static a() {}; }",
        r"class A { static constructor() {}; }",
        r"export default class A { static a() {}; }",
        r"export default class { static a() {}; }",
        r"export class A { static a() {}; }",
        r"class A {static [this.a] = 1}",
        r"class A { static a() {} }",
    ];

    let fix = vec![
        ("class A { static a() {}; }", "const A = { a() {}, }"),
        ("class A { static a() {} }", "const A = { a() {}, }"),
        ("const a = class { static bar = 2; static baz() {} }", "const a =  { bar: 2, baz() {}, }"),
        ("const A = class A { static a() {}; }", "const A = class A { static a() {}; }"),
        ("class A { static constructor() {}; }", "const A = { constructor() {}, }"),
        ("export default class A { static a() {}; }", "export default class A { static a() {}; }"),
        ("export default class{ static a() {}; }", "export default { a() {}, }"),
        ("export class A { static a() {}; }", "export const A = { a() {}, }"),
        ("class A {static foo = 1}", "const A = {foo: 1,}"),
        ("class A {static [this.a] = 1}", "const A = {[this.a]: 1,}"),
        ("class A { static a: string; }", "class A { static a: string; }"),
        (
            "class A { static a() {} /** comment ;;;; */ ;}",
            "const A = { a() {}, /** comment ;;;; */ }",
        ),
        (
            "class A { static a() {}; /** comment; */ static b() {}; }",
            "const A = { a() {}, /** comment; */ b() {}, }",
        ),
        (
            "class A { static a() {} /** comment; */ ;static b() {}; }",
            "const A = { a() {}, /** comment; */ b() {}, }",
        ),
        (
            "class A { static a() {}; /** comment; */ static b() {}; /** ;comment; */ static v = 2; }",
            "const A = { a() {}, /** comment; */ b() {}, /** ;comment; */ v: 2, }",
        ),
        (
            "class A { static a() {} /** comment; */ static b() {} /** ;comment; */ static v = 2 }",
            "const A = { a() {}, /** comment; */ b() {}, /** ;comment; */ v: 2, }",
        ),
        (
            "class A { static a() {}; /** comment ;;;; */ }",
            "const A = { a() {}, /** comment ;;;; */ }",
        ),
        ("function a() { return class{ static b = 2 }; }", "function a() { return { b: 2, }; }"),
        (
            "function a() { return class A { static b = 2 }; }",
            "function a() { return class A { static b = 2 }; }",
        ),
        ("(class { static A = 2 })", "( { A: 2, })"),
        ("(class A { static B = 2 })", "(class A { static B = 2 })"),
        (
            "class A { static foo = 2; static bar = 3; static baz() { return 2; } }",
            "const A = { foo: 2, bar: 3, baz() { return 2; }, }",
        ),
        ("class A { static foo; }", "const A = { foo: undefined, }"),
    ];

    Tester::new(NoStaticOnlyClass::NAME, NoStaticOnlyClass::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
