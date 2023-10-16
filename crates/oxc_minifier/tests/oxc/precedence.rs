use crate::test;

#[test]
fn comma() {
    test("1, 2, 3", "1,2,3;");
    test("1, a = b, 3", "1,a=b,3;");
    test("1, (2, 3), 4", "1,(2,3),4;");
}

#[test]
fn assignment() {
    test("a = b ? c : d", "a=b?c:d;");
    test("[a,b] = (1, 2)", "[a,b]=(1,2);");
    // `{a,b}` is a block, must wrap the whole expression to be an assignment expression
    test("({a,b} = (1, 2))", "({a,b}=(1,2));");
    test("a *= yield b", "a*=yield b;");
    test("a /= () => {}", "a/=()=>{};");
    test("a %= async () => {}", "a%=async()=>{};");
    test("a -= (1, 2)", "a-=(1,2);");
    test("a >>= b >>= c", "a>>=b>>=c;");
}

#[test]
fn r#yield() {
    test("function *foo() { yield }", "function*foo(){yield}");

    test("function *foo() { yield * a ? b : c }", "function*foo(){yield*a?b:c}");
    test("function *foo() { yield * yield * a }", "function*foo(){yield*yield*a}");
    test("function *foo() { yield * () => {} }", "function*foo(){yield*()=>{}}");
    test("function *foo() { yield * async () => {} }", "function*foo(){yield*async()=>{}}");

    test("function *foo() { yield a ? b : c }", "function*foo(){yield a?b:c}");
    test("function *foo() { yield yield a }", "function*foo(){yield yield a}");
    test("function *foo() { yield () => {} }", "function*foo(){yield ()=>{}}");
    test("function *foo() { yield async () => {} }", "function*foo(){yield async()=>{}}");

    test(
        "function *foo() { yield { a } = [ b ] = c ? b : d }",
        "function*foo(){yield {a}=[b]=c?b:d}",
    );
    // TODO: remove the extra space in `yield (a,b)`
    test("function *foo() { yield (a, b) }", "function*foo(){yield (a,b)}");
    test("function *foo() { yield a, b }", "function*foo(){yield a,b}");
}

#[test]
fn arrow() {
    test("x => a, b", "x=>a,b;");
    test("x => (a, b)", "x=>(a,b);");
    test("x => (a => b)", "x=>a=>b;");
    test("x => y => a, b", "x=>y=>a,b;");
    test("x => y => (a = b)", "x=>y=>a=b;");
    test("x => y => z => a = b, c", "x=>y=>z=>a=b,c;");
    test("x => y => z => a = (b, c)", "x=>y=>z=>a=(b,c);");
    test("x => ({} + 0)", "x=>({})+0;");
}

#[test]
fn conditional() {
    test("a ? b : c", "a?b:c;");
    test("a ? (b, c) : (d, e)", "a?(b,c):(d,e);");
    test("a ? b : c ? b : c", "a?b:c?b:c;");
    test("(a ? b : c) ? b : c", "a?b:c?b:c;");
    test("a, b ? c : d", "a,b?c:d;");
    test("(a, b) ? c : d", "(a,b)?c:d;");
    test("a = b ? c : d", "a=b?c:d;");
    test("(a = b) ? c : d", "(a=b)?c:d;");
}

#[test]
fn coalesce() {
    test("a ?? b", "a??b;");
    test("a ?? b ?? c ?? d", "a??b??c??d;");
    test("a ?? (b ?? (c ?? d))", "a??b??c??d;");
    test("(a ?? (b ?? (c ?? d)))", "a??b??c??d;");
    test("a, b ?? c", "a,b??c;");
    test("(a, b) ?? c", "(a,b)??c;");
    test("a, b ?? c, d", "a,b??c,d;");
    test("a, b ?? (c, d)", "a,b??(c,d);");
    test("a = b ?? c", "a=b??c;");
    test("a ?? (b = c)", "a??(b=c);");
    test("(a | b) ?? (c | d)", "a|b??c|d;");
}

#[test]
fn logical_or() {
    test("a || b || c", "a||b||c;");
    test("(a || (b || c)) || d", "a||b||c||d;");
    test("a || (b || (c || d))", "a||b||c||d;");
    test("a || b && c", "a||b&&c;");
    test("(a || b) && c", "(a||b)&&c;");
    test("a, b || c, d", "a,b||c,d;");
    test("(a, b) || (c, d)", "(a,b)||(c,d);");
    test("(a && b) || (c && d)", "a&&b||c&&d;");
    test("a && b || c && d", "a&&b||c&&d;");
}

#[test]
fn logical_and() {
    test("a && b && c", "a&&b&&c;");
    test("a && ((b && c) && d)", "a&&b&&c&&d;");
    test("((a && b) && c) && d", "a&&b&&c&&d;");
    test("(a || b) && (c || d)", "(a||b)&&(c||d);");
    test("a, b && c, d", "a,b&&c,d;");
    test("(a, b) && (c, d)", "(a,b)&&(c,d);");
    test("a || b && c || d", "a||b&&c||d;");
}

#[test]
fn bitwise_or() {
    test("a | b | c", "a|b|c;");
    test("(a | b) | c", "a|b|c;");
    test("a | (b | c)", "a|b|c;");
    test("a | b ^ c", "a|b^c;");
    test("a | (b ^ c)", "a|b^c;");
    test("a | (b && c)", "a|(b&&c);");
    test("a | b && c", "a|b&&c;");
    test("(a ^ b) | (c ^ d)", "a^b|c^d;");
    test("(a, b) | (c, d)", "(a,b)|(c,d);");
    test("a, b | c, d", "a,b|c,d;");
}

#[test]
fn bitwise_xor() {
    test("a ^ b ^ c", "a^b^c;");
    test("(a ^ b) ^ c", "a^b^c;");
    test("a ^ (b ^ c)", "a^b^c;");
    test("a | b & c", "a|b&c;");
    test("a | (b & c)", "a|b&c;");
    test("a | (b || c)", "a|(b||c);");
    test("a | b || c", "a|b||c;");
    test("(a, b) ^ (c, d)", "(a,b)^(c,d);");
    test("(a | b) ^ (c | d)", "(a|b)^(c|d);");
    test("a, b ^ c, d", "a,b^c,d;");
}

#[test]
fn bitwise_and() {
    test("a & b & c", "a&b&c;");
    test("((a & b) & c) & d", "a&b&c&d;");
    test("a & (b & (c & d))", "a&b&c&d;");
    test("a & b == c", "a&b==c;");
    test("a & (b == c)", "a&b==c;");
    test("a == b & c", "a==b&c;");
    test("(a == b) & c", "a==b&c;");
    test("a ^ b & c", "a^b&c;");
    test("(a ^ b) & c", "(a^b)&c;");
    test("(a, b) & (c, d)", "(a,b)&(c,d);");
    test("a, b & c, d", "a,b&c,d;");
}

#[test]
fn equality() {
    test("a == b != c === d !== e", "a==b!=c===d!==e;");
    test("a == (b != (c === (d !== e)))", "a==b!=c===d!==e;");
    test("(((a == b) != c) === d) !== e", "a==b!=c===d!==e;");
    test("a > b == c < d", "a>b==c<d;");
    test("(a > b) == (c < d)", "a>b==c<d;");
    test("a | b == c & d", "a|b==c&d;");
    test("(a | b) == (c & d)", "(a|b)==(c&d);");
    test("a, b == c , d", "a,b==c,d;");
    test("(a, b) == (c , d)", "(a,b)==(c,d);");
}
