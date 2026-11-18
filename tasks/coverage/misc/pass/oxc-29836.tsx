// https://github.com/oxc-project/oxc/issues/29836
// `(// c)` and `(/* c */)` are JSX text, not comments. The speculative parses of
// `(z) : <div>(` as an arrow function return type and of `<T extends U>(` and
// `case (z):` as arrow functions read them as code, and must not leave the
// comments which that reading records in `Program::comments` behind.
x = y ? (z) : <div>(// c)</div>;
foo();
x = y ? (z) : <div>(/* c */)</div>;
x = <T extends U>(/* c */)</T>;

function f() {
  switch (y) {
    case (z): {
      return <div>(/* c */)</div>;
    }
  }
}
