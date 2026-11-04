#!/usr/bin/env node
"use strict";
"directive with 'quote";
("ordinary string statement");

debugger;

const value = 1e21;
const tiny = 0.000000001;
const regex = /script/gi;
const bigint = 123n;
const template = `value: ${value}`;
const object = {
  value,
  __proto__,
  __proto__: value,
  get answer() { return 42; },
  set answer(next) { value = next; },
  async *[key](item) { yield item; },
  ...source,
};
const [first, , ...rest] = list;
const { value: renamed = 0, ...remaining } = object;
[first, ...rest] = list;
({ value: renamed = 0, ...remaining } = object);

label: for (let index = 0; index < 3; index++) {
  if (index === 1) continue label;
  if (index === 2) break label;
}
for (const item of list) item;
for (let async of list) async;
for (const key in object) object[key];
while (false) break;
do { value; } while (false);

if (value) {
  value;
} else if (tiny) {
  tiny;
} else value;

switch (value) {
  case 1:
    break;
  default:
    value;
}

try {
  throw regex;
} catch (error) {
  error;
} finally {
  value;
}
with (object) value;

class Example extends Base {
  static {}
  #private = 1;
  static #staticPrivate;
  get answer() { return this.#private; }
  set answer(next) { this.#private = next; }
  static get shared() { return this.#staticPrivate; }
  [key](item) { return (#private in item) && true; }
}

const member = (let)[0];
const chained = new Factory()(arg)?.field;
const dynamic = import("module", { with: { type: "json" } });
function rest(...args) { return args; }
if (value)
  if (tiny) value;
for ((let) of list) {}
for ((let)[0] of list) {}
import Default, * as Namespace from "module";
import { value as importedValue, "named" as importedNamed } from "module";
import data from "data" with { type: "json" };
export {};
export const exported = value;
export function exportedFunction() {}
export class Exported {}
export default class DefaultExport {}
export { member, chained, dynamic };
export * from "other";

const \u{61} = 1;

class PrivateFields {
  #field = \u{61};

  read() {
    return this.#field;
  }
}

const integerWithoutTrailingZero = 1234;
const fractionWithoutLeadingZeros = 0.5;
const fractionWithTooFewLeadingZerosForExponent = 0.001;
const exponentWithoutFraction = 1e21;
const exponentFoldedWhenItStaysShort = 1.5e21;
const exponentLeftUnfoldedWhenItWouldGrow = 1.5e-9;
