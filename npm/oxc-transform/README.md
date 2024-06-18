# The JavaScript Oxidation Compiler

See index.d.ts for `parseSync` and `parseAsync` API.

## ESM

```javascript
import oxc from './index.js';
import assert from 'assert';

test(oxc.isolatedDeclaration("test.ts", "class A {}"), "declare class A {}\n");

function test(ret, expected) {
  assert.equal(ret.sourceText, expected);
  assert(ret.errors.length == 0);
}
```
