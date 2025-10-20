// Examples of incorrect code for no-unsafe-type-assertion rule

declare const value: unknown;

const str = value as any; // unsafe type assertion

const obj = value as any as string; // double assertion through any

function processValue(input: unknown) {
  const processed = input as any; // unsafe
  return processed.someProperty;
}
