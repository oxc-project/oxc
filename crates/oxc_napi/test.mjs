import oxc from './index.js';
import assert from 'assert';

console.log(`Testing on ${process.platform}-${process.arch}`)

function test(ret) {
  console.log(ret.program);
  console.log(ret.errors);
  assert(ret.program.body.length == 1);
  assert(ret.errors.length == 0);
}

test(oxc.parseSync("foo"));

async function main() {
  test(await oxc.parseAsync("foo"));
}

main()
