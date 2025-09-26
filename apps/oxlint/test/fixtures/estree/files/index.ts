// @ts-nocheck
// dprint-ignore-file

// All `Identifier`s
let a = { x: y };

// No `ParenthesizedExpression`s in AST
const b = (x * ((('str' + ((123))))));

// TS syntax
type T = string;

// No `TSParenthesizedType`s in AST
type U = (((((string)) | ((number)))));
