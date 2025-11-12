<Foo>aaa</Foo>;

// We should return `false` for `isSpaceBetween(openingElement, closingElement)`, but we currently return `true`
<Bar>b c</Bar>;

// We should return `false` for `isSpaceBetween(openingElement, closingElement)`, but we currently return `true`
<Qux>
  d
  e
</Qux>;
