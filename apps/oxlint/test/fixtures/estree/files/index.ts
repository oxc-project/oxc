// @ts-nocheck

// All `Identifier`s
let a = { x: y };

// No `ParenthesizedExpression`s in AST
// prettier-ignore
const b = (x * ((('str' + ((123))))));

// TS syntax
type T = string;

// No `TSParenthesizedType`s in AST
// prettier-ignore
type U = (((((string)) | ((number)))));
