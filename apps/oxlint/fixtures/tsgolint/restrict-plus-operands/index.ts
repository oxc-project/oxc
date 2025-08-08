// Examples of incorrect code for restrict-plus-operands rule

declare const num: number;
declare const str: string;
declare const bool: boolean;
declare const obj: object;

// Mixed types
const result1 = num + str; // number + string
const result2 = str + bool; // string + boolean
const result3 = num + bool; // number + boolean
const result4 = obj + str; // object + string