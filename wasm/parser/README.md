## About

Experimental wasm package for the oxc parser, with full TypeScript typings support.

This package is built with different [wasm-pack's target](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html) builds:

- `wasm-pack build --target web` for bundler (webpack / vite) consumption.
- `wasm-pack build --target nodejs` for node.js

And exports the files as

```json
"main": "./node/oxc_parser_wasm.js",
"browser": "./web/oxc_parser_wasm.js",
"types": "./node/oxc_parser_wasm.d.ts",
```

Checkout [oxc-parser](https://www.npmjs.com/package/oxc-parser) for usage in node.js via napi bindings.

Source code: https://github.com/oxc-project/oxc/tree/main/wasm/parser

## Usage

```js
import initWasm, { parseSync } from '@oxc-parser/wasm';

async function main() {
  await initWasm();

  const code = 'let foo';
  const result = parseSync(code, { sourceFilename: 'test.ts' });
  console.log(result);
}

main();
```

## Notes

### UTF8 vs UTF16 byte offsets

The `span` value returned from the ASTs and diagnostics is in UTF8 byte offsets. Converting to UTF16 byte offsets:

```js
let sourceTextUtf8 = new TextEncoder().encode(sourceText);

const convertToUtf8 = (sourceTextUtf8, d) => {
  return new TextDecoder().decode(sourceTextUtf8.slice(0, d)).length;
};

const diagnostics = result.errors.map((d) => ({
  from: convertToUtf8(sourceTextUtf8, d.start),
  to: convertToUtf8(sourceTextUtf8, d.end),
  severity: d.severity.toLowerCase(),
  message: d.message,
}));
```

### Vite

`wasm-pack build --target web` is used for the wasm build.

You may need something like https://github.com/nshen/vite-plugin-wasm-pack to get it working with vite,
otherwise vite will load the wasm file as a HTML file causing a `CompileError: WebAssembly.instantiate(): expected magic word` error.
