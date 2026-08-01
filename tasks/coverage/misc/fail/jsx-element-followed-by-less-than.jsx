// `<div />` is a complete JSX element, so the `<` that follows begins a second,
// adjacent JSX element. The JSX grammar technically allows a `JSXElement` as the
// left-hand side of a relational expression, so `<div /> < 5` could be read as
// `(<div />) < 5`. But Babel and TypeScript both reject any `<` directly after a
// JSX element as unwrapped adjacent JSX, and oxc follows them for consistency.
const a = <div /> < 5;
