// `extends` after the name of a JSX element names an attribute if `=`, `>` or `/` follows it,
// or if the name contains `-`, which the name of a type parameter can't.
// Otherwise it's the constraint of an arrow function's type parameter, as in TypeScript.
// See `misc/fail/oxc-29899-1.tsx`.
<div extends />;
<div extends={foo} />;
<div extends>text</div>;
<my-element extends class="a" />;
<my-element extends foo></my-element>;
