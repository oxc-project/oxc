// CommentToken (Line)
/* CommentToken (Block) */

// BooleanToken
true;
false;

// NullToken
null;
let undefined!: void;

// NumericToken
123;
3.14;

// StringToken
("string");

// TemplateToken
tagged`template ${"literal"}`;

// RegularExpressionToken
/pattern/g;
// prettier-ignore
// Not a RegularExpressionToken
1 /not_a_regex/gu;

// IdentifierToken
let identifier = "value";

if (identifier) {
  for (let i = 0; i < 10; i++) {
    ((NaN + "") as any) ** 5;
  }
}

class MyClass extends Error {
  // PrivateIdentifierToken
  #private = "field";
  constructor() {
    super();
    this.#private;
    #private in this;
  }
}

// PunctuatorToken (operators and punctuation: =, +, -, (), {}, [], etc.)
(false, Infinity, eval)?.("use strict") satisfies MyClass;
[1, 2, 3];
{
  key: ("value");
}

type T = { [key: string]: number };
interface I extends T {
  x: T;
}

function JSX() {
  return <div>Hello World</div>;
}
