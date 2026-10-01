use crate::{CompressOptions, default_options, test, test_options, test_same};

#[test]
fn test_minimize_if() {
    test(
        "function writeInteger(int) {
            if (int >= 0)
                if (int <= 0xffffffff) return this.u32(int);
                else if (int > -0x80000000) return this.n32(int);
        }",
        "function writeInteger(int) {
            if (int >= 0) {
                if (int <= 4294967295) return this.u32(int);
                if (int > -2147483648) return this.n32(int);
            }
        }",
    );

    test(
        "function bar() {
          if (!x) {
            return null;
          } else {
            return foo;
          }
        }",
        "function bar() {
          return x ? foo : null;
        }",
    );

    test(
        "function bar() {
          if (!x) {
            return null;
          } else if (y) {
            return foo;
          } else if (z) {
            return bar;
          }
        }",
        "function bar() {
          if (!x) return null;
          if (y) return foo;
          if (z) return bar;
        }",
    );

    test(
        "function f() {
          if (foo)
            if (bar) return X;
            else return Y;
          return Z;
        }",
        "function f() {
          return foo ? bar ? X : Y : Z;
        }",
    );

    test(
        "function _() {
            if (currentChar === '\\n')
                return pos + 1;
            else if (currentChar !== ' ' && currentChar !== '\\t')
                return pos + 1;
        }",
        "function _() {
            if (currentChar === '\\n' || currentChar !== ' ' && currentChar !== '\\t')
                return pos + 1;
        }",
    );

    test(
        "function f(){if(a)if(b)var x=1;else var y=2;return x+y}",
        "function f(){if(a)if(b)var x=1;else var y=2;return x+y}",
    );
    test(
        "function f(){if(a)if(b)if(c)var x=1;else var y=2;return x+y}",
        "function f(){if(a&&b)if(c)var x=1;else var y=2;return x+y}",
    );
    test(
        "function f(){if(!a){}else if(b)var x=1;else var y=2;return x+y}",
        "function f(){if(a)if(b)var x=1;else var y=2;return x+y}",
    );
    test("function f(){if(a){}else return b;}", "function f(){if(!a)return b;}");
    test("function f(){if(!a){}else return b;}", "function f(){if(a)return b;}");
    test("function f(){if(!a)b();else return c;}", "function f(){if(!a)b();else return c;}");
    test("function f(){if(a)return c;else b();}", "function f(){if(a)return c;b();}");
    test("function f(){if((a(),b)){}else c();}", "function f(){a(),b||c();}");
    test("function f(){if(a(),!(b||c)){}else d();}", "function f(){a(),!(b||c)||d();}");
}

#[test]
fn test_dangling_else_codegen() {
    test_same("if(a){if(b)var x=1}else var y=2;");
    test_same("if(a){if(b)var x=1;else if(c)var y=2}else var z=3;");
    test_same("if(a){for(;b;)if(c)var x=1}else var y=2;");
}

#[test]
fn test_nested_if_without_sequences() {
    let options = CompressOptions { sequences: false, ..default_options() };
    test_options(
        "function f(){if(a)if(b)return c;else d();}",
        "function f(){if(a){if(b)return c;d();}}",
        &options,
    );
    test_options(
        "function f(){if(a)if(b)return c;else {d();e();}}",
        "function f(){if(a){if(b)return c;d();e();}}",
        &options,
    );
    test_options(
        "function f(){if(a)if(b)return x;else {let x=d();e(x);}}",
        "function f(){if(a){if(b)return x;{let x=d();e(x);}}}",
        &options,
    );
    test_options(
        "function f(){if(a)if(b)return g;else {function g(){}e(g);}}",
        "function f(){if(a){if(b)return g;{function g(){}e(g);}}}",
        &options,
    );
}
