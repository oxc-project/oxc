// https://github.com/oxc-project/oxc/issues/29836
// Speculative parses lex these JSX texts as code, but they contain no comments.
x = y ? (z) : <div>(// c)</div>;
foo();
x = y ? (z) : <div>(/* c */)</div>;

function f() {
  switch (y) {
    case (z): {
      return <div>(/* c */)</div>;
    }
  }
}
