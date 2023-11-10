# The JavaScript Oxidation Compiler

See index.d.ts for `resolveSync` and `ResolverFactory` API.

## ESM

```javascript
import path from 'path';
import resolve, { ResolverFactory } from './index.js';
import assert from 'assert';

// `resolve`
assert(resolve.sync(process.cwd(), "./index.js").path, path.join(cwd, 'index.js'));

// `ResolverFactory`
const resolver = new ResolverFactory();
assert(resolver.sync(process.cwd(), "./index.js").path, path.join(cwd, 'index.js'));
```
