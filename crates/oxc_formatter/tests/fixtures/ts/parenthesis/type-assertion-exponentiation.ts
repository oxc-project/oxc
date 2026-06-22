// `<T>x` as the base of `**` must stay parenthesized (`<T>x ** 2` is a syntax error).
a = (<T>x) ** 2;
// As the `**` exponent it does not need parentheses.
b = 2 ** (<T>x);
// Other operators are unaffected: `<<` keeps parens, `+` drops them.
c = (<T>x) << 2;
d = (<T>x) + 2;
// Right-associative nesting: the base `<T>x` must stay parenthesized.
e = (<T>x) ** 2 ** 3;
// Both operands are type assertions: only the base needs parentheses.
f = (<T>x) ** (<U>y);
