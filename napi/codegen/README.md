# oxc-codegen

Print an ESTree AST to JavaScript / TypeScript source code, powered by
[Oxc](https://oxc.rs)'s code generator.

> [!WARNING]
> Not implemented yet. This package defines the complete API surface and its tests, but
> `print` throws until the `raw_transfer_back` ESTree → arena encoder lands. The package is
> `private` and unpublished until then.

## Usage

```js
import { parseSync } from "oxc-parser";
import { print } from "oxc-codegen";

const { program } = parseSync("test.js", "const x = 1 + 2;");

print(program).code;
// "const x = 1 + 2;\n"
```

The AST must be in the shape `oxc-parser` produces (ESTree / TS-ESTree compatible).

### Options

```js
print(program, {
  // Original source text. Required for printing comments and for accurate source maps.
  sourceText,
  // Source filename, used as the `source` field of the source map. Default: "unknown".
  filename,
  // Produce a source map, returned as `map` on the result. Default: false.
  sourcemap: false,
  // Use single quotes instead of double quotes. Default: false.
  singleQuote: false,
  // Remove whitespace. Default: false.
  minify: false,
  // Print comments (requires `sourceText`): boolean, or `{ normal, jsdoc, annotation, legal }`.
  comments: true,
  // Indentation: "tab" (default) or "space".
  indentChar: "tab",
  // Characters per indentation level. Default: 1.
  indentWidth: 1,
  // Initial indentation level. Default: 0.
  initialIndent: 0,
});
```

## How it works

`print` serializes the ESTree AST directly into a 4 GiB-aligned buffer as real oxc arena
structs (the reverse of `oxc-parser`'s raw transfer), then the native side uses the buffer as
its memory arena and runs `oxc_codegen` on it with zero deserialization work.
