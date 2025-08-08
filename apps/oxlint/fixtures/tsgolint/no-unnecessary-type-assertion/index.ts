// Examples of incorrect code for no-unnecessary-type-assertion rule

const str: string = 'hello';
const redundant = str as string; // unnecessary, str is already string

function getString(): string {
  return 'hello';
}
const result = getString() as string; // unnecessary, getString() already returns string

const num = 42;
const alsoRedundant = num as 42; // unnecessary if TypeScript can infer literal type