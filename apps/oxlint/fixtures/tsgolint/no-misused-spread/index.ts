// Examples of incorrect code for no-misused-spread rule

// Spreading a non-iterable value in an array
const num = 42;
const arr = [...num]; // Runtime error: num is not iterable

// Spreading a Promise in an array
const promise = Promise.resolve([1, 2, 3]);
const arr2 = [...promise]; // Runtime error: Promise is not iterable

// Spreading non-object in object literal
const str = 'hello';
const obj = { ...str }; // Creates { '0': 'h', '1': 'e', ... } which might be unexpected