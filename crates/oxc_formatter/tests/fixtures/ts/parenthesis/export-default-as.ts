// Parens are load-bearing: without them the leading `function`/`class` makes
// `export default` parse the right-hand side as a declaration, detaching the cast.
export default (function foo() {} as unknown as Foo);
