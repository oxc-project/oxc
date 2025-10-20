// Examples of incorrect code for prefer-reduce-type-parameter rule

const numbers = [1, 2, 3];

// Casting the result
const sum = numbers.reduce((acc, val) => acc + val, 0) as number;

// Using type assertion on accumulator
const result = [1, 2, 3].reduce((acc: string[], curr) => {
  acc.push(curr.toString());
  return acc;
}, [] as string[]);
