// https://github.com/oxc-project/oxc/issues/29899
// `<div extends foo` starts the type parameters of an arrow function, as in TypeScript,
// not a JSX element with attributes `extends` and `foo`.
<div extends foo />;
