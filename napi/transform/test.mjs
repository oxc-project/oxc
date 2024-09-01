import oxc from "./index.js";
import assert from "assert";

console.log(`Testing on ${process.platform}-${process.arch}`);

test(oxc.isolatedDeclaration("test.ts", "class A {}", { sourcemap: true }), {
  sourceText: "declare class A {}\n",
  sourceMap: {
    mappings: "AAAA,cAAM,EAAE,CAAE",
    names: [],
    sources: ["test.ts"],
    sourcesContent: ["class A {}"],
  },
});

function test(ret, expected) {
  console.log(ret.sourceText);
  console.log(ret.sourceMap);
  for (const error of ret.errors) {
    console.log(error);
  }
  assert.equal(ret.sourceText, expected.sourceText);
  assert.deepEqual(ret.sourceMap, expected.sourceMap);
  assert(ret.errors.length == 0);
}
