// Examples of incorrect code for no-unsafe-assignment rule

declare const anyValue: any;

const str: string = anyValue; // unsafe assignment

let num: number;
num = anyValue; // unsafe assignment

const obj = {
  prop: anyValue as any, // unsafe assignment
};
