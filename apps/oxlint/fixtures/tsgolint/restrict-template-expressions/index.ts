// Examples of incorrect code for restrict-template-expressions rule

declare const obj: object;
declare const sym: symbol;
declare const fn: () => void;
declare const arr: unknown[];

// Objects become "[object Object]"
const str1 = `Value: ${obj}`;

// Symbols might not be what you expect
const str2 = `Symbol: ${sym}`;

// Functions become their string representation
const str3 = `Function: ${fn}`;

// Arrays of unknown might be unsafe
const str4 = `Array: ${arr}`;
