// Examples of incorrect code for strict-boolean-expressions rule

// Non-boolean values in conditions
const str = 'hello';
if (str) { // string is not a boolean
  console.log('string');
}

const num = 42;
if (num) { // number is not a boolean
  console.log('number');
}

const obj = { foo: 'bar' };
if (obj) { // object is not a boolean
  console.log('object');
}

// Nullable booleans
declare const maybeBool: boolean | null;
if (maybeBool) { // nullable boolean
  console.log('maybe');
}

// Undefined checks
declare const maybeString: string | undefined;
if (maybeString) { // should explicitly check !== undefined
  console.log(maybeString);
}

// Logical operators with non-booleans
const result1 = str && num;
const result2 = obj || num;

// While loops with non-boolean
while (num) {
  console.log('loop');
  break;
}

// Do-while with non-boolean
do {
  console.log('do');
} while (str);

// For loop with non-boolean
for (let i = 0; i; i++) {
  console.log('for');
}

// Ternary with non-boolean
const ternary = str ? 'yes' : 'no';

// Logical NOT on non-boolean
const negated = !str;

// Mixed types in logical expressions
declare const mixed: string | number;
if (mixed) {
  console.log('mixed');
}

// any type (should allow or warn depending on config)
declare const anyValue: any;
if (anyValue) {
  console.log('any');
}
