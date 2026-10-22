import { transformSync, HelperMode } from './index.js';

// Test case with type annotations
const input = `let old;
const dec = (name) => (cls, ctx) => {
  old = cls;
};
@dec('Foo') class Foo { }
assertEq(() => Foo, old);`;

const result = transformSync("test.ts", input, {
  sourceType: "script",
  target: "esnext",
  decorator: {},
  helpers: { mode: HelperMode.External },
});
console.log("Has errors:", result.errors.length);
console.log("Output:", result.code);
