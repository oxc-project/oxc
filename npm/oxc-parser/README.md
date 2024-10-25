# The JavaScript Oxidation Compiler

See `index.d.ts` for `parseSync` and `parseAsync` API.

```javascript
import assert from 'assert';
import oxc from 'oxc-parser';

const sourceText = "let foo: Foo = 'foo';";
// Filename extension is used to determine which
// dialect to parse source as
const options = { sourceFilename: 'text.tsx' };

test(oxc.parseSync(sourceText, options));
test(await oxc.parseAsync(sourceText, options));

function test(ret) {
  assert(ret.program.body.length == 1);
  assert(ret.errors.length == 0);
}
```
