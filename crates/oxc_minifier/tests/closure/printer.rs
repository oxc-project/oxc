//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/CodePrinterTest.java>

use crate::{expect, expect_reparse, expect_same};

macro_rules! lines {
    ($base:expr, $($segment:expr),+) => {&{
        let mut s = String::new();
        $(
            s.push_str($segment);
        )*
        s
    }}
}

#[test]
#[ignore]
fn test_big_int() {
    expect_same("1n");
    expect("0b10n", "2n");
    expect("0o3n", "3n");
    expect("0x4n", "4n");
    expect_same("-5n");
    expect("-0b110n", "-6n");
    expect("-0o7n", "-7n");
    expect("-0x8n", "-8n");
}

#[test]
#[ignore]
fn test_trailing_comma_in_array_and_object_with_pretty_print() {
    expect_same("({a:1, b:2,});\n");
    expect_same("[1, 2, 3,];\n");
    // An array starting with a hole is printed ideally but this is very rare.
    expect_same("[, ];\n");
}

#[test]
#[ignore]
fn test_trailing_comma_in_array_and_object_without_pretty_print() {
    expect("({a:1, b:2,})", "({a:1,b:2})");
    expect("[1, 2, 3,]", "[1,2,3]");

    // When the last element is empty,  the trailing comma must be kept.
    expect_same("[,]"); // same as `[undefined]`
    expect_same("[a,,]"); // same as `[a, undefined]`
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_array_literal() {
    // In cases where we modify the AST we might wind up with an array literal that has no elements
    // yet still has a trailing comma. This is meant to test for that. We need to build the tree
    // manually because an array literal with no elements and a trailing comma has a different
    // meaning: it represents a single undefined element.
    // Node arrLit = IR.arraylit();
    // arrLit.setTrailingComma(true);
    // expectNode("[]", arrLit);
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_object_literal() {
    // In cases where we modify the AST we might wind up with an object literal that has no elements
    // yet still has a trailing comma. This is meant to test for that. We need to build the tree
    // manually because an object literal with no elements and a trailing comma is a syntax error.
    // Node objLit = IR.objectlit();
    // objLit.setTrailingComma(true);
    // expectNode("{}", objLit);
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_param_list() {
    // In cases where we modify the AST we might wind up with a parameter list that has no elements
    // yet still has a trailing comma. This is meant to test for that. We need to build the tree
    // manually because a parameter list with no elements and a trailing comma is a syntax error.
    // Node paramList = IR.paramList();
    // IR.function(IR.name("f"), paramList, IR.block());
    // paramList.setTrailingComma(true);
    // expectNode("()", paramList);
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_call() {
    // In cases where we modify the AST we might wind up with a call node that has no elements
    // yet still has a trailing comma. This is meant to test for that. We need to build the tree
    // manually because a call node with no elements and a trailing comma is a syntax error.
    // Node call = IR.call(IR.name("f"));
    // call.setTrailingComma(true);
    // expectNode("f()", call);
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_opt_chain_call() {
    // In cases where we modify the AST we might wind up with an optional chain call node that has
    // no elements yet still has a trailing comma. This is meant to test for that. We need to build
    // the tree manually because an optional chain call node with no elements and a trailing comma
    // is a syntax error.
    // Node optChainCall = IR.startOptChainCall(IR.name("f"));
    // optChainCall.setTrailingComma(true);
    // expectNode("f?.()", optChainCall);
}

#[test]
#[ignore]
fn test_no_trailing_comma_in_empty_new() {
    // In cases where we modify the AST we might wind up with a new node that has no elements
    // yet still has a trailing comma. This is meant to test for that. We need to build the tree
    // manually because a new node with no elements and a trailing comma is a syntax error.
    // Node newNode = IR.newNode(IR.name("f"));
    // newNode.setTrailingComma(true);
    // expectNode("new f()", newNode);
}

#[test]
#[ignore]
fn test_trailing_comma_in_parameter_list_with_pretty_print() {
    expect_same("function f(a, b,) {\n}\n");
    expect_same("f(1, 2,);\n");
    expect_same("f?.(1, 2,);\n");
    expect_same("let x = new Number(1,);\n");
}

#[test]
#[ignore]
fn test_trailing_comma_in_parameter_list_without_pretty_print() {
    expect("function f(a, b,) {}", "function f(a,b){}");
    expect("f(1, 2,);", "f(1,2)");
    expect("f?.(1, 2,);", "f?.(1,2)");
    expect("let x = new Number(1,);", "let x=new Number(1)");
}

#[test]
#[ignore]
fn opt_chain() {
    expect_same("a.b?.c");
    expect_same("a.b?.[\"c\"]");
    expect_same("a.b?.()");
    expect_same("a?.b.c?.d");
    expect_same("(a?.b).c");
    expect_same("(a.b?.c.d).e");
    expect_same("(a?.[b])[c]");
    expect_same("(a.b?.())()");
}

#[test]
#[ignore]
fn test_unescaped_unicode_line_separator_2018() {
    expect_same("`\\u2028`");

    expect("'\\u2028'", "\"\\u2028\"");
    expect("\"\\u2028\"", "\"\\u2028\"");

    // printed as a unicode escape for ES_2018 output
    expect("'\\u2028'", "\"\\u2028\"");
    expect("\"\\u2028\"", "\"\\u2028\"");
}

#[test]
#[ignore]
fn test_unescaped_unicode_line_separator_2019() {
    expect("'\\u2028'", "\"\\u2028\"");
    expect("\"\\u2028\"", "\"\\u2028\"");

    // left unescaped for ES_2019 out
    expect("'\\u2028'", "\"\\u2028\"");
    expect("\"\\u2028\"", "\"\\u2028\"");
}

#[test]
#[ignore]
fn test_unescaped_unicode_paragraph_separator_2018() {
    expect_same("`\\u2029`");

    expect("'\\u2029'", "\"\\u2029\"");
    expect("\"\\u2029\"", "\"\\u2029\"");

    // printed as a unicode escape for ES_2018 output
    expect("'\\u2029'", "\"\\u2029\"");
    expect("\"\\u2029\"", "\"\\u2029\"");
}

#[test]
#[ignore]
fn test_unescaped_unicode_paragraph_separator_2019() {
    expect("'\\u2029'", "\"\\u2029\"");
    expect("\"\\u2029\"", "\"\\u2029\"");

    // left unescaped for ES_2019 out
    expect("'\\u2029'", "\"\\u2029\"");
    expect("\"\\u2029\"", "\"\\u2029\"");
}

#[test]
#[ignore]
fn test_optional_catch_block() {
    expect_same("try{}catch{}");
    expect_same("try{}catch{}finally{}");
}

#[test]
#[ignore]
fn test_exponentiation_operator() {
    expect_same("x**y");
    // Exponentiation is right associative
    expect("x**(y**z)", "x**y**z");
    expect_same("(x**y)**z");
    // parens are kept because ExponentiationExpression cannot expand to
    //     UnaryExpression ** ExponentiationExpression
    expect_same("(-x)**y");
    // parens are kept because unary operators are higher precedence than '**'
    expect_same("-(x**y)");
    // parens are not needed for a unary operator on the right operand
    expect("x**(-y)", "x**-y");
    // NOTE: "-x**y" is a syntax error tested in ParserTest

    // ** has a higher precedence than /
    expect("x/(y**z)", "x/y**z");
    expect_same("(x/y)**z");
}

#[test]
#[ignore]
fn test_exponentiation_assignment_operator() {
    expect_same("x**=y");
}

#[test]
#[ignore]
fn test_nullish_coalesce_operator() {
    expect_same("x??y??z");
    // Nullish coalesce is left associative
    expect_same("x??(y??z)");
    expect("(x??y)??z", "x??y??z");
    // // parens are kept because logical AND and logical OR must be separated from '??'
    expect_same("(x&&y)??z");
    expect_same("(x??y)||z");
    expect_same("x??(y||z)");
    // NOTE: "x&&y??z" is a syntax error tested in ParserTest
}

#[test]
#[ignore]
fn test_nullish_coalesce_operator2() {
    // | has higher precedence than ??
    expect("(a|b)??c", "a|b??c");
    expect_same("(a??b)|c");
    expect_same("a|(b??c)");
    expect("a??(b|c)", "a??b|c");
    // ?? has higher precedence than : ? (conditional)
    expect("(a??b)?(c??d):(e??f)", "a??b?c??d:e??f");
    expect_same("a??(b?c:d)");
    expect_same("(a?b:c)??d");
}

#[test]
#[ignore]
fn test_logical_assignment_operator() {
    expect_same("x||=y");
    expect_same("x&&=y");
    expect_same("x??=y");
}

#[test]
#[ignore]
fn test_object_literal_with_spread() {
    expect_same("({...{}})");
    expect_same("({...x})");
    expect_same("({...x,a:1})");
    expect_same("({a:1,...x})");
    expect_same("({a:1,...x,b:1})");
    expect_same("({...x,...y})");
    expect_same("({...x,...f()})");
    expect_same("({...{...{}}})");
}

#[test]
#[ignore]
fn test_object_literal_with_comma() {
    expect_same("({[(a,b)]:c})");
    expect_same("({a:(b,c)})");
    expect_same("({[(a,b)]:(c,d)})");
    expect_same("({[(a,b)]:c,[d]:(e,f)})");
}

#[test]
#[ignore]
fn test_print() {
    expect("10 + a + b", "10+a+b");
    expect("10 + (30*50)", "10+30*50");
    expect("with(x) { x + 3; }", "with(x)x+3");
    expect("\"aa'a\"", "\"aa'a\"");
    expect("\"aa\\\"a\"", "'aa\"a'");
    expect("function foo()\n{return 10;}", "function foo(){return 10}");
    expect("a instanceof b", "a instanceof b");
    expect("typeof(a)", "typeof a");
    expect(
        "var foo = x ? { a : 1 } : {a: 3, b:4, \"default\": 5, \"foo-bar\": 6}",
        "var foo=x?{a:1}:{a:3,b:4,\"default\":5,\"foo-bar\":6}",
    );

    // Safari: needs ';' at the end of a throw statement
    expect("function foo(){throw 'error';}", "function foo(){throw\"error\";}");

    // The code printer does not eliminate unnecessary blocks.
    expect("var x = 10; { var y = 20; }", "var x=10;{var y=20}");

    expect("while (x-- > 0);", "while(x-- >0);");
    expect("x-- >> 1", "x-- >>1");

    expect("(function () {})(); ", "(function(){})()");

    // Associativity
    expect("var a,b,c,d;a || (b&& c) && (a || d)", "var a,b,c,d;a||b&&c&&(a||d)");
    expect(
        "var a,b,c; a || (b || c); a * (b * c); a | (b | c)",
        "var a,b,c;a||(b||c);a*(b*c);a|(b|c)",
    );
    expect("var a,b,c; a / b / c;a / (b / c); a - (b - c);", "var a,b,c;a/b/c;a/(b/c);a-(b-c)");

    // Nested assignments
    expect("var a,b; a = b = 3;", "var a,b;a=b=3");
    expect("var a,b,c,d; a = (b = c = (d = 3));", "var a,b,c,d;a=b=c=d=3");
    expect("var a,b,c; a += (b = c += 3);", "var a,b,c;a+=b=c+=3");
    expect("var a,b,c; a *= (b -= c);", "var a,b,c;a*=b-=c");

    // Precedence
    expect("a ? delete b[0] : 3", "a?delete b[0]:3");
    expect("(delete a[0])/10", "delete a[0]/10");

    // optional '()' for new

    // simple new
    expect("new A", "new A");
    expect("new A()", "new A");
    expect("new A('x')", "new A(\"x\")");

    // calling instance method directly after new
    expect("new A().a()", "(new A).a()");
    expect("(new A).a()", "(new A).a()");

    // this case should be fixed
    expect("new A('y').a()", "(new A(\"y\")).a()");

    // internal class
    expect("new A.B", "new A.B");
    expect("new A.B()", "new A.B");
    expect("new A.B('z')", "new A.B(\"z\")");

    // calling instance method directly after new internal class
    expect("(new A.B).a()", "(new A.B).a()");
    expect("new A.B().a()", "(new A.B).a()");
    // this case should be fixed
    expect("new A.B('w').a()", "(new A.B(\"w\")).a()");

    // calling new on the result of a call
    expect_same("new (a())");
    expect("new (a())()", "new (a())");
    expect_same("new (a.b())");
    expect("new (a.b())()", "new (a.b())");

    // Operators: make sure we don't convert binary + and unary + into ++
    expect("x + +y", "x+ +y");
    expect("x - (-y)", "x- -y");
    expect("x++ +y", "x++ +y");
    expect("x-- -y", "x-- -y");
    expect("x++ -y", "x++-y");

    // Label
    expect("foo:for(;;){break foo;}", "foo:for(;;)break foo");
    expect("foo:while(1){continue foo;}", "foo:while(1)continue foo");
    expect_same("foo:;");
    expect("foo: {}", "foo:;");

    // Object literals.
    expect("({})", "({})");
    expect("var x = {};", "var x={}");
    expect("({}).x", "({}).x");
    expect("({})['x']", "({})[\"x\"]");
    expect("({}) instanceof Object", "({})instanceof Object");
    expect("({}) || 1", "({})||1");
    expect("1 || ({})", "1||{}");
    expect("({}) ? 1 : 2", "({})?1:2");
    expect("0 ? ({}) : 2", "0?{}:2");
    expect("0 ? 1 : ({})", "0?1:{}");
    expect("typeof ({})", "typeof{}");
    expect("f({})", "f({})");

    // Anonymous function expressions.
    expect("(function(){})", "(function(){})");
    expect("(function(){})()", "(function(){})()");
    expect("(function(){})instanceof Object", "(function(){})instanceof Object");
    expect("(function(){}).bind().call()", "(function(){}).bind().call()");
    expect("var x = function() { };", "var x=function(){}");
    expect("var x = function() { }();", "var x=function(){}()");
    expect("(function() {}), 2", "(function(){}),2");

    // Name functions expression.
    expect("(function f(){})", "(function f(){})");

    // Function declaration.
    expect("function f(){}", "function f(){}");

    // Make sure we don't treat non-Latin character escapes as raw strings.
    expect("({ 'a': 4, '\\u0100': 4 })", "({\"a\":4,\"\\u0100\":4})");
    expect("({ a: 4, '\\u0100': 4 })", "({a:4,\"\\u0100\":4})");

    // Test if statement and for statements with single statements in body.
    expect("if (true) { alert();}", "if(true)alert()");
    expect("if (false) {} else {alert(\"a\");}", "if(false);else alert(\"a\")");
    expect("for(;;) { alert();};", "for(;;)alert()");

    expect("do { alert(); } while(true);", "do alert();while(true)");
    expect("myLabel: { alert();}", "myLabel:alert()");
    expect("myLabel: for(;;) continue myLabel;", "myLabel:for(;;)continue myLabel");

    // Test nested var statement
    expect("if (true) var x; x = 4;", "if(true)var x;x=4");

    // Non-latin identifier. Make sure we keep them escaped.
    expect("\\u00fb", "\\u00fb");
    expect("\\u00fa=1", "\\u00fa=1");
    expect("function \\u00f9(){}", "function \\u00f9(){}");
    expect("x.\\u00f8", "x.\\u00f8");
    expect("x.\\u00f8", "x.\\u00f8");
    expect("abc\\u4e00\\u4e01jkl", "abc\\u4e00\\u4e01jkl");

    // Test the right-associative unary operators for spurious parens
    expect("! ! true", "!!true");
    expect("!(!(true))", "!!true");
    expect("typeof(void(0))", "typeof void 0");
    expect("typeof(void(!0))", "typeof void!0");
    expect("+ - + + - + 3", "+-+ +-+3"); // chained unary plus/minus
    expect("+(--x)", "+--x");
    expect("-(++x)", "-++x");

    // needs a space to prevent an ambiguous parse
    expect("-(--x)", "- --x");
    expect("!(~~5)", "!~~5");
    expect("~(a/b)", "~(a/b)");

    // Preserve parens to overcome greedy binding of NEW
    expect("new (foo.bar()).factory(baz)", "new (foo.bar().factory)(baz)");
    expect("new (bar()).factory(baz)", "new (bar().factory)(baz)");
    expect("new (new foobar(x)).factory(baz)", "new (new foobar(x)).factory(baz)");

    // Make sure that HOOK is right associative
    expect("a ? b : (c ? d : e)", "a?b:c?d:e");
    expect("a ? (b ? c : d) : e", "a?b?c:d:e");
    expect("(a ? b : c) ? d : e", "(a?b:c)?d:e");

    // Test nested ifs
    expect("if (x) if (y); else;", "if(x)if(y);else;");

    // Test comma.
    expect("a,b,c", "a,b,c");
    expect("(a,b),c", "a,b,c");
    expect("a,(b,c)", "a,b,c");
    expect("x=a,b,c", "x=a,b,c");
    expect("x=(a,b),c", "x=(a,b),c");
    expect("x=a,(b,c)", "x=a,b,c");
    expect("x=a,y=b,z=c", "x=a,y=b,z=c");
    expect("x=(a,y=b,z=c)", "x=(a,y=b,z=c)");
    expect("x=[a,b,c,d]", "x=[a,b,c,d]");
    expect("x=[(a,b,c),d]", "x=[(a,b,c),d]");
    expect("x=[(a,(b,c)),d]", "x=[(a,b,c),d]");
    expect("x=[a,(b,c,d)]", "x=[a,(b,c,d)]");
    expect("var x=(a,b)", "var x=(a,b)");
    expect("var x=a,b,c", "var x=a,b,c");
    expect("var x=(a,b),c", "var x=(a,b),c");
    expect("var x=a,b=(c,d)", "var x=a,b=(c,d)");
    expect("var x=(a,b)(c);", "var x=(a,b)(c)");
    expect("var x=(a,b)`c`;", "var x=(a,b)`c`");
    expect("foo(a,b,c,d)", "foo(a,b,c,d)");
    expect("foo((a,b,c),d)", "foo((a,b,c),d)");
    expect("foo((a,(b,c)),d)", "foo((a,b,c),d)");
    expect("f(a+b,(c,d,(e,f,g)))", "f(a+b,(c,d,e,f,g))");
    expect("({}) , 1 , 2", "({}),1,2");
    expect("({}) , {} , {}", "({}),{},{}");

    expect_same("var a=(b=c,d)");
    expect_same("var a=(b[c]=d,e)");
    expect_same("var a=(b[c]=d,e[f]=g,h)");

    expect("var a = /** @type {?} */ (b=c,d)", "var a=(b=c,d)");
    expect("var a = /** @type {?} */ (b[c]=d,e)", "var a=(b[c]=d,e)");
    expect("var a = /** @type {?} */ (b[c]=d,e[f]=g,h)", "var a=(b[c]=d,e[f]=g,h)");

    // EMPTY nodes
    expect("if (x){}", "if(x);");
    expect("if(x);", "if(x);");
    expect("if(x)if(y);", "if(x)if(y);");
    expect("if(x){if(y);}", "if(x)if(y);");
    expect("if(x){if(y){};;;}", "if(x)if(y);");
}

#[test]
#[ignore]
fn test_print_new_void() {
    // Odd looking but valid. This, of course, will cause a runtime exception but
    // should not cause a parse error as "new void 0" would.
    expect_same("new (void 0)");
}

#[test]
#[ignore]
fn test_print_comma1() {
    // Node node = IR.var(IR.name("a"), IR.comma(IR.comma(IR.name("b"), IR.name("c")), IR.name("d")));
    // expectNode("var a=(b,c,d)", node);
}

#[test]
#[ignore]
fn test_print_comma2() {
    // Node node = IR.var(IR.name("a"), IR.comma(IR.name("b"), IR.comma(IR.name("c"), IR.name("d"))));
    // expectNode("var a=(b,c,d)", node);
}

#[test]
#[ignore]
fn test_pretty_print_js_doc() {
    expect_same("/** @type {number} */ \nvar x;\n");
}

#[test]
#[ignore]
fn test_print_cast1() {
    expect("var x = /** @type {number} */ (0);", "var x=0");
    expect_same("var x = /** @type {number} */ (0);\n");
}

#[test]
#[ignore]
fn test_print_cast2() {
    expect("var x = (2+3) * 4;", "var x=(2+3)*4");
    expect("var x = /** @type {number} */ (2+3) * 4;", "var x=(2+3)*4");
    expect_same("var x = (/** @type {number} */ (2 + 3)) * 4;\n");
}

#[test]
#[ignore]
fn test_print_cast3() {
    expect("var x = (2*3) + 4;", "var x=2*3+4");
    expect("var x = /** @type {number} */ (2*3) + 4;", "var x=2*3+4");
    expect_same("var x = /** @type {number} */ (2 * 3) + 4;\n");
}

#[test]
#[ignore]
fn test_let_const_in_if() {
    expect("if (true) { let x; };", "if(true){let x}");
    expect("if (true) { const x = 0; };", "if(true){const x=0}");
}

#[test]
#[ignore]
fn test_print_block_scoped_functions() {
    // Safari 3 needs a "{" around a single function
    expect("if (true) function foo(){return}", "if(true){function foo(){return}}");
    expect("if(x){;;function y(){};;}", "if(x){function y(){}}");
}

#[test]
#[ignore]
fn test_print_array_pattern_var() {
    expect_same("var []=[]");
    expect_same("var [a]=[1]");
    expect_same("var [a,b]=[1,2]");
    expect_same("var [a,...b]=[1,2]");
    expect_same("var [,b]=[1,2]");
    expect_same("var [,,,,,,g]=[1,2,3,4,5,6,7]");
    expect_same("var [a,,c]=[1,2,3]");
    expect_same("var [a,,,d]=[1,2,3,4]");
    expect_same("var [a,,c,,e]=[1,2,3,4,5]");
}

#[test]
#[ignore]
fn test_print_array_pattern_let() {
    expect_same("let []=[]");
    expect_same("let [a]=[1]");
    expect_same("let [a,b]=[1,2]");
    expect_same("let [a,...b]=[1,2]");
    expect_same("let [,b]=[1,2]");
    expect_same("let [,,,,,,g]=[1,2,3,4,5,6,7]");
    expect_same("let [a,,c]=[1,2,3]");
    expect_same("let [a,,,d]=[1,2,3,4]");
    expect_same("let [a,,c,,e]=[1,2,3,4,5]");
}

#[test]
#[ignore]
fn test_print_array_pattern_const() {
    expect_same("const []=[]");
    expect_same("const [a]=[1]");
    expect_same("const [a,b]=[1,2]");
    expect_same("const [a,...b]=[1,2]");
    expect_same("const [,b]=[1,2]");
    expect_same("const [,,,,,,g]=[1,2,3,4,5,6,7]");
    expect_same("const [a,,c]=[1,2,3]");
    expect_same("const [a,,,d]=[1,2,3,4]");
    expect_same("const [a,,c,,e]=[1,2,3,4,5]");
}

#[test]
#[ignore]
fn test_print_array_pattern_assign() {
    expect_same("[]=[]");
    expect_same("[a]=[1]");
    expect_same("[a,b]=[1,2]");
    expect_same("[a,...b]=[1,2]");
    expect_same("[,b]=[1,2]");
    expect_same("[,,,,,,g]=[1,2,3,4,5,6,7]");
    expect_same("[a,,c]=[1,2,3]");
    expect_same("[a,,,d]=[1,2,3,4]");
    expect_same("[a,,c,,e]=[1,2,3,4,5]");
}

#[test]
#[ignore]
fn test_print_array_pattern_with_initializer() {
    expect_same("[x=1]=[]");
    expect_same("[a,,c=2,,e]=[1,2,3,4,5]");
    expect_same("[a=1,b=2,c=3]=foo()");
    expect_same("[a=(1,2),b]=foo()");
    expect_same("[a=[b=(1,2)]=bar(),c]=foo()");
}

#[test]
#[ignore]
fn test_print_nested_array_pattern() {
    expect_same("var [a,[b,c],d]=[1,[2,3],4]");
    expect_same("var [[[[a]]]]=[[[[1]]]]");

    expect_same("[a,[b,c],d]=[1,[2,3],4]");
    expect_same("[[[[a]]]]=[[[[1]]]]");
}

#[test]
#[ignore]
fn test_pretty_print_array_pattern() {
    expect("let [a,b,c]=foo();", "let [a, b, c] = foo();\n");
}

#[test]
#[ignore]
fn test_print_object_pattern_var() {
    expect_same("var {a}=foo()");
    expect_same("var {a,b}=foo()");
    expect_same("var {a:a,b:b}=foo()");
}

#[test]
#[ignore]
fn test_print_object_pattern_let() {
    expect_same("let {a}=foo()");
    expect_same("let {a,b}=foo()");
    expect_same("let {a:a,b:b}=foo()");
}

#[test]
#[ignore]
fn test_print_object_pattern_const() {
    expect_same("const {a}=foo()");
    expect_same("const {a,b}=foo()");
    expect_same("const {a:a,b:b}=foo()");
}

#[test]
#[ignore]
fn test_print_object_pattern_assign() {
    expect_same("({a}=foo())");
    expect_same("({a,b}=foo())");
    expect_same("({a:a,b:b}=foo())");
}

#[test]
#[ignore]
fn test_print_nested_object_pattern() {
    expect_same("({a:{b,c}}=foo())");
    expect_same("({a:{b:{c:{d}}}}=foo())");
}

#[test]
#[ignore]
fn test_print_object_pattern_initializer() {
    expect_same("({a=1}=foo())");
    expect_same("({a:{b=2}}=foo())");
    expect_same("({a:b=2}=foo())");
    expect_same("({a,b:{c=2}}=foo())");
    expect_same("({a:{b=2},c}=foo())");
    expect_same("({a=(1,2),b}=foo())");
    expect_same("({a:b=(1,2),c}=foo())");
}

#[test]
#[ignore]
fn test_print_object_pattern_with_rest() {
    expect_same("const {a,...rest}=foo()");
    expect_same("var {a,...rest}=foo()");
    expect_same("let {a,...rest}=foo()");
    expect_same("({a,...rest}=foo())");
    expect_same("({a=2,...rest}=foo())");
    expect_same("({a:b=2,...rest}=foo())");
}

#[test]
#[ignore]
fn test_pretty_print_object_pattern() {
    expect("const {a,b,c}=foo();", "const {a, b, c} = foo();\n");
}

#[test]
#[ignore]
fn test_print_mixed_destructuring() {
    expect_same("({a:[b,c]}=foo())");
    expect_same("[a,{b,c}]=foo()");
}

#[test]
#[ignore]
fn test_print_destructuring_in_param_list() {
    expect_same("function f([a]){}");
    expect_same("function f([a,b]){}");
    expect_same("function f([a,b]=c()){}");
    expect_same("function f([a=(1,2),b=(3,4)]=c()){}");
    expect_same("function f({a}){}");
    expect_same("function f({a,b}){}");
    expect_same("function f({a,b}=c()){}");
    expect_same("function f([a,{b,c}]){}");
    expect_same("function f({a,b:[c,d]}){}");
}

#[test]
#[ignore]
fn test_print_destructuring_in_rest_param() {
    expect_same("function f(...[a,b]){}");
    expect_same("function f(...{length:num_params}){}");
}

#[test]
#[ignore]
fn test_destructuring_for_in_loops() {
    expect_same("for({a}in b)c");
    expect_same("for(var {a}in b)c");
    expect_same("for(let {a}in b)c");
    expect_same("for(const {a}in b)c");

    expect_same("for({a:b}in c)d");
    expect_same("for(var {a:b}in c)d");
    expect_same("for(let {a:b}in c)d");
    expect_same("for(const {a:b}in c)d");

    expect_same("for([a]in b)c");
    expect_same("for(var [a]in b)c");
    expect_same("for(let [a]in b)c");
    expect_same("for(const [a]in b)c");
}

#[test]
#[ignore]
fn test_destructuring_for_of_loops1() {
    expect_same("for({a}of b)c");
    expect_same("for(var {a}of b)c");
    expect_same("for(let {a}of b)c");
    expect_same("for(const {a}of b)c");

    expect_same("for({a:b}of c)d");
    expect_same("for(var {a:b}of c)d");
    expect_same("for(let {a:b}of c)d");
    expect_same("for(const {a:b}of c)d");

    expect_same("for([a]of b)c");
    expect_same("for(var [a]of b)c");
    expect_same("for(let [a]of b)c");
    expect_same("for(const [a]of b)c");
}

#[test]
#[ignore]
fn test_destructuring_for_of_loops2() {
    // The destructuring 'var' statement is a child of the for-of loop, but
    // not the first child.
    expect_same("for(a of b)var {x}=y");
}

#[test]
#[ignore]
fn test_break_trusted_strings() {
    // Break scripts
    expect("'<script>'", "\"<script>\"");
    expect("'</script>'", "\"\\x3c/script>\"");
    expect("\"</script> </SCRIPT>\"", "\"\\x3c/script> \\x3c/SCRIPT>\"");

    expect("'-->'", "\"--\\x3e\"");
    expect("']]>'", "\"]]\\x3e\"");
    expect("' --></script>'", "\" --\\x3e\\x3c/script>\"");

    expect("/--> <\\/script>/g", "/--\\x3e <\\/script>/g");

    // Break HTML start comments. Certain versions of WebKit
    // begin an HTML comment when they see this.
    expect("'<!-- I am a string -->'", "\"\\x3c!-- I am a string --\\x3e\"");

    expect("'<=&>'", "\"<=&>\"");
}

#[test]
#[ignore]
fn test_break_untrusted_strings() {
    // trustedStrings = false;

    // Break scripts
    expect("'<script>'", "\"\\x3cscript\\x3e\"");
    expect("'</script>'", "\"\\x3c/script\\x3e\"");
    expect("\"</script> </SCRIPT>\"", "\"\\x3c/script\\x3e \\x3c/SCRIPT\\x3e\"");

    expect("'-->'", "\"--\\x3e\"");
    expect("']]>'", "\"]]\\x3e\"");
    expect("' --></script>'", "\" --\\x3e\\x3c/script\\x3e\"");

    expect("/--> <\\/script>/g", "/--\\x3e <\\/script>/g");

    // Break HTML start comments. Certain versions of WebKit
    // begin an HTML comment when they see this.
    expect("'<!-- I am a string -->'", "\"\\x3c!-- I am a string --\\x3e\"");

    expect("'<=&>'", "\"\\x3c\\x3d\\x26\\x3e\"");
    expect("/(?=x)/", "/(?=x)/");
}

#[test]
#[ignore]
fn test_html_comments() {
    expect("3< !(--x)", "3< !--x");
    expect("while (x-- > 0) {}", "while(x-- >0);");
}

#[test]
#[ignore]
fn test_print_array() {
    expect("[void 0, void 0]", "[void 0,void 0]");
    expect("[undefined, undefined]", "[undefined,undefined]");
    expect("[ , , , undefined]", "[,,,undefined]");
    expect("[ , , , 0]", "[,,,0]");
}

#[test]
#[ignore]
fn test_hook() {
    expect("a ? b = 1 : c = 2", "a?b=1:c=2");
    expect("x = a ? b = 1 : c = 2", "x=a?b=1:c=2");
    expect("(x = a) ? b = 1 : c = 2", "(x=a)?b=1:c=2");

    expect("x, a ? b = 1 : c = 2", "x,a?b=1:c=2");
    expect("x, (a ? b = 1 : c = 2)", "x,a?b=1:c=2");
    expect("(x, a) ? b = 1 : c = 2", "(x,a)?b=1:c=2");

    expect("a ? (x, b) : c = 2", "a?(x,b):c=2");
    expect("a ? b = 1 : (x,c)", "a?b=1:(x,c)");

    expect("a ? b = 1 : c = 2 + x", "a?b=1:c=2+x");
    expect("(a ? b = 1 : c = 2) + x", "(a?b=1:c=2)+x");
    expect("a ? b = 1 : (c = 2) + x", "a?b=1:(c=2)+x");

    expect("a ? (b?1:2) : 3", "a?b?1:2:3");
}

#[test]
#[ignore]
fn test_for_in() {
    expect_same("for(a in b)c");
    expect_same("for(var a in b)c");
    expect_same("for(var a in b=c)d");
    expect_same("for(var a in b,c)d");
}

#[test]
#[ignore]
fn test_print_in_operator_in_for_loop() {
    // Check for in expression in for's init expression.
    // Check alone, with + (higher precedence), with ?: (lower precedence),
    // and with conditional.
    expect(
        "var a={}; for (var i = (\"length\" in a); i;) {}",
        "var a={};for(var i=(\"length\"in a);i;);",
    );
    expect(
        "var a={}; for (var i = (\"length\" in a) ? 0 : 1; i;) {}",
        "var a={};for(var i=(\"length\"in a)?0:1;i;);",
    );
    expect(
        "var a={}; for (var i = (\"length\" in a) + 1; i;) {}",
        "var a={};for(var i=(\"length\"in a)+1;i;);",
    );
    expect(
        "var a={};for (var i = (\"length\" in a|| \"size\" in a);;);",
        "var a={};for(var i=(\"length\"in a)||(\"size\"in a);;);",
    );
    expect(
        "var a={};for (var i = (a || a) || (\"size\" in a);;);",
        "var a={};for(var i=a||a||(\"size\"in a);;);",
    );

    // Test works with unary operators and calls.
    expect(
        "var a={}; for (var i = -(\"length\" in a); i;) {}",
        "var a={};for(var i=-(\"length\"in a);i;);",
    );
    // expect(
    // "var a={};function b_(p){ return p;};" + "for(var i=1,j=b_(\"length\" in a);;) {}",
    // "var a={};function b_(p){return p}" + "for(var i=1,j=b_(\"length\"in a);;);",
    // );

    // Test we correctly handle an in operator in the test clause.
    expect("var a={}; for (;(\"length\" in a);) {}", "var a={};for(;\"length\"in a;);");

    // Test we correctly handle an in operator inside a comma.
    expect_same("for(x,(y in z);;)foo()");
    expect_same("for(var x,w=(y in z);;)foo()");

    // And in operator inside a hook.
    expect_same("for(a=c?0:(0 in d);;)foo()");

    // And inside an arrow function body
    expect("var a={}; for(var i = () => (0 in a); i;) {}", "var a={};for(var i=()=>(0 in a);i;);");
    expect("var a={}; for(var i = () => ({} in a); i;) {}", "var a={};for(var i=()=>({}in a);i;);");

    // And inside a destructuring declaration
    expect(
        "var a={}; for(var {noop} = (\"prop\" in a); noop;) {}",
        "var a={};for(var {noop}=(\"prop\"in a);noop;);",
    );
}

#[test]
#[ignore]
fn test_for_of() {
    expect_same("for(a of b)c");
    expect_same("for(var a of b)c");
    expect_same("for(var a of b=c)d");
    expect_same("for(var a of(b,c))d");
}

// In pretty-print mode, make sure there is a space before and after the 'of' in a for/of loop.
#[test]
#[ignore]
fn test_for_of_pretty() {
    expect_same("for ([x, y] of b) {\n  c;\n}\n");
    expect_same("for (x of [[1, 2]]) {\n  c;\n}\n");
    expect_same("for ([x, y] of [[1, 2]]) {\n  c;\n}\n");
}

#[test]
#[ignore]
fn test_for_await_of() {
    expect_same("async()=>{for await(a of b)c}");
    expect_same("async()=>{for await(var a of b)c}");
    expect_same("async()=>{for await(var a of b=c)d}");
    expect_same("async()=>{for await(var a of(b,c))d}");
}

// In pretty-print mode, make sure there is a space before and after the 'of' in a for/of loop.
#[test]
#[ignore]
fn test_for_await_of_pretty() {
    expect_same("async() => {\n  for await ([x, y] of b) {\n    c;\n  }\n};\n");
    expect_same("async() => {\n  for await (x of [[1, 2]]) {\n    c;\n  }\n};\n");
    expect_same("async() => {\n  for await ([x, y] of [[1, 2]]) {\n    c;\n  }\n};\n");
}

#[test]
#[ignore]
fn test_let_for() {
    expect_same("for(let a=0;a<5;a++)b");
    expect_same("for(let a in b)c");
    expect_same("for(let a of b)c");

    expect_same("async()=>{for await(let a of b)c}");
}

#[test]
#[ignore]
fn test_const_for() {
    expect_same("for(const a=5;b<a;b++)c");
    expect_same("for(const a in b)c");
    expect_same("for(const a of b)c");

    expect_same("async()=>{for await(const a of b)c}");
}

#[test]
#[ignore]
fn test_literal_property() {
    expect("(64).toString()", "(64).toString()");
}

// Make sure that the code generator doesn't associate an
// else clause with the wrong if clause.
#[test]
#[ignore]
fn test_ambiguous_else_clauses() {
    // expectNode(
    // "if(x)if(y);else;",
    // new Node(
    // Token.IF,
    // Node.newString(Token.NAME, "x"),
    // new Node(
    // Token.BLOCK,
    // new Node(
    // Token.IF,
    // Node.newString(Token.NAME, "y"),
    // new Node(Token.BLOCK),

    // // ELSE clause for the inner if
    // new Node(Token.BLOCK)))));

    // expectNode(
    // "if(x){if(y);}else;",
    // new Node(
    // Token.IF,
    // Node.newString(Token.NAME, "x"),
    // new Node(
    // Token.BLOCK,
    // new Node(Token.IF, Node.newString(Token.NAME, "y"), new Node(Token.BLOCK))),

    // // ELSE clause for the outer if
    // new Node(Token.BLOCK)));

    // expectNode(
    // "if(x)if(y);else{if(z);}else;",
    // new Node(
    // Token.IF,
    // Node.newString(Token.NAME, "x"),
    // new Node(
    // Token.BLOCK,
    // new Node(
    // Token.IF,
    // Node.newString(Token.NAME, "y"),
    // new Node(Token.BLOCK),
    // new Node(
    // Token.BLOCK,
    // new Node(
    // Token.IF, Node.newString(Token.NAME, "z"), new Node(Token.BLOCK))))),

    // // ELSE clause for the outermost if
    // new Node(Token.BLOCK)));
}

#[test]
#[ignore]
fn test_line_break() {
    // // line break after function if in a statement context
    // assertLineBreak(
    // "function a() {}\n" + "function b() {}", "function a(){}\n" + "function b(){}\n");

    // // line break after ; after a function
    // assertLineBreak(
    // "var a = {};\n" + "a.foo = function () {}\n" + "function b() {}",
    // "var a={};a.foo=function(){};\n" + "function b(){}\n");

    // // break after comma after a function
    // assertLineBreak(
    // "var a = {\n" + "  b: function() {},\n" + "  c: function() {}\n" + "};\n" + "alert(a);",
    // "var a={b:function(){},\n" + "c:function(){}};\n" + "alert(a)");
}

#[test]
#[ignore]
fn test_pretty_printer() {
    // Ensure that the pretty printer inserts line breaks at appropriate
    // places.
    expect("(function(){})();", "(function() {\n})();\n");
    expect("var a = (function() {});alert(a);", "var a = function() {\n};\nalert(a);\n");

    // Check we correctly handle putting brackets around all if clauses so
    // we can put breakpoints inside statements.
    // expect("if (1) {}", "if (1) {\n" + "}\n");
    // expect("if (1) {alert(\"\");}", "if (1) {\n" + "  alert(\"\");\n" + "}\n");
    // expect("if (1)alert(\"\");", "if (1) {\n" + "  alert(\"\");\n" + "}\n");
    // expect("if (1) {alert();alert();}", "if (1) {\n" + "  alert();\n" + "  alert();\n" + "}\n");

    // Don't add blocks if they weren't there already.
    expect("label: alert();", "label: alert();\n");

    // But if statements and loops get blocks automagically.
    // expect("if (1) alert();", "if (1) {\n" + "  alert();\n" + "}\n");
    // expect("for (;;) alert();", "for (;;) {\n" + "  alert();\n" + "}\n");

    // expect("while (1) alert();", "while (1) {\n" + "  alert();\n" + "}\n");

    // Do we put else clauses in blocks?
    // expect("if (1) {} else {alert(a);}", "if (1) {\n" + "} else {\n  alert(a);\n}\n");

    // Do we add blocks to else clauses?
    // expect(
    // "if (1) alert(a); else alert(b);",
    // "if (1) {\n" + "  alert(a);\n" + "} else {\n" + "  alert(b);\n" + "}\n",
    // );

    // Do we put for bodies in blocks?
    // expect("for(;;) { alert();}", "for (;;) {\n" + "  alert();\n" + "}\n");
    // expect("for(;;) {}", "for (;;) {\n" + "}\n");
    // expect(
    // "for(;;) { alert(); alert(); }",
    // "for (;;) {\n" + "  alert();\n" + "  alert();\n" + "}\n",
    // );
    // expect(
    // "for(var x=0;x<10;x++) { alert(); alert(); }",
    // "for (var x = 0; x < 10; x++) {\n" + "  alert();\n" + "  alert();\n" + "}\n",
    // );

    // How about do loops?
    // expect("do { alert(); } while(true);", "do {\n" + "  alert();\n" + "} while (true);\n");

    // label?
    // expect("myLabel: { alert();}", "myLabel: {\n" + "  alert();\n" + "}\n");
    expect("myLabel: {}", "myLabel: {\n}\n");
    expect("myLabel: ;", "myLabel: ;\n");

    // Don't move the label on a loop, because then break {label} and
    // continue {label} won't work.
    // expect(
    // "myLabel: for(;;) continue myLabel;",
    // "myLabel: for (;;) {\n" + "  continue myLabel;\n" + "}\n",
    // );

    expect("var a;", "var a;\n");
    expect("i--", "i--;\n");
    expect("i++", "i++;\n");

    // There must be a space before and after binary operators.
    expect("var foo = 3+5;", "var foo = 3 + 5;\n");

    // There should be spaces between the ternary operator
    expect("var foo = bar ? 3 : null;", "var foo = bar ? 3 : null;\n");

    // Ensure that string literals after return and throw have a space.
    expect("function foo() { return \"foo\"; }", "function foo() {\n  return \"foo\";\n}\n");
    expect("throw \"foo\";", "throw \"foo\";\n");

    // Test that loops properly have spaces inserted.
    expect("do{ alert(); } while(true);", "do {\n  alert();\n} while (true);\n");
    expect("while(true) { alert(); }", "while (true) {\n  alert();\n}\n");
}

#[test]
#[ignore]
fn test_pretty_printer2() {
    // expect("if(true) f();", "if (true) {\n" + "  f();\n" + "}\n");

    // expect(
    // "if (true) { f() } else { g() }",
    // "if (true) {\n" + "  f();\n" + "} else {\n" + "  g();\n" + "}\n",
    // );

    // expect(
    // "if(true) f(); for(;;) g();",
    // "if (true) {\n" + "  f();\n" + "}\n" + "for (;;) {\n" + "  g();\n" + "}\n",
    // );
}

#[test]
#[ignore]
fn test_pretty_printer3() {
    // expect(
    // "try {} catch(e) {}if (1) {alert();alert();}",
    // "try {\n"
    // + "} catch (e) {\n"
    // + "}\n"
    // + "if (1) {\n"
    // + "  alert();\n"
    // + "  alert();\n"
    // + "}\n",
    // );

    // expect(
    // "try {} finally {}if (1) {alert();alert();}",
    // "try {\n"
    // + "} finally {\n"
    // + "}\n"
    // + "if (1) {\n"
    // + "  alert();\n"
    // + "  alert();\n"
    // + "}\n",
    // );

    // expect(
    // "try {} catch(e) {} finally {} if (1) {alert();alert();}",
    // "try {\n"
    // + "} catch (e) {\n"
    // + "} finally {\n"
    // + "}\n"
    // + "if (1) {\n"
    // + "  alert();\n"
    // + "  alert();\n"
    // + "}\n",
    // );
}

#[test]
#[ignore]
fn test_pretty_printer4() {
    // expect(
    // "function f() {}if (1) {alert();}",
    // "function f() {\n" + "}\n" + "if (1) {\n" + "  alert();\n" + "}\n",
    // );

    // expect(
    // "var f = function() {};if (1) {alert();}",
    // "var f = function() {\n" + "};\n" + "if (1) {\n" + "  alert();\n" + "}\n",
    // );

    // expect(
    // "(function() {})();if (1) {alert();}",
    // "(function() {\n" + "})();\n" + "if (1) {\n" + "  alert();\n" + "}\n",
    // );

    // expect(
    // "(function() {alert();alert();})();if (1) {alert();}",
    // "(function() {\n"
    // + "  alert();\n"
    // + "  alert();\n"
    // + "})();\n"
    // + "if (1) {\n"
    // + "  alert();\n"
    // + "}\n",
    // );
}

#[test]
#[ignore]
fn test_pretty_printer_arrow() {
    expect("(a)=>123;", "a => 123;\n");
}

#[test]
#[ignore]
fn test_pretty_printer_default_value() {
    expect("(a=1)=>123;", "(a = 1) => 123;\n");
    expect("[a=(1,2)]=[];", "[a = (1, 2)] = [];\n");
}

// For https://github.com/google/closure-compiler/issues/782
#[test]
#[ignore]
fn test_pretty_printer_space_before_single_quote() {
    // expect(
    // "var f = function() { return 'hello'; };",
    // "var f = function() {\n" + "  return 'hello';\n" + "};\n",
    // new CompilerOptionBuilder() {
    // @Override
    // void setOptions(CompilerOptions options) {
    // options.setPreferSingleQuotes(true);
    // }
    // });
}

// For https://github.com/google/closure-compiler/issues/782
#[test]
#[ignore]
fn test_pretty_printer_space_before_unary_operators() {
    // expect(
    // "var f = function() { return !b; };",
    // "var f = function() {\n" + "  return !b;\n" + "};\n",
    // );
    // expect("var f = function*(){yield -b}", "var f = function*() {\n" + "  yield -b;\n" + "};\n");
    // expect(
    // "var f = function() { return +b; };",
    // "var f = function() {\n" + "  return +b;\n" + "};\n",
    // );
    // expect(
    // "var f = function() { throw ~b; };",
    // "var f = function() {\n" + "  throw ~b;\n" + "};\n",
    // );
    // expect(
    // "var f = function() { return ++b; };",
    // "var f = function() {\n" + "  return ++b;\n" + "};\n",
    // );
    // expect(
    // "var f = function() { return --b; };",
    // "var f = function() {\n" + "  return --b;\n" + "};\n",
    // );
}

#[test]
#[ignore]
fn test_pretty_printer_var_let_const() {
    expect("var x=0;", "var x = 0;\n");
    expect("const x=0;", "const x = 0;\n");
    expect("let x=0;", "let x = 0;\n");
}

#[test]
#[ignore]
fn test_pretty_printer_number() {
    expect_same("var x = 10;\n");
    expect_same("var x = 1.;\n");
    expect("var x = 0xFE;", "var x = 254;\n");
    // expect_same("var x = 1" + String.format("%0100d", 0) + ";\n"); // a googol
    expect_same("f(10000);\n");
    expect("var x = -10000;\n", "var x = -10000;\n");
    expect("var x = y - -10000;\n", "var x = y - -10000;\n");
    expect("f(-10000);\n", "f(-10000);\n");
    expect_same("x < 2592000;\n");
    expect_same("x < 1000.000;\n");
    expect_same("x < 1000.912;\n");
    expect_same("var x = 1E20;\n");
    expect_same("var x = 1E1;\n");
    expect_same("var x = void 0;\n");
    expect_same("foo(-0);\n");
    expect("var x = 4-1000;", "var x = 4 - 1000;\n");
}

#[test]
#[ignore]
fn test_type_annotations() {
    // assertTypeAnnotations(
    // "/** @constructor */ function Foo(){}",
    // "/**\n * @constructor\n */\nfunction Foo() {\n}\n",
    // );
}

#[test]
#[ignore]
fn test_non_null_types() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @constructor */",
    // "function Foo() {}",
    // "/** @return {!Foo} */",
    // "Foo.prototype.f = function() { return new Foo; };"
    // ),
    // lines!(
    // "/**",
    // " * @constructor",
    // " */",
    // "function Foo() {\n}",
    // "/**",
    // " * @return {!Foo}",
    // " */",
    // "Foo.prototype.f = function() {",
    // "  return new Foo();",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_type_def() {
    // TODO(johnlenz): It would be nice if there were some way to preserve
    // typedefs but currently they are resolved into the basic types in the
    // type registry.
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var goog = {};",
    // "/** @const */ goog.java = {};",
    // "/** @typedef {Array<number>} */ goog.java.Long;",
    // "/** @param {!goog.java.Long} a*/",
    // "function f(a){};"
    // ),
    // lines!(
    // "/** @const */ var goog = {};",
    // "/** @const */ goog.java = {};",
    // "goog.java.Long;",
    // "/**",
    // " * @param {!Array<number>} a",
    // " * @return {undefined}",
    // " */",
    // "function f(a) {\n}\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_assign() {
    // assertTypeAnnotations(
    // "/** @constructor */ var Foo = function(){}",
    // lines!("/**\n * @constructor\n */", "var Foo = function() {\n};\n"),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_namespace_var_without_js_doc() {
    // assertTypeAnnotations(
    // lines!(
    // "var a = {};", //
    // "/** @constructor */ a.Foo = function(){}"
    // ),
    // lines!(
    // "var a = {};", //
    // "/**",
    // " * @constructor",
    // " */",
    // "a.Foo = function() {",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_namespace_var_with_const_js_doc() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */", //
    // "var a = {};",
    // "/** @constructor */ a.Foo = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/**",
    // " * @constructor",
    // " */",
    // "a.Foo = function() {",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_namespace_const_declaration_without_js_doc() {
    // assertTypeAnnotations(
    // lines!(
    // "const a = {};", //
    // "/** @constructor */ a.Foo = function(){}"
    // ),
    // lines!(
    // "const a = {};", //
    // "/**",
    // " * @constructor",
    // " */",
    // "a.Foo = function() {",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_namespace_const_declaration_with_js_doc() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @export */",
    // "const a = {};", //
    // "/** @constructor */ a.Foo = function(){}"
    // ),
    // lines!(
    // "/** @export */ const a = {};", //
    // "/**",
    // " * @constructor",
    // " */",
    // "a.Foo = function() {",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_namespace_qname_with_const_js_doc() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */",
    // "var a = {};",
    // "/** @const */",
    // "a.b = {};",
    // "/** @constructor */ a.b.Foo = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/** @const */ a.b = {};",
    // "/**",
    // " * @constructor",
    // " */",
    // "a.b.Foo = function() {",
    // "};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_member_subclass() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var a = {};",
    // "/** @constructor */ a.Foo = function(){};",
    // "/** @constructor \n @extends {a.Foo} */ a.Bar = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/**\n * @constructor\n */",
    // "a.Foo = function() {\n};",
    // "/**\n * @extends {a.Foo}",
    // " * @constructor\n */",
    // "a.Bar = function() {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_interface() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var a = {};",
    // "/** @interface */ a.Foo = function(){};",
    // "/** @interface \n @extends {a.Foo} */ a.Bar = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/**\n * @interface\n */",
    // "a.Foo = function() {\n};",
    // "/**\n * @extends {a.Foo}",
    // " * @interface\n */",
    // "a.Bar = function() {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_multiple_interface() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var a = {};",
    // "/** @interface */ a.Foo1 = function(){};",
    // "/** @interface */ a.Foo2 = function(){};",
    // "/** @interface \n @extends {a.Foo1} \n @extends {a.Foo2} */",
    // "a.Bar = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/**\n * @interface\n */",
    // "a.Foo1 = function() {\n};",
    // "/**\n * @interface\n */",
    // "a.Foo2 = function() {\n};",
    // "/**\n * @extends {a.Foo1}",
    // " * @extends {a.Foo2}",
    // " * @interface\n */",
    // "a.Bar = function() {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_member() {
    // assertTypeAnnotations(
    // lines!(
    // "var a = {};",
    // "/** @constructor */ a.Foo = function(){}",
    // "/** @param {string} foo",
    // "  * @return {number} */",
    // "a.Foo.prototype.foo = function(foo) { return 3; };",
    // "/** @type {!Array|undefined} */",
    // "a.Foo.prototype.bar = [];"
    // ),
    // lines!(
    // "var a = {};",
    // "/**\n * @constructor\n */",
    // "a.Foo = function() {\n};",
    // "/**",
    // " * @param {string} foo",
    // " * @return {number}",
    // " */",
    // "a.Foo.prototype.foo = function(foo) {\n  return 3;\n};",
    // "/** @type {!Array<?>} */",
    // "a.Foo.prototype.bar = [];\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotations_member_stub() {
    // TODO(blickly): Investigate why the method's type isn't preserved.
    // assertTypeAnnotations(
    // lines!(
    // "/** @interface */ function I(){};",
    // "/** @return {undefined} @param {number} x */ I.prototype.method;"
    // ),
    // "/**\n * @interface\n */\nfunction I() {\n}\nI.prototype.method;\n",
    // );
}

#[test]
#[ignore]
fn test_type_annotations_implements() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var a = {};",
    // "/** @constructor */ a.Foo = function(){};",
    // "/** @interface */ a.I = function(){};",
    // "/** @record */ a.I2 = function(){};",
    // "/** @record @extends {a.I2} */ a.I3 = function(){};",
    // "/** @constructor \n @extends {a.Foo}",
    // " * @implements {a.I} \n @implements {a.I2}",
    // " */ a.Bar = function(){}"
    // ),
    // lines!(
    // "/** @const */ var a = {};",
    // "/**\n * @constructor\n */",
    // "a.Foo = function() {\n};",
    // "/**\n * @interface\n */",
    // "a.I = function() {\n};",
    // "/**\n * @record\n */",
    // "a.I2 = function() {\n};",
    // "/**\n * @extends {a.I2}",
    // " * @record\n */",
    // "a.I3 = function() {\n};",
    // "/**\n * @extends {a.Foo}",
    // " * @implements {a.I}",
    // " * @implements {a.I2}",
    // " * @constructor\n */",
    // "a.Bar = function() {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotation_class_implements() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @interface */ class Foo {}", //
    // "/** @implements {Foo} */ class Bar {}"
    // ),
    // lines!(
    // "/**\n * @interface\n */",
    // "class Foo {\n}",
    // "/**\n * @implements {Foo}\n */",
    // "class Bar {\n}\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotation_class_member() {
    // assertTypeAnnotations(
    // lines!(
    // "class Foo {", //
    // "  /** @return {number} */ method(/** string */ arg) {}",
    // "}"
    // ),
    // lines!(
    // "class Foo {",
    // "  /**",
    // "   * @param {string} arg",
    // "   * @return {number}",
    // "   */",
    // "  method(arg) {",
    // "  }",
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_type_annotation_class_constructor() {
    // assertTypeAnnotations(
    // lines!(
    // "/**",
    // " * @template T",
    // " */",
    // "class Foo {", //
    // "  /** @param {T} arg */",
    // "  constructor(arg) {}",
    // "}"
    // ),
    // lines!(
    // "/**",
    // " * @template T",
    // " */",
    // "class Foo {",
    // "  /**",
    // "   * @param {T} arg",
    // "   */",
    // "  constructor(arg) {",
    // "  }",
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_rest_parameter() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {...string} args */", //
    // "function f(...args) {}"
    // ),
    // lines!(
    // "/**\n * @param {...string} args\n * @return {undefined}\n */",
    // "function f(...args) {\n}\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_default_parameter() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {string=} msg */", //
    // "function f(msg = 'hi') {}"
    // ),
    // lines!(
    // "/**\n * @param {string=} msg\n * @return {undefined}\n */",
    // "function f(msg = \"hi\") {\n}\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_object_destructuring_parameter() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {{a: number, b: number}} ignoredName */", //
    // "function f({a, b}) {}"
    // ),
    // lines!(
    // "/**",
    // " * @param {{a: number, b: number}} p0", // old JSDoc name is ignored
    // " * @return {undefined}",
    // " */",
    // "function f({a, b}) {", // whitespace in output must match
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_object_destructuring_parameter_with_default() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {{a: number, b: number}=} ignoredName */", //
    // "function f({a, b} = {a: 1, b: 2}) {}"
    // ),
    // lines!(
    // "/**",
    // " * @param {{a: number, b: number}=} p0", // old JSDoc name is ignored
    // " * @return {undefined}",
    // " */",
    // "function f({a, b} = {a:1, b:2}) {", // whitespace in output must match
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_array_destructuring_parameter() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {!Iterable<number>} ignoredName */", //
    // "function f([a, b]) {}"
    // ),
    // lines!(
    // "/**",
    // " * @param {!Iterable<number>} p0", // old JSDoc name is ignored
    // " * @return {undefined}",
    // " */",
    // "function f([a, b]) {", // whitespace in output must match
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_array_destructuring_parameter_with_default() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @param {!Iterable<number>=} ignoredName */", //
    // "function f([a, b] = [1, 2]) {}"
    // ),
    // lines!(
    // "/**",
    // " * @param {!Iterable<number>=} p0", // old JSDoc name is ignored
    // " * @return {undefined}",
    // " */",
    // "function f([a, b] = [1, 2]) {", // whitespace in output must match
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_u2_u_function_type_annotation1() {
    // assertTypeAnnotations(
    // "/** @type {!Function} */ var x = function() {}",
    // "/** @type {!Function} */\nvar x = function() {\n};\n",
    // );
}

#[test]
#[ignore]
fn test_u2_u_function_type_annotation2() {
    // TODO(johnlenz): we currently report the type of the RHS which is not
    // correct, we should export the type of the LHS.
    // assertTypeAnnotations(
    // "/** @type {Function} */ var x = function() {}",
    // "/** @type {!Function} */\nvar x = function() {\n};\n",
    // );
}

#[test]
#[ignore]
fn test_emit_unknown_param_types_as_all_type() {
    // x is unused, so NTI infers that x can be omitted.
    // assertTypeAnnotations(
    // "var a = function(x) {}",
    // lines!(
    // "/**",
    // " * @param {?} x",
    // " * @return {undefined}",
    // " */",
    // "var a = function(x) {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_optional_types_annotation() {
    // assertTypeAnnotations(
    // "/** @param {string=} x */ var a = function(x) {}",
    // lines!(
    // "/**",
    // " * @param {string=} x",
    // " * @return {undefined}",
    // " */",
    // "var a = function(x) {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_optional_types_annotation2() {
    // assertTypeAnnotations(
    // "/** @param {undefined=} x */ var a = function(x) {}",
    // lines!(
    // "/**",
    // " * @param {undefined=} x",
    // " * @return {undefined}",
    // " */",
    // "var a = function(x) {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_variable_arguments_types_annotation() {
    // assertTypeAnnotations(
    // "/** @param {...string} x */ var a = function(x) {}",
    // lines!(
    // "/**",
    // " * @param {...string} x",
    // " * @return {undefined}",
    // " */",
    // "var a = function(x) {\n};\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_temp_constructor() {
    // assertTypeAnnotations(
    // lines!(
    // "var x = function() {",
    // "  /** @constructor */ function t1() {}",
    // "  /** @constructor */ function t2() {}",
    // "  t1.prototype = t2.prototype",
    // "}"
    // ),
    // lines!(
    // "/**",
    // " * @return {undefined}",
    // " */",
    // "var x = function() {",
    // "  /**",
    // "   * @constructor",
    // "   */",
    // "  function t1() {",
    // "  }",
    // "  /**",
    // "   * @constructor",
    // "   */",
    // "  function t2() {",
    // "  }",
    // "  t1.prototype = t2.prototype;",
    // "};",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_enum_annotation1() {
    // assertTypeAnnotations(
    // "/** @enum {string} */ const Enum = {FOO: 'x', BAR: 'y'};",
    // "/** @enum {string} */\nconst Enum = {FOO:\"x\", BAR:\"y\"};\n",
    // );
}

#[test]
#[ignore]
fn test_enum_annotation2() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var goog = goog || {};",
    // "/** @enum {string} */ goog.Enum = {FOO: 'x', BAR: 'y'};",
    // "/** @const */ goog.Enum2 = goog.x ? {} : goog.Enum;"
    // ),
    // lines!(
    // "/** @const */ var goog = goog || {};",
    // "/** @enum {string} */\ngoog.Enum = {FOO:\"x\", BAR:\"y\"};",
    // "/** @type {(!Object|{})} */\ngoog.Enum2 = goog.x ? {} : goog.Enum;\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_enum_annotation3() {
    // assertTypeAnnotations(
    // "/** @enum {!Object} */ var Enum = {FOO: {}};",
    // "/** @enum {!Object} */\nvar Enum = {FOO:{}};\n",
    // );
}

#[test]
#[ignore]
fn test_enum_annotation4() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @enum {number} */ var E = {A:1, B:2};",
    // "function f(/** !E */ x) { return x; }"
    // ),
    // lines!(
    // "/** @enum {number} */",
    // "var E = {A:1, B:2};",
    // "/**",
    // " * @param {number} x",
    // " * @return {?}",
    // " */",
    // "function f(x) {",
    // "  return x;",
    // "}",
    // ""
    // ),
    // );
}

#[test]
#[ignore]
fn test_closure_library_type_annotation_examples() {
    // assertTypeAnnotations(
    // lines!(
    // "/** @const */ var goog = goog || {};",
    // "/** @param {Object} obj */goog.removeUid = function(obj) {};",
    // "/** @param {Object} obj The object to remove the field from. */",
    // "goog.removeHashCode = goog.removeUid;"
    // ),
    // lines!(
    // "/** @const */ var goog = goog || {};",
    // "/**",
    // " * @param {(Object|null)} obj",
    // " * @return {undefined}",
    // " */",
    // "goog.removeUid = function(obj) {",
    // "};",
    // "/**",
    // " * @param {(Object|null)} p0",
    // " * @return {undefined}",
    // " */",
    // "goog.removeHashCode = goog.removeUid;\n"
    // ),
    // );
}

#[test]
#[ignore]
fn test_function_type_annotation() {
    // assertTypeAnnotations(
    // "/**\n * @param {{foo:number}} arg\n */\nfunction f(arg) {}",
    // "/**\n * @param {{foo: number}} arg\n * @return {undefined}\n */\nfunction f(arg) {\n}\n",
    // );
    // assertTypeAnnotations(
    // "/**\n * @param {number} arg\n */\nfunction f(arg) {}",
    // "/**\n * @param {number} arg\n * @return {undefined}\n */\nfunction f(arg) {\n}\n",
    // );
    // assertTypeAnnotations(
    // "/**\n * @param {!Array<string>} arg\n */\nfunction f(arg) {}",
    // "/**\n * @param {!Array<string>} arg\n * @return {undefined}\n */\nfunction f(arg) {\n}\n",
    // );
}

#[test]
#[ignore]
fn test_function_with_this_type_annotation() {
    // assertTypeAnnotations(
    // "/**\n * @this {{foo:number}}\n */\nfunction foo() {}",
    // "/**\n * @return {undefined}\n * @this {{foo: number}}\n */\nfunction foo() {\n}\n",
    // );
    // assertTypeAnnotations(
    // "/**\n * @this {!Array<string>}\n */\nfunction foo() {}",
    // "/**\n * @return {undefined}\n * @this {!Array<string>}\n */\nfunction foo() {\n}\n",
    // );
}

#[test]
#[ignore]
fn test_return_with_type_annotation() {
    // preserveTypeAnnotations = true;
    expect(
        "function f() { return (/** @return {number} */ function() { return 42; }); }",
        lines!(
            "function f() {",
            "  return (/**",
            "   * @return {number}",
            "   */",
            "  function() {",
            "    return 42;",
            "  });",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_deprecated_annotation_includes_newline() {
    // String js =
    // lines!(
    // "/**",
    // " * @type {number}",
    // " * @deprecated See {@link replacementClass} for more details.",
    // " */",
    // "var x;",
    // "");

    // expect(js, js);
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_non_trailing_block_comment() {
    // preserveNonJSDocComments = true;
    expect("/* test_comment */ function Foo(){}", "/* testComment */ function Foo() {\n}\n");
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_end_of_file_line_comment() {
    // preserveNonJSDocComments = true;
    expect(
        lines!(
            "function f1() {}", //
            "if (true) {",
            "// first",
            "f1();",
            "}",
            "// second"
        ),
        lines!(
            "function f1() {\n}", //
            "if (true) {",
            "  // first",
            "  f1();",
            "}",
            " // second\n"
        ),
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_end_of_block_comment() {
    // preserveNonJSDocComments = true;
    expect(
        lines!(
            "function f1() {}", //
            "if (true) {",
            "// first",
            "f1();",
            "/* second */",
            "}"
        ),
        lines!(
            "function f1() {\n}", //
            "if (true) {",
            "  // first",
            "  f1(); ",
            "  /* second */",
            "}\n"
        ),
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_end_of_block_many_mixed_comments() {
    // preserveNonJSDocComments = true;
    expect(
        lines!(
            "function f1() {}", //
            "if (true) {",
            "// first",
            "f1();",
            "// second",
            "/* third */",
            "// fourth",
            "}"
        ),
        lines!(
            "function f1() {\n}", //
            "if (true) {",
            "  // first",
            "  f1(); ",
            "  // second",
            "  /* third */",
            "  // fourth",
            "}\n"
        ),
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_last_trailing() {
    // preserveNonJSDocComments = true;
    expect(
        lines!(
            "function f1() {}", //
            "if (true) {",
            "// first",
            "f1(); // second ",
            "}"
        ),
        lines!(
            "function f1() {\n}", //
            "if (true) {",
            "  // first",
            "  f1(); // second",
            "}\n"
        ),
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_non_trailing_line_comment() {
    // preserveNonJSDocComments = true;
    expect("// test_comment\nfunction Foo(){}", "// testComment\nfunction Foo() {\n}\n");
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_between_code_same_line() {
    // preserveNonJSDocComments = true;

    expect("function /* test_comment */ Foo(){}", "function/* testComment */ Foo() {\n}\n");
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_between_code_differentlines() {
    // preserveNonJSDocComments = true;
    expect("function /* test_comment */\nFoo(){}", "function/* testComment */\nFoo() {\n}\n");
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_non_trailing_inline_comments() {
    // preserveNonJSDocComments = true;
    // tests inline comments in parameter lists are parsed and printed
    expect(
        "function Foo(/*first*/ x, /* second*/ y) {}",
        "function Foo(/*first*/ x, /* second*/ y) {\n}\n",
    );
}

// Args on new line are condensed onto the same line by prettyPrint
#[test]
#[ignore]
fn test_args_no_comments_newlines() {
    // expect(
    // lines!(" var rpcid = new RpcId(a,\n b, \nc);"),
    // lines!("var rpcid = new RpcId(a, b, c);\n"),
    // );
}

// Comments are printed when args on new line are condensed onto the same line by prettyPrint
#[test]
#[ignore]
fn test_non_js_doc_comments_printed_non_trailing_inline_comments_newlines() {
    // preserveNonJSDocComments = true;
    // expect(
    // lines!(" var rpcid = new RpcId(a,\n /* comment1 */ b, \n/* comment1 */ c);"),
    // lines!("var rpcid = new RpcId(a, /* comment1 */ b, /* comment1 */ c);\n"),
    // );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_trailing_and_non_trailing_inline_comments() {
    // preserveNonJSDocComments = true;
    // tests inline trailing comments in parameter lists are parsed and printed
    expect(
        "function Foo(x //first\n, /* second*/ y) {}",
        "function Foo(x //first\n, /* second*/ y) {\n}\n",
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_trailing_inline_comments_param_list() {
    // preserveNonJSDocComments = true;
    expect("function Foo(x) {}", "function Foo(x) {\n}\n");
    expect("function Foo(x /*first*/) {}", "function Foo(x /*first*/) {\n}\n");
    expect("function Foo(x //first\n) {}", "function Foo(x //first\n) {\n}\n");
}

#[test]
#[ignore]
fn test_class_extends_left_hand_side_expression() {
    expect("class A {} class B extends (0, A) {}", "class A {\n}\nclass B extends(0, A) {\n}\n");
}

// Same as above, but tests argList instead of Param list
#[test]
#[ignore]
fn test_non_js_doc_comments_printed_trailing_inline_comments_call_arg_list() {
    expect("foo(x);", "foo(x);\n");
    expect("foo(x /*first*/);", "foo(x /*first*/);\n");
    expect("foo(x //first\n);", "foo(x //first\n);\n");
}

#[test]
#[ignore]
fn test_subtraction() {
    // Compiler compiler = new Compiler();
    // Node n = compiler.parse_test_code("x - -4");
    // assertThat(compiler.getErrorCount()).isEqualTo(0);

    // assertThat(printNode(n)).isEqualTo("x- -4");
}

#[test]
#[ignore]
fn test_function_with_call() {
    // expect(
    // "var user = new function() {" + "alert(\"foo\")}",
    // "var user=new function(){" + "alert(\"foo\")}",
    // );
    // expect(
    // "var user = new function() {"
    // + "this.name = \"foo\";"
    // + "this.local = function(){alert(this.name)};}",
    // "var user=new function(){"
    // + "this.name=\"foo\";"
    // + "this.local=function(){alert(this.name)}}",
    // );
}

#[test]
#[ignore]
fn test_line_length() {
    // list
    // assertLineLength("var aba,bcb,cdc", "var aba,bcb," + "\ncdc");

    // // operators, and two breaks
    // assertLineLength(
    // "\"foo\"+\"bar,baz,bomb\"+\"whee\"+\";long-string\"\n+\"aaa\"",
    // "\"foo\"+\"bar,baz,bomb\"+" + "\n\"whee\"+\";long-string\"+" + "\n\"aaa\"");

    // // assignment
    // assertLineLength("var abazaba=1234", "var abazaba=" + "\n1234");

    // // statements
    // assertLineLength("var abab=1;var bab=2", "var abab=1;" + "\nvar bab=2");

    // // don't break regexes
    // assertLineLength(
    // "var a=/some[reg](ex),with.*we?rd|chars/i;var b=a",
    // "var a=/some[reg](ex),with.*we?rd|chars/i;" + "\nvar b=a");

    // // don't break strings
    // assertLineLength("var a=\"foo,{bar};baz\";var b=a", "var a=\"foo,{bar};baz\";" + "\nvar b=a");

    // // don't break before post inc/dec
    // assertLineLength("var a=\"a\";a++;var b=\"bbb\";", "var a=\"a\";a++;\n" + "var b=\"bbb\"");
}

#[test]
#[ignore]
fn test_parse_print_parse() {
    expect_reparse("3;");
    expect_reparse("var a = b;");
    expect_reparse("var x, y, z;");
    expect_reparse("try { foo() } catch(e) { bar() }");
    expect_reparse("try { foo() } catch(e) { bar() } finally { stuff() }");
    expect_reparse("try { foo() } finally { stuff() }");
    expect_reparse("throw 'me'");
    expect_reparse("function foo(a) { return a + 4; }");
    expect_reparse("function foo() { return; }");
    expect_reparse("var a = function(a, b) { foo(); return a + b; }");
    expect_reparse("b = [3, 4, 'paul', \"Buchhe it\",,5];");
    expect_reparse("v = (5, 6, 7, 8)");
    expect_reparse("d = 34.0; x = 0; y = .3; z = -22");
    expect_reparse("d = -x; t = !x + ~y;");
    // expect_reparse(
    // "'hi'; /* just a test */ stuff(a,b) \n" + " foo(); // and another \n" + " bar();",
    // );
    expect_reparse("a = b++ + ++c; a = b++-++c; a = - --b; a = - ++b;");
    expect_reparse("a++; b= a++; b = ++a; b = a--; b = --a; a+=2; b-=5");
    expect_reparse("a = (2 + 3) * 4;");
    expect_reparse("a = 1 + (2 + 3) + 4;");
    expect_reparse("x = a ? b : c; x = a ? (b,3,5) : (foo(),bar());");
    // expect_reparse("a = b | c || d ^ e " + "&& f & !g != h << i <= j < k >>> l > m * n % !o");
    // expect_reparse("a == b; a != b; a === b; a == b == a;" + " (a == b) == a; a == (b == a);");
    expect_reparse("if (a > b) a = b; if (b < 3) a = 3; else c = 4;");
    expect_reparse("if (a == b) { a++; } if (a == 0) { a++; } else { a --; }");
    expect_reparse("for (var i in a) b += i;");
    // expect_reparse("for (var i = 0; i < 10; i++){ b /= 2;" + " if (b == 2)break;else continue;}");
    expect_reparse("for (x = 0; x < 10; x++) a /= 2;");
    expect_reparse("for (;;) a++;");
    expect_reparse("while(true) { blah(); }while(true) blah();");
    expect_reparse("do stuff(); while(a>b);");
    expect_reparse("[0, null, , true, false, this];");
    expect_reparse("s.replace(/absc/, 'X').replace(/ab/gi, 'Y');");
    expect_reparse("new Foo; new Bar(a, b,c);");
    expect_reparse("with(foo()) { x = z; y = t; } with(bar()) a = z;");
    expect_reparse("delete foo['bar']; delete foo;");
    expect_reparse("var x = { 'a':'paul', 1:'3', 2:(3,4) };");
    // expect_reparse(
    // "switch(a) { case 2: case 3: stuff(); break;"
    // + "case 4: morestuff(); break; default: done();}",
    // );
    expect_reparse("x = foo['bar'] + foo['my stuff'] + foo[bar] + f.stuff;");
    expect_reparse("a.v = b.v; x['foo'] = y['zoo'];");
    expect_reparse("'test' in x; 3 in x; a in x;");
    expect_reparse("'foo\"bar' + \"foo'c\" + 'stuff\\n and \\\\more'");
    expect_reparse("x.__proto__;");
}

#[test]
#[ignore]
fn test_do_loop_ie_compatibility() {
    // Do loops within IFs cause syntax errors in IE6 and IE7.
    expect(
        "function f(){if(e1){do foo();while(e2)}else foo()}",
        "function f(){if(e1){do foo();while(e2)}else foo()}",
    );

    expect(
        "function f(){if(e1)do foo();while(e2)else foo()}",
        "function f(){if(e1){do foo();while(e2)}else foo()}",
    );

    expect("if(x){do{foo()}while(y)}else bar()", "if(x){do foo();while(y)}else bar()");

    expect("if(x)do{foo()}while(y);else bar()", "if(x){do foo();while(y)}else bar()");

    expect("if(x){do{foo()}while(y)}", "if(x){do foo();while(y)}");

    expect("if(x)do{foo()}while(y);", "if(x){do foo();while(y)}");

    expect("if(x)A:do{foo()}while(y);", "if(x){A:do foo();while(y)}");

    expect(
        "var i = 0;a: do{b: do{i++;break b;} while(0);} while(0);",
        "var i=0;a:do{b:do{i++;break b}while(0)}while(0)",
    );
}

#[test]
#[ignore]
fn test_function_safari_compatibility() {
    // Functions within IFs cause syntax errors on Safari.
    expect(
        "function f(){if(e1){function goo(){return true}}else foo()}",
        "function f(){if(e1){function goo(){return true}}else foo()}",
    );

    expect(
        "function f(){if(e1)function goo(){return true}else foo()}",
        "function f(){if(e1){function goo(){return true}}else foo()}",
    );

    expect("if(e1){function goo(){return true}}", "if(e1){function goo(){return true}}");

    expect("if(e1)function goo(){return true}", "if(e1){function goo(){return true}}");
}

#[test]
#[ignore]
fn test_exponents() {
    // expectNumber("1", 1);
    // expectNumber("10", 10);
    // expectNumber("100", 100);
    // expectNumber("1E3", 1000);
    // expectNumber("1E4", 10000);
    // expectNumber("1E5", 100000);
    // expectNumber("1E18", 1000000000000000000d);
    // expectNumber("1E5", 100000.0);
    // expectNumber("100000.1", 100000.1);

    // expectNumber("1E-6", 0.000001);
    // expectNumber("0x38d7ea4c68001", 0x38d7ea4c68001p0d);
    // expectNumber("0x7fffffffffffffff", 0x7fffffffffffffffp0d);

    // expectNumber(".01", 0.01);
    // expectNumber("1.01", 1.01);
}

#[test]
#[ignore]
fn test_bigger_than_max_long_numeric_literals() {
    // Since ECMAScript implements IEEE 754 "round to nearest, ties to even",
    // any literal in the range [0x7ffffffffffffe00,0x8000000000000400] will
    // round to the same value, namely 2^63. The fact that we print this as
    // 2^63-1 doesn't matter, since it must be rounded back to 2^63 at runtime.
    // See:
    //   http://www.ecma-international.org/ecma-262/5.1/#sec-8.5
    expect("9223372036854775808", "0x7fffffffffffffff");
    expect("0x8000000000000000", "0x7fffffffffffffff");
    expect(
        "0b1000000000000000000000000000000000000000000000000000000000000000",
        "0x7fffffffffffffff",
    );
    expect("0o1000000000000000000000", "0x7fffffffffffffff");
}

#[test]
#[ignore]
fn test_direct_eval() {
    expect("eval('1');", "eval(\"1\")");
}

#[test]
#[ignore]
fn test_indirect_eval() {
    // Node n = parse("eval('1');");
    // expectNode("eval(\"1\")", n);
    // n.getFirstFirstChild().getFirstChild().putBooleanProp(Node.DIRECT_EVAL, false);
    // expectNode("(0,eval)(\"1\")", n);
}

#[test]
#[ignore]
fn free_call_tagged_template() {
    // Node n = parse("a.b`xyz`");
    // Node call = n.getFirstFirstChild();
    // assertThat(call.isTaggedTemplateLit()).isTrue();
    // call.putBooleanProp(Node.FREE_CALL, true);
    // expectNode("(0,a.b)`xyz`", n);
}

#[test]
#[ignore]
fn free_call_opt_chain() {
    // Node n = parse("(a?.b)()");
    // Node call = n.getFirstFirstChild();
    // assertThat(call.isCall()).isTrue();
    // call.putBooleanProp(Node.FREE_CALL, true);
    // expectNode("(0,a?.b)()", n);
}

#[test]
#[ignore]
fn free_call_opt_chain_opt_chain_call() {
    // Node n = parse("(a?.b)?.()");
    // Node call = n.getFirstFirstChild();
    // assertThat(call.isOptChainCall()).isTrue();
    // call.putBooleanProp(Node.FREE_CALL, true);
    // expectNode("(0,a?.b)?.()", n);
}

#[test]
#[ignore]
fn opt_chain_callee_for_new_requires_parentheses() {
    expect_same("new (a?.b)");
}

#[test]
#[ignore]
fn test_free_call1() {
    expect("foo(a);", "foo(a)");
    expect("x.foo(a);", "x.foo(a)");
}

#[test]
#[ignore]
fn test_free_call2() {
    // Node n = parse("foo(a);");
    // expectNode("foo(a)", n);
    // Node call = n.getFirstFirstChild();
    // assertThat(call.isCall()).isTrue();
    // call.putBooleanProp(Node.FREE_CALL, true);
    // expectNode("foo(a)", n);
}

#[test]
#[ignore]
fn test_free_call3() {
    // Node n = parse("x.foo(a);");
    // expectNode("x.foo(a)", n);
    // Node call = n.getFirstFirstChild();
    // assertThat(call.isCall()).isTrue();
    // call.putBooleanProp(Node.FREE_CALL, true);
    // expectNode("(0,x.foo)(a)", n);
}

#[test]
#[ignore]
fn test_print_script() {
    // Verify that SCRIPT nodes not marked as synthetic are printed as
    // blocks.
    // Node ast =
    // new Node(
    // Token.SCRIPT,
    // new Node(Token.EXPR_RESULT, Node.newString("f")),
    // new Node(Token.EXPR_RESULT, Node.newString("g")));
    // String result = new CodePrinter.Builder(ast).setPrettyPrint(true).build();
    // assertThat(result).isEqualTo("\"f\";\n\"g\";\n");
}

#[test]
#[ignore]
fn test_object_lit() {
    expect("({x:1})", "({x:1})");
    expect("var x=({x:1})", "var x={x:1}");
    expect("var x={'x':1}", "var x={\"x\":1}");
    expect("var x={1:1}", "var x={1:1}");
    expect("({},42)+0", "({},42)+0");
}

#[test]
#[ignore]
fn test_object_lit2() {
    expect("var x={1:1}", "var x={1:1}");
    expect("var x={'1':1}", "var x={1:1}");
    expect("var x={'1.0':1}", "var x={\"1.0\":1}");
    expect("var x={1.5:1}", "var x={\"1.5\":1}");
}

#[test]
#[ignore]
fn test_object_lit3() {
    expect("var x={3E9:1}", "var x={3E9:1}");
    expect(
        "var x={'3000000000':1}", // More than 31 bits
        "var x={3E9:1}",
    );
    expect("var x={'3000000001':1}", "var x={3000000001:1}");
    expect(
        "var x={'6000000001':1}", // More than 32 bits
        "var x={6000000001:1}",
    );
    expect(
        "var x={\"12345678901234567\":1}", // More than 53 bits
        "var x={\"12345678901234567\":1}",
    );
}

#[test]
#[ignore]
fn test_object_lit4() {
    // More than 128 bits.
    expect(
        "var x={\"123456789012345671234567890123456712345678901234567\":1}",
        "var x={\"123456789012345671234567890123456712345678901234567\":1}",
    );
}

#[test]
#[ignore]
fn test_extended_object_lit() {
    expect_same("var a={b}");
    expect_same("var a={b,c}");
    expect_same("var a={b,c:d,e}");
    expect_same("var a={b,c(){},d,e:f}");
}

#[test]
#[ignore]
fn test_computed_properties() {
    expect_same("var a={[b]:c}");
    expect_same("var a={[b+3]:c}");

    expect_same("var a={[b](){}}");
    expect_same("var a={[b](){alert(foo)}}");
    expect_same("var a={*[b](){yield\"foo\"}}");
    expect_same("var a={[b]:()=>c}");

    expect_same("var a={get [b](){return null}}");
    expect_same("var a={set [b](val){window.b=val}}");
}

#[test]
#[ignore]
fn test_computed_properties_class_methods() {
    expect_same("class C{[m](){}}");

    expect_same("class C{[\"foo\"+bar](){alert(1)}}");
}

#[test]
#[ignore]
fn test_getter() {
    expect("var x = {}", "var x={}");
    expect("var x = {get a() {return 1}}", "var x={get a(){return 1}}");
    expect("var x = {get a() {}, get b(){}}", "var x={get a(){},get b(){}}");

    expect("var x = {get 'a'() {return 1}}", "var x={get \"a\"(){return 1}}");

    expect("var x = {get 1() {return 1}}", "var x={get 1(){return 1}}");

    expect("var x = {get \"()\"() {return 1}}", "var x={get \"()\"(){return 1}}");

    expect_same("var x={get function(){return 1}}");
}

#[test]
#[ignore]
fn test_getter_in_es3() {
    // Getters and setters and not supported in ES3 but if someone sets the
    // the ES3 output mode on an AST containing them we still produce them.

    // Node getter = Node.newString(Token.GETTER_DEF, "f");
    // getter.addChildToBack(NodeUtil.emptyFunction());
    // expectNode("({get f(){}})", IR.exprResult(IR.objectlit(getter)));
}

#[test]
#[ignore]
fn test_setter() {
    expect("var x = {}", "var x={}");
    expect("var x = {set a(y) {return 1}}", "var x={set a(y){return 1}}");

    expect("var x = {get 'a'() {return 1}}", "var x={get \"a\"(){return 1}}");

    expect("var x = {set 1(y) {return 1}}", "var x={set 1(y){return 1}}");

    expect("var x = {set \"(x)\"(y) {return 1}}", "var x={set \"(x)\"(y){return 1}}");

    expect_same("var x={set function(x){}}");
}

#[test]
#[ignore]
fn test_setter_in_es3() {
    // Getters and setters and not supported in ES3 but if someone sets the
    // the ES3 output mode on an AST containing them we still produce them.

    // Node getter = Node.newString(Token.SETTER_DEF, "f");
    // getter.addChildToBack(IR.function(IR.name(""), IR.paramList(IR.name("a")), IR.block()));
    // expectNode("({set f(a){}})", IR.exprResult(IR.objectlit(getter)));
}

#[test]
#[ignore]
fn test_neg_no_collapse() {
    expect("var x = - - 2;", "var x=- -2");
    expect("var x = - (2);", "var x=-2");
}

#[test]
#[ignore]
fn test_strict() {
    // String result =
    // defaultBuilder(parse("var x", [> typeChecked= <] true)).setTagAsStrict(true).build();
    // assertThat(result).isEqualTo("'use strict';var x");
}

#[test]
#[ignore]
fn test_strict_pretty() {
    // String result =
    // defaultBuilder(parse("var x", [> typeChecked= <] true))
    // .setTagAsStrict(true)
    // .setPrettyPrint(true)
    // .build();
    // assertThat(result).isEqualTo("'use strict';\nvar x;\n");
}

#[test]
#[ignore]
fn test_ijs() {
    // String result =
    // defaultBuilder(parse("var x", [> typeChecked= <] true)).setTagAsTypeSummary(true).build();
    // assertThat(result).isEqualTo("/** @fileoverview @typeSummary */\nvar x");
}

#[test]
#[ignore]
fn test_ijs_with_provide_already_provided() {
    expect_same("/** @provideAlreadyProvided */ \ngoog.provide(\"a.b.c\");\n");
}

#[test]
#[ignore]
fn test_array_literal() {
    expect("var x = [,];", "var x=[,]");
    expect("var x = [,,];", "var x=[,,]");
    expect("var x = [,s,,];", "var x=[,s,,]");
    expect("var x = [,s];", "var x=[,s]");
    expect("var x = [s,];", "var x=[s]");
}

#[test]
#[ignore]
fn test_zero() {
    expect("var x ='\\0';", "var x=\"\\x00\"");
    expect("var x ='\\x00';", "var x=\"\\x00\"");
    expect("var x ='\\u0000';", "var x=\"\\x00\"");
    expect("var x ='\\u00003';", "var x=\"\\x003\"");
}

#[test]
#[ignore]
fn test_octal_in_string() {
    expect("var x ='\\0';", "var x=\"\\x00\"");
    expect("var x ='\\07';", "var x=\"\\u0007\"");

    // Octal 12 = Hex 0A = \n
    expect("var x ='\\012';", "var x=\"\\n\"");

    // Octal 13 = Hex 0B = \v
    expect("var x ='\\013';", "var x=\"\\v\"");

    // Octal 34 = Hex 1C
    expect("var x ='\\034';", "var x=\"\\u001c\"");

    // 8 and 9 are not octal digits
    expect("var x ='\\08';", "var x=\"\\x008\"");
    expect("var x ='\\09';", "var x=\"\\x009\"");

    // Only the first two digits are part of the octal literal.
    expect("var x ='\\01234';", "var x=\"\\n34\"");
}

#[test]
#[ignore]
fn test_octal_in_string_no_leading_zero() {
    expect("var x ='\\7';", "var x=\"\\u0007\"");

    // Octal 12 = Hex 0A = \n
    expect("var x ='\\12';", "var x=\"\\n\"");

    // Octal 13 = Hex 0B = \v.
    expect("var x ='\\13';", "var x=\"\\v\"");

    // Octal 34 = Hex 1C
    expect("var x ='\\34';", "var x=\"\\u001c\"");

    // Octal 240 = Hex A0
    expect("var x ='\\240';", "var x=\"\\u00a0\"");

    // Only the first three digits are part of the octal literal.
    expect("var x ='\\2400';", "var x=\"\\u00a00\"");

    // Only the first two digits are part of the octal literal because '8'
    // is not an octal digit.
    // Octal 67 = Hex 37 = "7"
    expect("var x ='\\6789';", "var x=\"789\"");

    // 8 and 9 are not octal digits. '\' is ignored and the digit
    // is just a regular character.
    expect("var x ='\\8';", "var x=\"8\"");
    expect("var x ='\\9';", "var x=\"9\"");

    // Only the first three digits are part of the octal literal.
    // Octal 123 = Hex 53 = "S"
    expect("var x ='\\1234';", "var x=\"S4\"");
}

#[test]
#[ignore]
fn test_unicode() {
    expect("var x ='\\x0f';", "var x=\"\\u000f\"");
    expect("var x ='\\x68';", "var x=\"h\"");
    expect("var x ='\\x7f';", "var x=\"\\u007f\"");
}

// Separate from test_numeric_keys() so we can set allowWarnings.
#[test]
#[ignore]
fn test_octal_numeric_key() {
    expect("var x = {010: 1};", "var x={8:1}");
}

#[test]
#[ignore]
fn test_numeric_keys() {
    expect("var x = {'010': 1};", "var x={\"010\":1}");

    expect("var x = {0x10: 1};", "var x={16:1}");
    expect("var x = {'0x10': 1};", "var x={\"0x10\":1}");

    // I was surprised at this result too.
    expect("var x = {.2: 1};", "var x={\"0.2\":1}");
    expect("var x = {'.2': 1};", "var x={\".2\":1}");

    expect("var x = {0.2: 1};", "var x={\"0.2\":1}");
    expect("var x = {'0.2': 1};", "var x={\"0.2\":1}");
}

#[test]
#[ignore]
fn test_issue582() {
    expect("var x = -0.0;", "var x=-0");
}

#[test]
#[ignore]
fn test_issue942() {
    expect("var x = {0: 1};", "var x={0:1}");
}

#[test]
#[ignore]
fn test_issue601() {
    expect("'\\v' == 'v'", "\"\\v\"==\"v\"");
    expect("'\\u000B' == '\\v'", "\"\\v\"==\"\\v\"");
    expect("'\\x0B' == '\\v'", "\"\\v\"==\"\\v\"");
}

#[test]
#[ignore]
fn test_issue620() {
    expect("alert(/ / / / /);", "alert(/ // / /)");
    expect("alert(/ // / /);", "alert(/ // / /)");
}

#[test]
#[ignore]
fn test_issue5746867() {
    expect("var a = { '$\\\\' : 5 };", "var a={\"$\\\\\":5}");
}

#[test]
#[ignore]
fn test_comma_spacing() {
    expect("var a = (b = 5, c = 5);", "var a=(b=5,c=5)");
    expect("var a = (b = 5, c = 5);", "var a = (b = 5, c = 5);\n");
}

#[test]
#[ignore]
fn test_many_commas() {
    // int numCommas = 10000;
    // List<String> numbers = new ArrayList<>();
    // numbers.add("0");
    // numbers.add("1");
    // Node current = new Node(Token.COMMA, Node.newNumber(0), Node.newNumber(1));
    // for (int i = 2; i < numCommas; i++) {
    // current = new Node(Token.COMMA, current);

    // // 1000 is printed as 1E3, and screws up our test.
    // int num = i % 1000;
    // numbers.add(String.valueOf(num));
    // current.addChildToBack(Node.newNumber(num));
    // }

    // String expected = Joiner.on(",").join(numbers);
    // String actual = printNode(current).replace("\n", "");
    // assertThat(actual).isEqualTo(expected);
}

#[test]
#[ignore]
fn test_many_adds() {
    // int numAdds = 10000;
    // List<String> numbers = new ArrayList<>();
    // numbers.add("0");
    // numbers.add("1");
    // Node current = new Node(Token.ADD, Node.newNumber(0), Node.newNumber(1));
    // for (int i = 2; i < numAdds; i++) {
    // current = new Node(Token.ADD, current);

    // // 1000 is printed as 1E3, and screws up our test.
    // int num = i % 1000;
    // numbers.add(String.valueOf(num));
    // current.addChildToBack(Node.newNumber(num));
    // }

    // String expected = Joiner.on("+").join(numbers);
    // String actual = printNode(current).replace("\n", "");
    // assertThat(actual).isEqualTo(expected);
}

#[test]
#[ignore]
fn test_minus_negative_zero() {
    // Negative zero is weird, because we have to be able to distinguish
    // it from positive zero (there are some subtle differences in behavior).
    expect("x- -0", "x- -0");
}

#[test]
#[ignore]
fn test_string_escape_sequences() {
    // From the SingleEscapeCharacter grammar production.
    expect_same("var x=\"\\b\"");
    expect_same("var x=\"\\f\"");
    expect_same("var x=\"\\n\"");
    expect_same("var x=\"\\r\"");
    expect_same("var x=\"\\t\"");
    expect_same("var x=\"\\v\"");
    expect("var x=\"\\\"\"", "var x='\"'");
    expect("var x=\"\\\'\"", "var x=\"'\"");

    // From the LineTerminator grammar
    expect("var x=\"\\u000A\"", "var x=\"\\n\"");
    expect("var x=\"\\u000D\"", "var x=\"\\r\"");
    expect_same("var x=\"\\u2028\"");
    expect_same("var x=\"\\u2029\"");

    // Now with regular expressions.
    expect_same("var x=/\\b/");
    expect_same("var x=/\\f/");
    expect_same("var x=/\\n/");
    expect_same("var x=/\\r/");
    expect_same("var x=/\\t/");
    expect_same("var x=/\\v/");
    expect_same("var x=/\\u000A/");
    expect_same("var x=/\\u000D/");
    expect_same("var x=/\\u2028/");
    expect_same("var x=/\\u2029/");
}

#[test]
#[ignore]
fn test_regexp_escape() {
    expect_same("/\\bword\\b/");
    expect_same("/Java\\BScript/");
    expect_same("/\\ca/");
    expect_same("/\\cb/");
    expect_same("/\\cc/");
    expect_same("/\\cA/");
    expect_same("/\\cB/");
    expect_same("/\\cC/");
    expect_same("/\\d/");
    expect_same("/\\D/");
    expect_same("/\\0/");
    expect_same("/\\\\/");
    expect_same("/(.)\\1/");
    expect_same("/\\x0B/"); // Don't print this as \v (as is done in strings)
}

#[test]
#[ignore]
fn test_regexp_unnecessary_escape() {
    expect("/\\a/", "/a/");
    expect("/\\e/", "/e/");
    expect("/\\g/", "/g/");
    expect("/\\h/", "/h/");
    expect("/\\i/", "/i/");
    expect("/\\¡/", "/\\u00a1/");
}

#[test]
#[ignore]
fn test_keyword_properties1() {
    expect_same("x.foo=2");
    expect_same("x.function=2");

    expect_same("x.foo=2");
}

#[test]
#[ignore]
fn test_keyword_properties1a() {

    // Node nodes = parse("x.function=2");

    // expectNode("x[\"function\"]=2", nodes);
}

#[test]
#[ignore]
fn test_keyword_properties2() {
    expect_same("x={foo:2}");
    expect_same("x={function:2}");

    expect_same("x={foo:2}");
}

#[test]
#[ignore]
fn test_keyword_properties2a() {

    // Node nodes = parse("x={function:2}");

    // expectNode("x={\"function\":2}", nodes);
}

#[test]
#[ignore]
fn test_issue1062() {
    expect_same("3*(4%3*5)");
}

#[test]
#[ignore]
fn test_preserve_type_annotations() {
    // preserveTypeAnnotations = true;
    expect_same("/** @type {foo} */ var bar");
    expect_same("function/** void */ f(/** string */ s,/** number */ n){}");

    // preserveTypeAnnotations = false;
    expect("/** @type {foo} */ var bar;", "var bar");
}

#[test]
#[ignore]
fn test_preserve_type_annotations2() {
    // preserveTypeAnnotations = true;

    expect_same("/** @const */ var ns={}");

    expect_same(lines!(
        "/**", //
        " * @const",
        " * @suppress {const,duplicate}",
        " */",
        "var ns={}"
    ));
}

#[test]
#[ignore]
fn test_default_parameters() {
    expect_same("function f(a=0){}");
    expect_same("function f(a,b=0){}");
    expect_same("function f(a=(1,2),b){}");
    expect_same("function f(a,b=(1,2)){}");
}

#[test]
#[ignore]
fn test_rest_parameters() {
    expect_same("function f(...args){}");
    expect_same("function f(first,...rest){}");
}

#[test]
#[ignore]
fn test_default_parameters_with_rest_parameters() {
    expect_same("function f(first=0,...args){}");
    expect_same("function f(first,second=0,...rest){}");
}

#[test]
#[ignore]
fn test_spread_expression() {
    expect_same("f(...args)");
    expect_same("f(...arrayOfArrays[0])");
    expect_same("f(...[1,2,3])");
    expect_same("f(...([1],[2]))");
}

#[test]
#[ignore]
fn test_class() {
    expect_same("class C{}");
    expect_same("(class C{})");
    expect_same("class C extends D{}");
    expect_same("class C{static member(){}}");
    expect_same("class C{member(){}get f(){}}");
    expect_same("var x=class C{}");
}

#[test]
#[ignore]
fn test_class_computed_properties() {
    expect_same("class C{[x](){}}");
    expect_same("class C{get [x](){}}");
    expect_same("class C{set [x](val){}}");

    expect_same("class C{static [x](){}}");
    expect_same("class C{static get [x](){}}");
    expect_same("class C{static set [x](val){}}");
}

#[test]
#[ignore]
fn test_class_pretty() {
    expect("class C{}", "class C {\n}\n");
    // expect(
    // "class C{member(){}get f(){}}",
    // "class C {\n" + "  member() {\n" + "  }\n" + "  get f() {\n" + "  }\n" + "}\n");
    expect("var x=class C{}", "var x = class C {\n};\n");
}

#[test]
#[ignore]
fn test_class_field() {
    expect_same(lines!(
        "class C {", //
        "  x;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  x=2;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  x=2;",
        "  y=3;",
        "}",
        ""
    ));
    expect(
        lines!(
            "class C {", //
            "  x=2",
            "  y=3",
            "}",
            ""
        ),
        lines!(
            "class C {", //
            "  x=2;",
            "  y=3;",
            "}",
            ""
        ),
    );
    expect(
        "class C {x=2;y=3}",
        lines!(
            "class C {", //
            "  x=2;",
            "  y=3;",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_class_field_check_state() {
    expect_same(lines!(
        "/** @interface */ ", //
        "class C {",
        "  x;",
        "}",
        ""
    ));
    expect_same(lines!(
        "/** @record */ ", //
        "class C {",
        "  x;",
        "}",
        ""
    ));
}

#[test]
#[ignore]
fn test_class_field_static() {
    expect_same(lines!(
        "class C {", //
        "  static x;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  static x=2;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  static x=2;",
        "  static y=3;",
        "}",
        ""
    ));
    expect_same(lines!(
        "/** @interface */ ", //
        "class C {",
        "  static x;",
        "}",
        ""
    ));
    expect_same(lines!(
        "/** @record */ ", //
        "class C {",
        "  static x;",
        "}",
        ""
    ));
}

#[test]
#[ignore]
fn test_computed_class_field_literal_string_number() {
    expect(
        "class C { 'str' = 2;}",
        lines!(
            "class C {", //
            "  [\"str\"]=2;",
            "}",
            ""
        ),
    );
    expect(
        "class C { 1 = 2;}",
        lines!(
            "class C {", //
            "  [1]=2;",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_computed_class_field() {
    expect_same(lines!(
        "class C {", //
        "  [x];",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  [x]=2;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  [x]=2;",
        "  y=3;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  [x]=2;",
        "  [y]=3;",
        "}",
        ""
    ));
}

#[test]
#[ignore]
fn test_computed_class_field_static() {
    expect_same(lines!(
        "class C {", //
        "  static [x];",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  static [x]=2;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  static [x]=2;",
        "  static y=3;",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {", //
        "  static [x]=2;",
        "  static [y]=3;",
        "}",
        ""
    ));
}

#[test]
#[ignore]
fn test_super() {
    expect_same("class C extends foo(){}");
    expect_same("class C extends m.foo(){}");
    expect_same("class C extends D{member(){super.foo()}}");
}

#[test]
#[ignore]
fn test_new_target() {
    expect_same("function f(){new.target}");
    expect("function f() {\nnew\n.\ntarget;\n}", "function f(){new.target}");
}

#[test]
#[ignore]
fn test_import_meta() {
    expect_same("import.meta");
    expect_same("import.meta.url");
    expect_same("console.log(import.meta.url)");
}

#[test]
#[ignore]
fn test_generator_yield() {
    expect_same("function*f(){yield 1}");
    expect_same("function*f(){yield}");
    expect_same("function*f(){yield 1?0:2}");
    expect_same("function*f(){yield 1,0}");
    expect_same("function*f(){1,yield 0}");
    expect_same("function*f(){yield(a=0)}");
    expect_same("function*f(){a=yield 0}");
    expect_same("function*f(){(yield 1)+(yield 1)}");
    // Parens required for evaluating arrow function expression i.e. `yield (() => expr)`
    expect_same("function*f(){yield(()=>({}))}");
}

#[test]
#[ignore]
fn test_generator_yield_pretty() {
    expect("function *f() {yield 1}", lines!("function* f() {", "  yield 1;", "}", ""));

    expect("function *f() {yield}", lines!("function* f() {", "  yield;", "}", ""));
}

#[test]
#[ignore]
fn test_member_generator_yield1() {
    expect_same("class C{*member(){(yield 1)+(yield 1)}}");
    expect_same("class C{*[0](){(yield 1)+(yield 1)}}");
    expect_same("var obj={*member(){(yield 1)+(yield 1)}}");
    expect_same("var obj={*[0](){(yield 1)+(yield 1)}}");
    expect_same("var obj={[0]:function*(){(yield 1)+(yield 1)}}");
}

#[test]
#[ignore]
fn test_arrow_function_zero_params() {
    expect_same("()=>1");
    expect("(()=>1)", "()=>1");
    expect_same("()=>{}");
    expect("(()=>a),b", "()=>a,b");
    expect("()=>(a=b)", "()=>a=b");
}

#[test]
#[ignore]
fn test_arrow_function_one_param() {
    expect_same("a=>b");
    expect_same("([a])=>b");
    expect_same("(...a)=>b");
    expect_same("(a=0)=>b");
    expect_same("(a=>b)(1)");
    expect_same("fn?.(a=>a)");
    expect("(a=>a)?.['length']", "(a=>a)?.[\"length\"]");
    expect_same("(a=>a)?.(1)");
    expect_same("(a=>a)?.length");
    expect_same("a=>a?.length");
    expect_same("var z={x:a=>1}");
    expect_same("[1,2].forEach(x=>y)");
}

#[test]
#[ignore]
fn test_arrow_function_many_params() {
    expect_same("(a,b)=>b");
}

#[test]
#[ignore]
fn test_arrow_function_body_edge_cases() {
    expect_same("()=>(a,b)");
    expect_same("()=>({a:1})");
    expect_same("()=>{return 1}");
}

#[test]
#[ignore]
fn test_async_function() {
    expect_same("async function f(){}");
    expect_same("let f=async function f(){}");
    expect_same("let f=async function(){}");
    // implicit semicolon prevents async being treated as a keyword
    expect("async\nfunction f(){}", "async;function f(){}");
    expect("let f=async\nfunction f(){}", "let f=async;function f(){}");
}

#[test]
#[ignore]
fn test_async_generator_function() {
    expect_same("async function*f(){}");
    expect_same("let f=async function*f(){}");
    expect_same("let f=async function*(){}");
    // implicit semicolon prevents async being treated as a keyword
    expect("async\nfunction*f(){}", "async;function*f(){}");
    expect("let f=async\nfunction*f(){}", "let f=async;function*f(){}");
}

#[test]
#[ignore]
fn test_async_arrow_function() {
    expect_same("async()=>1");
    expect("async (a) => 1", "async a=>1");

    // implicit semicolon prevents async being treated as a keyword
    expect("f=async\n()=>1", "f=async;()=>1");
}

#[test]
#[ignore]
fn test_async_method() {
    expect_same("o={async m(){}}");
    expect_same("o={async[a+b](){}}");
    expect_same("o={[0]:async function(){}}"); // (not technically a method)
    expect_same("class C{async m(){}}");
    expect_same("class C{async[a+b](){}}");
    expect_same("class C{static async m(){}}");
    expect_same("class C{static async[a+b](){}}");
}

#[test]
#[ignore]
fn test_async_generator_method() {
    expect_same("o={async *m(){}}");
    expect_same("o={async*[a+b](){}}");
    expect_same("o={[0]:async*function(){}}"); // (not technically a method)
    expect_same("class C{async *m(){}}");
    expect_same("class C{async*[a+b](){}}");
    expect_same("class C{static async *m(){}}");
    expect_same("class C{static async*[a+b](){}}");
}

#[test]
#[ignore]
fn test_await_expression() {
    expect_same("async function f(promise){return await promise}");
    expect_same("pwait=async function(promise){return await promise}");
    expect_same("class C{async pwait(promise){await promise}}");
    expect_same("o={async pwait(promise){await promise}}");
    expect_same("pwait=async promise=>await promise");
}

/** Regression test for b/235871063 - necessary parens dropped around awaited arrow function. */
#[test]
#[ignore]
fn test_parans_around_await_arrow_function() {
    // Parens required for evaluating arrow function expression i.e. `await (() => expr)`
    expect(
        "async function f(){return await (()=>new Promise((resolve)=>setTimeout(resolve,0)));}",
        "async function f(){return await (()=>new Promise(resolve=>setTimeout(resolve,0)))}",
    );
    // System.out.println("--------------");
    // Parens not required for evaluating new
    expect(
        "async function f(){return await new Promise((resolve)=>setTimeout(resolve,0));}",
        "async function f(){return await new Promise(resolve=>setTimeout(resolve,0))}",
    );
}

/**
 * Regression test for b/28633247 - necessary parens dropped around arrow functions.
 *
 * <p>Many of these cases use single param arrows because their PARAM_LIST parens should also be
 * dropped, which can make this harder to parse for humans.
 */
#[test]
#[ignore]
fn test_parens_around_arrow() {
    // Parens required for non-assignment binary operator
    expect("x||((_)=>true)", "x||(_=>true)");
    // Parens required for unary operator
    expect("void((e)=>e*5)", "void(e=>e*5)");
    // Parens not required for comma operator
    expect("((_) => true), ((_) => false)", "_=>true,_=>false");
    // Parens not required for right side of assignment operator
    // NOTE: An arrow function on the left side would be a parse error.
    expect("x = ((_) => _ + 1)", "x=_=>_+1");
    // Parens required for template tag
    expect("((_)=>\"\")`template`", "(_=>\"\")`template`");
    // Parens required to reference a property
    expect_same("((a,b,c)=>a+b+c).length");
    expect_same("((a,b,c)=>a+b+c)[\"length\"]");
    // Parens not required when evaluating property name.
    // (It doesn't make much sense to do it, though.)
    expect("x[((_)=>0)]", "x[_=>0]");
    // Parens required to call the arrow function immediately
    expect("((x)=>x*5)(10)", "(x=>x*5)(10)");
    // Parens not required for function call arguments
    expect("x(((_) => true), ((_) => false))", "x(_=>true,_=>false)");
    // Parens required for first operand to a conditional, but not the rest.
    expect("((x)=>1)?a:b", "(x=>1)?a:b");
    expect("x?((x)=>0):((x)=>1)", "x?x=>0:x=>1");
    expect("new ((x)=>x)", "new (x=>x)");
    expect_same("new C(x=>x)");
}

#[test]
#[ignore]
fn test_parens_around_arrow_fn_in_cast() {
    // preserveTypeAnnotations = false;
    expect("x(/** @type {?} */ (()=>{x}))", "x(()=>{x})");
    expect("x(/** @type {?} */ (()=>{x})())", "x((()=>{x})())");
    expect("x(/** @type {string} */ (/** @type {?} */ (()=>{x}))())", "x((()=>{x})())");

    // preserveTypeAnnotations = true;
    expect_same("x(/** @type {?} */ (()=>{x}))");
    expect_same("x(/** @type {?} */ (()=>{x})())");
    expect_same("x(/** @type {string} */ (/** @type {?} */ (()=>{x}))())");
}

#[test]
#[ignore]
fn test_parens_around_variable_declarator() {
    expect_same("var o=(test++,{one:1})");
    expect_same("({one}=(test++,{one:1}))");
    expect_same("[one]=(test++,[1])");

    expect_same("var {one}=(test++,{one:1})");
    expect_same("var [one]=(test++,[1])");
    expect("var {one}=/** @type {{one: number}} */(test++,{one:1})", "var {one}=(test++,{one:1})");
    expect("var [one]=/** @type {!Array<number>} */(test++,[1])", "var [one]=(test++,[1])");
}

#[test]
#[ignore]
fn test_parens_around_arrow_return_value() {
    expect_same("()=>({})");
    expect_same("()=>({a:1})");
    expect_same("()=>({a:1,b:2})");
    expect("()=>/** @type {Object} */({})", "()=>({})");
    expect("()=>/** @type {Object} */({a:1})", "()=>({a:1})");
    expect("()=>/** @type {Object} */({a:1,b:2})", "()=>({a:1,b:2})");
    expect("()=>/** @type {number} */(3)", "()=>3");
    expect("()=>/** @type {Object} */ ({}={})", "()=>({}={})");

    expect_same("()=>(1,2)");
    expect_same("()=>({},2)");
    expect_same("()=>(1,{})");
    expect("()=>/** @type {?} */(1,2)", "()=>(1,2)");
    expect("()=>/** @type {?} */({},2)", "()=>({},2)");
    expect("()=>/** @type {?} */(1,{})", "()=>(1,{})");

    // Test object literals more deeply nested
    expect_same("fn=()=>({})||3");
    expect_same("fn=()=>3||{}");
    expect_same("fn=()=>({}={})");
    expect_same("()=>function(){}"); // don't need parentheses around a function
    expect_same("for(var i=()=>({});;);");

    // preserveTypeAnnotations = true;
    expect_same("()=>/** @type {Object} */ ({})");
}

#[test]
#[ignore]
fn test_pretty_arrow_function() {
    expect(
        "if (x) {var f = ()=>{alert(1); alert(2)}}",
        lines!("if (x) {", "  var f = () => {", "    alert(1);", "    alert(2);", "  };", "}", ""),
    );
}

#[test]
#[ignore]
fn test_pretty_print_switch() {
    expect(
        "switch(something){case 0:alert(0);break;case 1:alert(1);break}",
        lines!(
            "switch(something) {",
            "  case 0:",
            "    alert(0);",
            "    break;",
            "  case 1:",
            "    alert(1);",
            "    break;",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_blocks_in_case_are_preserved() {
    // String js =
    // lines!(
    // "switch(something) {",
    // "  case 0:",
    // "    {",
    // "      const x = 1;",
    // "      break;",
    // "    }",
    // "  case 1:",
    // "    break;",
    // "  case 2:",
    // "    console.log(`case 2!`);",
    // "    {",
    // "      const x = 2;",
    // "      break;",
    // "    }",
    // "}",
    // "");
    // expect(js, js);
}

#[test]
#[ignore]
fn test_blocks_are_preserved() {
    // String js =
    // lines!(
    // "console.log(0);",
    // "{",
    // "  let x = 1;",
    // "  console.log(x);",
    // "}",
    // "console.log(x);",
    // "");
    // expect(js, js);
}

#[test]
#[ignore]
fn test_blocks_not_preserved() {
    expect("if (x) {};", "if(x);");
    expect("while (x) {};", "while(x);");
}

#[test]
#[ignore]
fn test_empty_class_static_block() {
    expect_same("class C {\n  static {\n  }\n}\n");
    expect("class C {\n  static {\n  }\n}\n", "class C{static{}}");
    expect_same("let a = class {\n  static {\n  }\n};\n");
}

#[test]
#[ignore]
fn test_class_static_block() {
    expect_same(lines!(
        "class C {",
        "  static field1=1;",
        "  static field2=2;",
        "  static {",
        "    let x = this.field1;",
        "    let y = this.field2;",
        "  }",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {",
        "  static {",
        "    this.field1 = 1;",
        "    this.field2 = 2;",
        "  }",
        "}",
        ""
    ));
    expect_same(lines!(
        "let a = class {",
        "  static field1=1;",
        "  static field2=2;",
        "  static {",
        "    let x = this.field1;",
        "    let y = this.field2;",
        "  }",
        "};",
        ""
    ));
    expect_same(lines!(
        "let a = class {",
        "  static {",
        "    this.field1 = 1;",
        "    this.field2 = 2;",
        "  }",
        "};",
        ""
    ));
}

#[test]
#[ignore]
fn test_multiple_class_static_blocks() {
    // empty
    expect_same("class C {\n  static {\n  }\n  static {\n  }\n}\n");
    expect_same("let a = class {\n  static {\n  }\n  static {\n  }\n};\n");
    // multiple fields
    expect_same(lines!(
        "class C {",
        "  static field1=1;",
        "  static field2=2;",
        "  static {",
        "    let x = this.field1;",
        "  }",
        "  static {",
        "    let y = this.field2;",
        "  }",
        "}",
        ""
    ));
    expect_same(lines!(
        "class C {",
        "  static {",
        "    this.field1 = 1;",
        "  }",
        "  static {",
        "    this.field2 = 2;",
        "  }",
        "}",
        ""
    ));
    expect_same(lines!(
        "let a = class {",
        "  static field1=1;",
        "  static field2=2;",
        "  static {",
        "    let x = this.field1;",
        "  }",
        "  static {",
        "    let y = this.field2;",
        "  }",
        "};",
        ""
    ));
    expect_same(lines!(
        "let a = class {",
        "  static {",
        "    this.field1 = 1;",
        "  }",
        "  static {",
        "    this.field2 = 2;",
        "  }",
        "};",
        ""
    ));
}

#[test]
#[ignore]
fn test_declarations() {
    expect_same("let x");
    expect_same("let x,y");
    expect_same("let x=1");
    expect_same("let x=1,y=2");
    expect_same("if(a){let x}");

    expect_same("const x=1");
    expect_same("const x=1,y=2");
    expect_same("if(a){const x=1}");

    expect_same("function f(){}");
    expect_same("if(a){function f(){}}");
    expect_same("if(a)(function(){})");

    expect_same("class f{}");
    expect_same("if(a){class f{}}");
    expect_same("if(a)(class{})");
}

#[test]
#[ignore]
fn test_imports() {
    // diagnosticsToIgnore = ImmutableList.of(MODULE_LOAD); // allow importing nonexistent modules
    expect_same("import x from\"./foo\"");
    expect_same("import\"./foo\"");
    expect_same("import x,{a as b}from\"./foo\"");
    expect_same("import{a as b,c as d}from\"./foo\"");
    expect_same("import x,{a}from\"./foo\"");
    expect_same("import{a,c}from\"./foo\"");
    expect_same("import x,*as f from\"./foo\"");
    expect_same("import*as f from\"./foo\"");
}

#[test]
#[ignore]
fn test_exports() {
    // export declarations
    // diagnosticsToIgnore = ImmutableList.of(MODULE_LOAD); // allow importing nonexistent modules
    expect_same("export var x=1");
    expect_same("export var x;export var y");
    expect_same("export let x=1");
    expect_same("export const x=1");
    expect_same("export function f(){}");
    expect_same("export class f{}");
    expect_same("export class f{}export class b{}");

    // export all from
    expect("export * from './a.b.c'", "export*from\"./a.b.c\"");

    // from
    expect_same("export{a}from\"./a.b.c\"");
    expect_same("export{a as x}from\"./a.b.c\"");
    expect_same("export{a,b}from\"./a.b.c\"");
    expect_same("export{a as x,b as y}from\"./a.b.c\"");
    expect_same("export{a}");
    expect_same("export{a as x}");

    expect_same("export{a,b}");
    expect_same("export{a as x,b as y}");

    // export default
    expect_same("export default x");
    expect_same("export default 1");
    expect_same("export default class Foo{}export function f(){}");
    expect_same("export function f(){}export default class Foo{}");
}

#[test]
#[ignore]
fn test_export_async_function() {
    expect_same("export async function f(){}");
}

#[test]
#[ignore]
fn test_template_literal() {
    // We need to use the raw string instead of the normalized string in template literals
    expect_same("`hello`");
    expect_same("`\\\\bhello`");
    expect_same("`hell\rlo`");
    expect_same("`hell\\rlo`");
    expect_same("`hell\r\nlo`");
    expect_same("`hell\\r\\nlo`");
    expect("`hello`\n'world'", "`hello`;\"world\"");
    expect("`hello`\n`world`", "`hello``world`");
    expect("var x=`TestA`\n`TemplateB`", "var x=`TestA``TemplateB`");
    expect_same("`hello``world`");

    expect_same("`hello${world}!`");
    expect_same("`hello${world} ${name}!`");

    expect_same("`hello${(function(){let x=3})()}`");
    expect_same("(function(){})()`${(function(){})()}`");
    expect_same("url`hello`");
    expect_same("url(`hello`)");
    expect_same("`\\u{2026}`");
    expect_same("`start\\u{2026}end`");
    expect_same("`\\u{1f42a}`");
    expect_same("`start\\u{1f42a}end`");
    expect_same("`\\u2026`");
    expect_same("`start\\u2026end`");
    expect_same("`\"`");
    expect_same("`'`");
    expect_same("`\\\"`");
    expect_same("`\\'`");
    expect_same("`\\``");

    expect_same("foo`\\unicode`");
    // b/114808380
    expect_same("String.raw`a\\ b`");

    // Nested substitutions.
    expect_same("`Hello ${x?`Alice`:`Bob`}?`");
    expect_same("`Hello ${x?`Alice ${y(`Kitten`)}`:`Bob`}?`");

    // Substitution without padding.
    expect_same("`Unbroken${x}string`");

    // Template strings terminate statements if needed.
    expect_same("let a;`a`");
}

#[test]
#[ignore]
fn test_multi_line_template_literal_preserves_interval_new_and_blanklines() {
    expect_same(lines!(
        "var y=`hello", // Line break (0 blank lines).
        "world",
        "", // Single blank line.
        "foo",
        "", // Multiple blank lines.
        "",
        "",
        "bar`"
    ));

    expect_same(lines!(
        "var y = `hello", // Line break (0 blank lines).
        "world",
        "", // Single blank line.
        "foo",
        "", // Multiple blank lines.
        "",
        "",
        "bar`;",
        ""
    ));
}

#[test]
#[ignore]
fn test_multi_line_template_literal_does_not_preserve_new_lines_in_substitutions() {
    expect(
        lines!(
            "var y=`Hello ${x", //
            "+",
            "z", //
            "}`"
        ),
        "var y=`Hello ${x+z}`",
    );

    expect(
        lines!(
            "var y=`Hello ${x", //
            "+",
            "z", //
            "}`"
        ),
        lines!(
            "var y = `Hello ${x + z}`;", //
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_multi_line_template_literal_not_indented_by_pretty_print() {
    // We intentionally put all the delimiter characters on the start of their own line to check
    // their indentation.
    expect(
        lines!(
            "function indentScope() {", //
            "  var y =",
            "`hello", // Open backtick.
            "world",
            "foo",
            "${", // Open substitution.
            "bing",
            "}", // Close substitution.
            "bar",
            "`;", // Close backtick.
            "}"
        ),
        lines!(
            "function indentScope() {", //
            "  var y = `hello",
            "world",
            "foo",
            "${bing}",
            "bar",
            "`;",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_multi_line_template_literal_broken_onto_last_line_is_not_collapsed() {
    // related to b/117613188

    // Given
    // Configure these so that the printer would otherwise attempt to reuse an existing newline.
    // CompilerOptions codePrinterOptions = new CompilerOptions();
    // codePrinterOptions.setLineLengthThreshold(30); // Must be big compared to the last line length.

    // String input =
    // lines!(
    // "`hello", //
    // "world", //
    // "foo", //
    // "bar`;");

    // // When
    // String actual =
    // new CodePrinter.Builder(parse(input))
    // .setCompilerOptions(codePrinterOptions)
    // .setPrettyPrint(false)
    // .build();

    // // Then
    // assertThat(actual)
    // .isEqualTo(
    // lines!(
    // "`hello", //
    // "world", //
    // "foo", //
    // "bar`"));
}

#[test]
#[ignore]
fn test_es6_goog_module() {
    // String code =
    // lines!(
    // "goog.module('foo.bar');",
    // "const STR = '3';",
    // "function fn() {",
    // "  alert(STR);",
    // "}",
    // "exports.fn = fn;");
    // String expectedCode =
    // lines!(
    // "goog.module('foo.bar');",
    // "var module$exports$foo$bar = {};",
    // "const STR = '3';",
    // "function fn() {",
    // "  alert(STR);",
    // "}",
    // "exports.fn = fn;\n");

    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.setClosurePass(true);
    // compilerOptions.setPreserveDetailedSourceInfo(true);
    // compilerOptions.setContinueAfterErrors(true);
    // Compiler compiler = new Compiler();
    // compiler.disableThreads();
    // checkWithOriginalName(code, expectedCode, compilerOptions);
}

#[test]
#[ignore]
fn test_es6_arrow_function_sets_original_name_for_this() {
    // String code = "(x)=>{this.foo[0](3);}";
    // String expectedCode =
    // ""
    // + "var $jscomp$this = this;\n" // TODO(tomnguyen): Avoid printing this line.
    // + "(function(x) {\n" // TODO(tomnguyen): This should print as an => function.
    // + "  this.foo[0](3);\n"
    // + "});\n";
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set
    // checkWithOriginalName(code, expectedCode, compilerOptions);
}

#[test]
#[ignore]
fn test_es6_arrow_function_sets_original_name_for_arguments() {
    // With original names in output set, the end result is not correct code, but the "this" is
    // not rewritten.
    // String code = "(x)=>{arguments[0]();}";
    // String expectedCode =
    // ""
    // + "var $jscomp$arguments = arguments;\n"
    // + "(function(x) {\n"
    // + "  arguments[0]();\n"
    // + "});\n";
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set
    // checkWithOriginalName(code, expectedCode, compilerOptions);
}

#[test]
#[ignore]
fn test_es6_new_target_bare() {
    expect_same("class C{constructor(){new.target.prototype}}");
}

#[test]
#[ignore]
fn test_es6_new_target_prototype() {
    expect_same(
        "class C{constructor(){var callable=Object.setPrototypeOf(obj,new.target.prototype)}}",
    );
}

#[test]
#[ignore]
fn test_es6_new_target_conditional() {
    expect(
        lines!("function f() {", "  if (!new.target) throw 'Must be called with new!';", "}"),
        "function f(){if(!new.target)throw\"Must be called with new!\";}",
    );
}

#[test]
#[ignore]
fn test_goog_scope() {
    // TODO(mknichel): Function declarations need to be rewritten to match the original source
    // instead of being assigned to a local variable with duplicate JS Doc.
    // String code =
    // ""
    // + "goog.provide('foo.bar');\n"
    // + "goog.require('baz.qux.Quux');\n"
    // + "goog.require('foo.ScopedType');\n"
    // + "\n"
    // + "goog.scope(function() {\n"
    // + "var Quux = baz.qux.Quux;\n"
    // + "var ScopedType = foo.ScopedType;\n"
    // + "\n"
    // + "var STR = '3';\n"
    // + "/** @param {ScopedType} obj */\n"
    // + "function fn(obj) {\n"
    // + "  alert(STR);\n"
    // + "  alert(Quux.someProperty);\n"
    // + "}\n"
    // + "}); // goog.scope\n";
    // String expectedCode =
    // ""
    // + "goog.provide('foo.bar');\n"
    // + "goog.require('baz.qux.Quux');\n"
    // + "goog.require('foo.ScopedType');\n"
    // + "/**\n"
    // + " * @param {ScopedType} obj\n"
    // + " */\n"
    // + "var $jscomp$scope$3556498$1$fn = /**\n"
    // + " * @param {ScopedType} obj\n"
    // + " */\n"
    // + "function(obj) {\n"
    // + "  alert(STR);\n"
    // + "  alert(Quux.someProperty);\n"
    // + "};\n"
    // + "var $jscomp$scope$3556498$0$STR = '3';\n";

    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.setChecksOnly(true);
    // compilerOptions.setClosurePass(true);
    // compilerOptions.setPreserveDetailedSourceInfo(true);
    // compilerOptions.setCheckTypes(true);
    // compilerOptions.setContinueAfterErrors(true);
    // compilerOptions.setPreserveClosurePrimitives(true);
    // Compiler compiler = new Compiler();
    // compiler.disableThreads();
    // compiler.compile(
    // ImmutableList.<SourceFile>of(), // Externs
    // ImmutableList.of(SourceFile.fromCode("test", code)),
    // compilerOptions);
    // Node node = compiler.getRoot().getLastChild().getFirstChild();

    // CompilerOptions codePrinterOptions = new CompilerOptions();
    // codePrinterOptions.setPreferSingleQuotes(true);
    // codePrinterOptions.setLineLengthThreshold(80);
    // codePrinterOptions.setPreserveTypeAnnotations(true);
    // codePrinterOptions.setUseOriginalNamesInOutput(true);
    // assertThat(
    // new CodePrinter.Builder(node)
    // .setCompilerOptions(codePrinterOptions)
    // .setPrettyPrint(true)
    // .setLineBreak(true)
    // .build())
    // .isEqualTo(expectedCode);
}

#[test]
#[ignore]
fn test_escape_dollar_in_template_literal_in_output() {
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set

    // checkWithOriginalName(
    // "let Foo; const x = `${Foo}`;", "let Foo;\nconst x = `${Foo}`;\n", compilerOptions);

    // checkWithOriginalName("const x = `\\${Foo}`;", "const x = `\\${Foo}`;\n", compilerOptions);

    // checkWithOriginalName(
    // "let Foo; const x = `${Foo}\\${Foo}`;",
    // "let Foo;\nconst x = `${Foo}\\${Foo}`;\n",
    // compilerOptions);

    // checkWithOriginalName(
    // "let Foo; const x = `\\${Foo}${Foo}`;",
    // "let Foo;\nconst x = `\\${Foo}${Foo}`;\n",
    // compilerOptions);
}

#[test]
#[ignore]
fn test_escape_dollar_in_template_literal_es5_output() {
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set

    // checkWithOriginalName(
    // "let Foo; const x = `${Foo}`;", "var Foo;\nvar x = '' + Foo;\n", compilerOptions);

    // checkWithOriginalName("const x = `\\${Foo}`;", "var x = '${Foo}';\n", compilerOptions);

    // checkWithOriginalName(
    // "let Foo; const x = `${Foo}\\${Foo}`;",
    // "var Foo;\nvar x = Foo + '${Foo}';\n",
    // compilerOptions);
    // checkWithOriginalName(
    // "let Foo; const x = `\\${Foo}${Foo}`;",
    // "var Foo;\nvar x = '${Foo}' + Foo;\n",
    // compilerOptions);
}

#[test]
#[ignore]
fn test_do_not_escape_dollar_in_regex() {
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set
    // checkWithOriginalName("var x = /\\$qux/;", "var x = /\\$qux/;\n", compilerOptions);
    // checkWithOriginalName("var x = /$qux/;", "var x = /$qux/;\n", compilerOptions);
}

#[test]
#[ignore]
fn test_do_not_escape_dollar_in_string_literal() {
    // String code = "var x = '\\$qux';";
    // String expectedCode = "var x = '$qux';\n";
    // CompilerOptions compilerOptions = new CompilerOptions();
    // compilerOptions.skipAllCompilerPasses();
    // compilerOptions.set
    // checkWithOriginalName(code, expectedCode, compilerOptions);
    // checkWithOriginalName("var x = '\\$qux';", "var x = '$qux';\n", compilerOptions);
    // checkWithOriginalName("var x = '$qux';", "var x = '$qux';\n", compilerOptions);
}

#[test]
#[ignore]
fn test_pretty_printer_if_else_if_added_block() {
    expect_same(lines!(
        "if (0) {",
        "  0;",
        "} else if (1) {",
        "  if (2) {",
        "    2;",
        "  }",
        "} else if (3) {",
        "  3;",
        "}",
        ""
    ));

    expect(
        "if(0)if(1)1;else 2;else 3;",
        lines!(
            "if (0) {",
            "  if (1) {",
            "    1;",
            "  } else {",
            "    2;",
            "  }",
            "} else {",
            "  3;",
            "}",
            ""
        ),
    );
}

#[test]
#[ignore]
fn test_non_js_doc_comments_printed_get_prop() {
    // preserveNonJSDocComments = true;
    // TODO(b/228156705): Fix comment printing properly for GETPROP.
    expect("a.// testComment\nb", "// testComment\na.b;\n");
}
