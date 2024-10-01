import assert from 'assert';
import oxc from './index.js';

console.log(`Testing on ${process.platform}-${process.arch}`);

function test(ret) {
  console.log(ret);
  assert(JSON.parse(ret.program).body.length == 1);
  assert(ret.errors.length == 0);
  assert(ret.comments.length == 1);
}

const sourceText = '/* comment */ foo';

test(oxc.parseSync(sourceText));

async function main() {
  test(await oxc.parseAsync(sourceText));
}

main();
