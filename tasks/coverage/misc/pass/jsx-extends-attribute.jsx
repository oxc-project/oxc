// JavaScript has no type parameters, so `<` and a name always start a JSX element,
// and `extends` after the name names an attribute, whatever follows it.
// In TypeScript, it can be the constraint of a type parameter. See `jsx-extends-attribute.tsx`.
<div extends />;
<div extends={foo} />;
<div extends>text</div>;
<div extends foo />;
<div extends foo></div>;
x = <T extends U>(x)</T>;
<div extends {...p} />;
<div extends:x="1" />;
<div extends-foo="x" />;
