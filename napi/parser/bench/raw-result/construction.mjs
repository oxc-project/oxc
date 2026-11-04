// oxlint-disable no-console
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
import { root } from "./prepare.mjs";

const [variant, destination] = process.argv.slice(2);
assert.ok(variant && destination);
const { wrapResult } = await import(
  pathToFileURL(path.join(root, "target/raw-result", `${variant}.mjs`))
);
const inputs = Array.from({ length: 32267 }, () => ({
  program: { body: [] },
  module: {},
  comments: [],
  errors: [],
}));
for (let pass = 0; pass < 2; pass++) {
  for (let i = 0; i < 257; i++) wrapResult(inputs[i], null);
}
globalThis.gc();
const retained = [];
const start = performance.now();
for (const data of inputs) retained.push(wrapResult(data, null));
const elapsedMilliseconds = performance.now() - start;
let checksum = 0;
for (let i = 0; i < retained.length; i++) {
  assert.equal(retained[i].program, inputs[i].program);
  checksum += retained[i].program.body.length + retained[i].errors.length;
}
const report = {
  variant,
  node: process.version,
  v8: process.versions.v8,
  count: retained.length,
  elapsedMilliseconds,
  checksum,
};
fs.writeFileSync(destination, JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report));
process.exit(0);
