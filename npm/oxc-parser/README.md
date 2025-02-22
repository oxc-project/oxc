# Oxc Parser

## Features

- Returns ESM information.

## Caveat

The parser alone does not fully check for syntax errors that are associated with semantic data (symbols and scopes).
The full compiler is needed for such case, as the compiler does an additional semantic pass.

With this caveat, `oxc-parser` is best suited for parser plugins,
where you need quick access to ESM information, as well as fast `magic-string` operations.

## API

```javascript
import oxc from './index.js';

const code = 'const url: String = /* 🤨 */ import.meta.url;';

// File extension is used to determine which dialect to parse source as.
const filename = 'test.tsx';

const result = oxc.parseSync(filename, code);
// or `await oxc.parseAsync(filename, code)`

// An array of errors, if any.
console.log(result.errors);

// AST and comments.
console.log(result.program, result.comments);

// ESM information - imports, exports, `import.meta`s.
console.log(result.module);
```
