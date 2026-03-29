use crate::test;

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
}
