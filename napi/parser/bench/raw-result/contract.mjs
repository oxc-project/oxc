// oxlint-disable no-console, typescript/unbound-method
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseSync, parse } from "../../src-js/index.js";
import "./prepare.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const { parseSyncRaw: baselineSync, parse: baselineAsync } = await import(
  pathToFileURL(path.join(root, "target/raw-result/baseline.mjs"))
);
const fixtures = [
  ["x.js", "#!/usr/bin/env node\r\n/* 雪😀 */ export const x = [123n, /雪/gu, '\\uD800'];"],
  ["x.ts", "#!/usr/bin/env node\n/* 😀 */ export type T = string; const x: T = '\\uDFFF';"],
  ["x.jsx", "// 😀\r\nexport default <div title='雪'>😀</div>;"],
  ["x.tsx", "/* 😀 */ export const x: JSX.Element = <div>{123n}</div>;"],
  ["x.js", "export const = ;"],
  ["x.ts", "let x: = ;"],
  ["x.js", 'const x = "\ud800";'],
  ["x.js", "let x; let x;"],
];
const report = { node: process.version, optionsChecked: 0, jsonDifferences: [], compatibility: {} };
for (const range of [false, true]) {
  for (const experimentalParent of [false, true]) {
    for (const experimentalTokens of [false, true]) {
      const options = {
        experimentalRawTransfer: true,
        range,
        experimentalParent,
        experimentalTokens,
        showSemanticErrors: true,
      };
      for (const [filename, source] of fixtures) {
        const baseline = baselineSync(filename, source, options);
        const actual = parseSync(filename, source, options);
        assert.deepEqual(Reflect.ownKeys(actual), Reflect.ownKeys(baseline));
        assert.equal(Object.getPrototypeOf(actual), Object.getPrototypeOf(baseline));
        for (const key of Object.keys(baseline)) {
          assert.deepEqual(actual[key], baseline[key], `${filename}: ${key}`);
          const expectedDescriptor = Object.getOwnPropertyDescriptor(baseline, key);
          const actualDescriptor = Object.getOwnPropertyDescriptor(actual, key);
          assert.deepEqual(
            { ...actualDescriptor, get: null },
            { ...expectedDescriptor, get: null },
          );
          assert.equal(actualDescriptor.get.name, expectedDescriptor.get.name);
          assert.equal(actualDescriptor.get.length, expectedDescriptor.get.length);
        }
        report.optionsChecked++;
        if (!experimentalParent && !experimentalTokens) {
          const json = parseSync(filename, source, { range, showSemanticErrors: true });
          for (const key of ["program", "module", "comments", "errors"]) {
            try {
              assert.deepEqual(baseline[key], json[key]);
            } catch {
              report.jsonDifferences.push({ filename, source, range, key });
            }
          }
        }
      }
    }
  }
}
await Promise.all(
  fixtures.map(async ([filename, source]) => {
    const options = {
      experimentalRawTransfer: true,
      range: true,
      experimentalTokens: true,
      experimentalParent: true,
    };
    const expected = await baselineAsync(filename, source, options);
    const actual = await parse(filename, source, options);
    for (const key of Object.keys(expected)) assert.deepEqual(actual[key], expected[key]);
  }),
);
const options = { experimentalRawTransfer: true };
const baseline = baselineSync("x.js", "let x;", options);
const actual = parseSync("x.js", "let x;", options);
const foreign = parseSync("other.js", "let other;", options);
for (const key of ["program", "module", "comments", "errors"]) {
  const getBefore = Object.getOwnPropertyDescriptor(baseline, key).get;
  const getAfter = Object.getOwnPropertyDescriptor(actual, key).get;
  assert.equal(getBefore.call({}), baseline[key]);
  assert.equal(new Proxy(baseline, {})[key], baseline[key]);
  assert.throws(() => getAfter.call({}), TypeError);
  assert.throws(() => new Proxy(actual, {})[key], TypeError);
  assert.equal(getAfter.call(foreign), foreign[key]);
}
report.compatibility = {
  ownDescriptors: "equal",
  ownKeyOrder: "equal",
  prototype: "Object.prototype",
  privateStorageVisibleToReflectOwnKeys: false,
  getterIdentity: "shared instead of unique per result",
  detachedForeignReceiver: "TypeError instead of original result field",
  proxyWithDefaultReceiver: "TypeError instead of original result field",
  getterOnAnotherResult: "returns receiver's field instead of original result field",
  inheritedReceiver: "TypeError instead of original result field",
};
fs.writeFileSync(
  path.join(root, "target/raw-result/contract.json"),
  JSON.stringify(report, null, 2) + "\n",
);
console.log(JSON.stringify(report));
process.exit(0);
