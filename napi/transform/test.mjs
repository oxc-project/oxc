import oxc from './index.js';
import assert from 'assert';

console.log(`Testing on ${process.platform}-${process.arch}`)

test(oxc.isolatedDeclaration("test.ts", "class A {}"), "declare class A {}\n");

function test(ret, expected) {
  console.log(ret);
  assert.equal(ret.sourceText, expected);
  assert(ret.errors.length == 0);
}
