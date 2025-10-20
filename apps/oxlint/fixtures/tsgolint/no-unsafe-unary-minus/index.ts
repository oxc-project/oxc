// Examples of incorrect code for no-unsafe-unary-minus rule

declare const value: any;
const result1 = -value; // unsafe on any

declare const str: string;
const result2 = -str; // unsafe on string

declare const bool: boolean;
const result3 = -bool; // unsafe on boolean

declare const obj: object;
const result4 = -obj; // unsafe on object
