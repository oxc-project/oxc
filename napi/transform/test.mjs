import oxc from './index.js';
import assert from 'assert';

console.log(`Testing on ${process.platform}-${process.arch}`)

test(oxc.isolatedDeclaration("test.ts", "class A {}"), "declare class A {}\n");

function test(ret, expected) {
  console.log(ret.sourceText);
  for (const error of ret.errors) {
    console.log(error)
  }
  assert.equal(ret.sourceText, expected);
  assert(ret.errors.length == 0);
}
