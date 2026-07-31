import { importedResult, importedString } from "./values";
import { parseSync } from "oxc-parser";

declare const numberValue: number;
declare const bigintValue: bigint;
declare const anyValue: any;
declare const stringValue: string;
declare const unknownValue: unknown;
declare const mixedValue: number | string;

-numberValue;
-bigintValue;
-anyValue;
-stringValue;
// eslint-disable-next-line typescript/no-unsafe-unary-minus
-unknownValue;
-mixedValue;
-importedString;
-importedResult();
-parseSync("example.ts", "const value = 1;");

function negate<T extends number>(value: T) {
  return -value;
}
