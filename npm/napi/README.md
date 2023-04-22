# The JavaScript Oxidation Compiler

See index.d.ts for `parseSync` and `parseAsync` API.

TypeScript typings for the AST is currently work in progress.

## cjs

```javascript
const oxc = require("@oxidation-compiler/napi");
const assert = require('assert');

function test(ret) {
  assert(ret.program.body.length == 1);
  assert(ret.errors.length == 0);
}

test(oxc.parseSync("foo"));

async function main() {
  test(await oxc.parseAsync("foo"));
}

main()
```

## ESM

```javascript
import oxc from '@oxidation-compiler/napi';
import assert from 'assert';

function test(ret) {
  assert(ret.program.body.length == 1);
  assert(ret.errors.length == 0);
}

test(oxc.parseSync("foo"));

async function main() {
  test(await oxc.parseAsync("foo"));
}

main()
```
