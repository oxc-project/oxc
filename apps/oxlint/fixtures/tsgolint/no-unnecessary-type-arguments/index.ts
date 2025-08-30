// Examples of incorrect code for no-unnecessary-type-arguments rule

function identity<T = string>(arg: T): T {
  return arg;
}

// Unnecessary type argument - string is the default
const result = identity<string>('hello');

interface Container<T = number> {
  value: T;
}