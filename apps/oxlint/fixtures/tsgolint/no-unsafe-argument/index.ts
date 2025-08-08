// Examples of incorrect code for no-unsafe-argument rule

declare const anyValue: any;

function takesString(str: string): void {
  console.log(str.length);
}

takesString(anyValue); // unsafe

declare function takesNumber(num: number): number;
const result = takesNumber(anyValue); // unsafe