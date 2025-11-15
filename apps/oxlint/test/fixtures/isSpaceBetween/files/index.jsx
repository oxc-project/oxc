<Foo>aaa</Foo>;

// We should return `false` for `isSpaceBetween(openingElement, closingElement)`, but we currently return `true`
<Bar>b c</Bar>;

// We should return `false` for `isSpaceBetween(openingElement, closingElement)`, but we currently return `true`
// prettier-ignore
<Qux>d
e</Qux>;
