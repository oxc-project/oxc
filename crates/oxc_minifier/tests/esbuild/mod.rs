use crate::expect;

#[test]
#[ignore]
fn number() {
    expect("x = 1e-100", "x=1e-100");
    expect("x = 1e-5", "x=1e-5");
    expect("x = 1e-4", "x=1e-4");
    expect("x = 1e-3", "x=.001");
    expect("x = 1e-2", "x=.01");
    expect("x = 1e-1", "x=.1");
    expect("x = 1e0", "x=1");
    expect("x = 1e1", "x=10");
    expect("x = 1e2", "x=100");
    expect("x = 1e3", "x=1e3");
    expect("x = 1e4", "x=1e4");
    expect("x = 1e100", "x=1e100");
}

#[test]
fn array() {
    expect("[]", "[]");
    expect("[,]", "[,]");
    expect("[,,]", "[,,]");
}

#[test]
fn splat() {
    expect("[...(a, b)]", "[...(a,b)]");
    expect("x(...(a, b))", "x(...(a,b))");
    expect("({...(a, b)})", "({...(a,b)})");
}

#[test]
fn call() {
    expect("x()()()", "x()()()");
    expect("x().y()[z]()", "x().y()[z]()");
    expect("(--x)();", "(--x)()");
    expect("(x--)();", "(x--)()");

    expect("eval(x)", "eval(x)");
    expect("eval?.(x)", "eval?.(x)");
    // expect("(eval)(x)", "eval(x)");
    // expect("(eval)?.(x)", "eval?.(x)");

    expect("eval(x, y)", "eval(x,y)");
    expect("eval?.(x, y)", "eval?.(x,y)");
    expect("(1, eval)(x)", "(1,eval)(x)");
    expect("(1, eval)?.(x)", "(1,eval)?.(x)");
    // expect("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // expect("(1 ? eval : 2)?.(x)", "eval?.(x)");

    expect("eval?.(x)", "eval?.(x)");
    expect("eval(x,y)", "eval(x,y)");
    expect("eval?.(x,y)", "eval?.(x,y)");
    expect("(1, eval)(x)", "(1,eval)(x)");
    expect("(1, eval)?.(x)", "(1,eval)?.(x)");
    // expect("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // expect("(1 ? eval : 2)?.(x)", "eval?.(x)");
}

#[test]
fn comma() {
    expect("1, 2, 3", "1,2,3");
    // expect("(1, 2), 3", "1,2,3");
    // expect("1, (2, 3)", "1,2,3");
    expect("a ? (b, c) : (d, e)", "a?(b,c):(d,e)");
    expect("let x = (a, b)", "let x=(a,b)");
    // expect("(x = a), b", "x=a,b");
    expect("x = (a, b)", "x=(a,b)");
    expect("x((1, 2))", "x((1,2))");
}

#[test]
fn function() {
    expect("function foo(a = (b, c), ...d) {}", "function foo(a=(b,c),...d){}");
    expect(
        "function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        "function foo({[1+2]:a=3}={[1+2]:3}){}",
    );
    expect(
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        "function foo([a=(1,2),...[b,...c]]=[1,[2,3]]){}",
    );
    expect("function foo([] = []) {}", "function foo([]=[]){}");
    expect("function foo([,] = [,]) {}", "function foo([,]=[,]){}");
    expect("function foo([,,] = [,,]) {}", "function foo([,,]=[,,]){}");
}

#[test]
fn whitespace() {
    expect("- -x", "- -x");
    expect("+ -x", "+-x");
    expect("- +x", "-+x");
    expect("+ +x", "+ +x");
    expect("- --x", "- --x");
    expect("+ --x", "+--x");
    expect("- ++x", "-++x");
    expect("+ ++x", "+ ++x");

    expect("- -x", "- -x");
    expect("+ -x", "+-x");
    expect("- +x", "-+x");
    expect("+ +x", "+ +x");
    expect("- --x", "- --x");
    expect("+ --x", "+--x");
    expect("- ++x", "-++x");
    expect("+ ++x", "+ ++x");

    expect("x - --y", "x- --y");
    expect("x + --y", "x+--y");
    expect("x - ++y", "x-++y");
    expect("x + ++y", "x+ ++y");

    expect("x-- > y", "x-- >y");
    expect("x < !--y", "x<! --y");
    // expect("x > !--y", "x>!--y");
    expect("!--y", "!--y");

    expect("1 + -0", "1+-0");
    expect("1 - -0", "1- -0");
    // expect("1 + -Infinity", "1+-Infinity");
    // expect("1 - -Infinity", "1- -Infinity");

    // expect("/x/ / /y/", "/x// /y/");
    expect("/x/ + Foo", "/x/+Foo");
    expect("/x/ instanceof Foo", "/x/ instanceof Foo");
    // expect("[x] instanceof Foo", "[x]instanceof Foo");

    expect("throw x", "throw x");
    expect("throw typeof x", "throw typeof x");
    expect("throw delete x", "throw delete x");
    expect("throw function(){}", "throw function(){}");

    expect("x in function(){}", "x in function(){}");
    expect("x instanceof function(){}", "x instanceof function(){}");
    expect("π in function(){}", "π in function(){}");
    expect("π instanceof function(){}", "π instanceof function(){}");

    expect("()=>({})", "()=>({})");
    // expect("()=>({}[1])", "()=>({})[1]");
    expect("()=>({}+0)", "()=>({}+0)");
    expect("()=>function(){}", "()=>function(){}");

    expect("(function(){})", "(function(){})");
    expect("(class{})", "(class{})");
    expect("({})", "({})");
}

#[test]
#[ignore]
fn infinity() {
    expect("x = Infinity", "x=1/0");
    expect("x = -Infinity", "x=-1/0");
    expect("x = (Infinity).toString", "x=(1/0).toString");
    expect("x = (-Infinity).toString", "x=(-1/0).toString");
    expect("x = Infinity ** 2", "x=(1/0) ** 2");
    expect("x = (-Infinity) ** 2", "x=(-1/0)**2");
    expect("x = Infinity * y", "x=1/0*y");
    expect("x = Infinity / y", "x=1/0/y");
    expect("x = y * Infinity", "x=y*(1/0)");
    expect("x = y / Infinity", "x=y/(1/0)");
    expect("throw Infinity", "throw 1/0");

    expect("x = Infinity", "x=1/0");
    expect("x = -Infinity", "x=-1/0");
    expect("x = (Infinity).toString", "x=(1/0).toString");
    expect("x = (-Infinity).toString", "x=(-1/0).toString");
    expect("x = Infinity ** 2", "x=(1/0)**2");
    expect("x = (-Infinity) ** 2", "x=(-1/0)**2");
    expect("x = Infinity * y", "x=1/0*y");
    expect("x = Infinity / y", "x=1/0/y");
    expect("x = y * Infinity", "x=y*(1/0)");
    expect("x = y / Infinity", "x=y/(1/0)");
    expect("throw Infinity", "throw 1/0");
}
