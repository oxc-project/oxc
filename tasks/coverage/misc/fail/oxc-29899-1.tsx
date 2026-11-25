// https://github.com/oxc-project/oxc/issues/29899
// `<T extends U` starts the type parameters of an arrow function, as in TypeScript.
// So this is an arrow function with no `=>`, not a JSX element with attributes `extends` and `U`.
x = <T extends U>(x)</T>;
