# Oxc Parser

## Features

- Returns ESM information.
- Built-in `magic-string` on the Rust side exposed through N-API.
- "clever" approach to overcome the Rust UTF8 vs JavaScript UTF16 length problem.

## Caveat

The parser alone does not fully check for syntax errors that are associated with semantic data (symbols and scopes).
The full compiler is needed for such case, as the compiler does an additional semantic pass.

With this caveat, `oxc-parser` is best suited for parser plugins,
where you need quick access to ESM information, as well as fast `magic-string` operations.

## API

```javascript
import oxc from './index.js';

// The emoji makes the span of `import.meta.url` to be different in UTF8 and UTF16.
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

// A `magic-string` instance for accessing and manipulating the source text.
// All returned spans are in UTF8 offsets, which cannot be used directly on our JavaScript.
// JavaScript string lengths are in UTF16 offsets.
const ms = result.magicString;

for (const span of result.module.importMetas) {
  // Extra methods for access the source text through spans with UTF8 offsets.
  console.log(ms.getSourceText(span.start, span.end)); // prints `import.meta`
  console.log(ms.getLineColumnNumber(span.start)); // prints `{ line: 0, column: 20 }`
  console.log(code.substring(ms.getUtf16ByteOffset(span.start)).startsWith('import.meta.url')); // prints `true`
}
```
