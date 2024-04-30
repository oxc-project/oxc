# The JavaScript Oxidation Compiler

See index.d.ts for `parseSync` and `parseAsync` API.

## ESM

```javascript
import oxc from "oxc-parser";
import assert from "assert";

function test(ret) {
  const program = JSON.parse(ret.program);
  assert(program.body.length == 1);
  assert(ret.errors.length == 0);
}

const sourceText = "let foo: Foo = 'foo';";
const options = {
  sourceFilename: "text.tsx", // the extension is used to determine which dialect to parse
};

test(oxc.parseSync(sourceText, options));

async function main() {
  test(await oxc.parseAsync(sourceText, options));
}

main();
```
