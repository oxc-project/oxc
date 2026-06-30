use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::no_negated_condition::{
        DOCUMENTATION, run_on_conditional_expression, run_on_if_statement,
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoNegatedCondition;

declare_oxc_lint!(
    NoNegatedCondition,
    unicorn,
    pedantic,
    fix,
    docs = DOCUMENTATION,
    version = "0.0.18",
    short_description = "Disallow negated conditions.",
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                run_on_if_statement(if_stmt, ctx);
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                run_on_conditional_expression(node, conditional_expr, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Cases from eslint-plugin-unicorn `test/no-negated-condition.js`
    let pass = vec![
        r"if (a) {}",
        r"if (a) {} else {}",
        r"if (!a) {}",
        r"if (!a) {} else if (b) {}",
        r"if (!a) {} else if (b) {} else {}",
        r"if (a == b) {}",
        r"if (a == b) {} else {}",
        r"if (a != b) {}",
        r"if (a != b) {} else if (b) {}",
        r"if (a != b) {} else if (b) {} else {}",
        r"if (a !== b) {}",
        r"if (a === b) {} else {}",
        r"a ? b : c",
    ];

    let fail = vec![
        r"if (!a) {;} else {;}",
        r"if (a != b) {;} else {;}",
        r"if (a !== b) {;} else {;}",
        r"!a ? b : c",
        r"a != b ? c : d",
        r"a !== b ? c : d",
        r"(( !a )) ? b : c",
        r"!(( a )) ? b : c",
        r"if(!(( a ))) b(); else c();",
        r"if((( !a ))) b(); else c();",
        r"function a() {return!a ? b : c}",
        r"function a() {return!0 ? b:c}",
        r"function a() {return!\u0061 ? b:c}",
        r"function a() {return!(( a )) ? b : c}",
        "function a() {
return ! // comment
a ? b : c;
}",
        "function a() {
return (! // ReturnStatement argument is parenthesized
a ? b : c);
}",
        "function a() {
return (
! // UnaryExpression argument is parenthesized
a) ? b : c;
}",
        "function a() {
throw ! // comment
a ? b : c;
}",
        "function* a() {
yield ! // comment
a ? b : c;
}",
        "function a() {return !\r a ? b : c;}",
        "function a() {throw !\u{2028}a ? b : c;}",
        "function a() {return !\u{2029}a ? b : c;}",
        r"!{} ? a : b",
        r"!function(){} ? a : b",
        r"!class {} ? a : b",
        r"for (!a ? x in y : z;;) {}",
        r"!a ? b : c ? d : e",
        r"!a ? b : (( c ? d : e ))",
        "a
![] ? b : c",
        "a
!+b ? c : d",
        "a
!(b) ? c : d",
        "a
!b ? c : d",
        "if (!a)
b()
else
c()",
        r"if(!a) b(); else c()",
        "function fn() {
if(!a) b(); else return
}",
        r"if(!a) {b()} else {c()}",
        r"if(!!a) b(); else c();",
        r"(!!a) ? b() : c();",
        "function fn() {
return!a !== b ? c : d
return((!((a)) != b)) ? c : d
}",
        "if (!a) {
b();
} else if (!c) {
d();
} else {
e();
}",
    ];

    let fix = vec![
        (r"if (!a) {;} else {;}", r"if (a) {;} else {;}"),
        (r"if (a != b) {;} else {;}", r"if (a == b) {;} else {;}"),
        (r"if (a !== b) {;} else {;}", r"if (a === b) {;} else {;}"),
        (r"!a ? b : c", r"a ? c : b"),
        (r"a != b ? c : d", r"a == b ? d : c"),
        (r"a !== b ? c : d", r"a === b ? d : c"),
        (r"(( !a )) ? b : c", r"(( a )) ? c : b"),
        (r"!(( a )) ? b : c", r"(( a )) ? c : b"),
        (r"if(!(( a ))) b(); else c();", r"if((( a ))) {c();} else {b();}"),
        (r"if((( !a ))) b(); else c();", r"if((( a ))) {c();} else {b();}"),
        (r"function a() {return!a ? b : c}", r"function a() {return a ? c : b}"),
        (r"function a() {return!0 ? b:c}", r"function a() {return 0 ? c:b}"),
        (r"function a() {return!\u0061 ? b:c}", r"function a() {return \u0061 ? c:b}"),
        (r"function a() {return!(( a )) ? b : c}", r"function a() {return (( a )) ? c : b}"),
        (
            "function a() {
return ! // comment
a ? b : c;
}",
            "function a() {
return ( // comment
a ? c : b);
}",
        ),
        (
            "function a() {
return (! // ReturnStatement argument is parenthesized
a ? b : c);
}",
            "function a() {
return ( // ReturnStatement argument is parenthesized
a ? c : b);
}",
        ),
        (
            "function a() {
return (
! // UnaryExpression argument is parenthesized
a) ? b : c;
}",
            "function a() {
return (
 // UnaryExpression argument is parenthesized
a) ? c : b;
}",
        ),
        (
            "function a() {
throw ! // comment
a ? b : c;
}",
            "function a() {
throw ( // comment
a ? c : b);
}",
        ),
        (
            "function* a() {
yield ! // comment
a ? b : c;
}",
            "function* a() {
yield ( // comment
a ? c : b);
}",
        ),
        ("function a() {return !\r a ? b : c;}", "function a() {return (\r a ? c : b);}"),
        ("function a() {throw !\u{2028}a ? b : c;}", "function a() {throw (\u{2028}a ? c : b);}"),
        ("function a() {return !\u{2029}a ? b : c;}", "function a() {return (\u{2029}a ? c : b);}"),
        (r"!{} ? a : b", r"({}) ? b : a"),
        (r"!function(){} ? a : b", r"(function(){}) ? b : a"),
        (r"!class {} ? a : b", r"(class {}) ? b : a"),
        (r"for (!a ? x in y : z;;) {}", r"for (a ? z : (x in y);;) {}"),
        (r"!a ? b : c ? d : e", r"a ? c ? d : e : b"),
        (r"!a ? b : (( c ? d : e ))", r"a ? (( c ? d : e )) : b"),
        (
            "a
![] ? b : c",
            "a
;[] ? c : b",
        ),
        (
            "a
!+b ? c : d",
            "a
;+b ? d : c",
        ),
        (
            "a
!(b) ? c : d",
            "a
;(b) ? d : c",
        ),
        (
            "a
!b ? c : d",
            "a
b ? d : c",
        ),
        (
            "if (!a)
b()
else
c()",
            "if (a)
{c()}
else
{b()}",
        ),
        (r"if(!a) b(); else c()", r"if(a) {c()} else {b();}"),
        (
            "function fn() {
if(!a) b(); else return
}",
            "function fn() {
if(a) {return} else {b();}
}",
        ),
        (r"if(!a) {b()} else {c()}", r"if(a) {c()} else {b()}"),
        (r"if(!!a) b(); else c();", r"if(!a) {c();} else {b();}"),
        (r"(!!a) ? b() : c();", r"(!a) ? c() : b();"),
        (
            "function fn() {
return!a !== b ? c : d
return((!((a)) != b)) ? c : d
}",
            "function fn() {
return!a === b ? d : c
return((!((a)) == b)) ? d : c
}",
        ),
        (
            "if (!a) {
b();
} else if (!c) {
d();
} else {
e();
}",
            "if (!a) {
b();
} else if (c) {
e();
} else {
d();
}",
        ),
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
