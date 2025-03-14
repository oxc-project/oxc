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

Check out [oxc-parser](https://www.npmjs.com/package/oxc-parser) for an alternative in Node.js
which performs the same function, but using native code via NAPI bindings (slightly faster).

Source code: https://github.com/oxc-project/oxc/tree/main/wasm/parser

## Usage

### Node.js

```js
import { parseSync } from '@oxc-parser/wasm';

const code = 'let foo';
const result = parseSync(code, { sourceFilename: 'test.ts' });
console.log(result.program);
```

### Browser

Try Vite example app on Stackblitz https://stackblitz.com/edit/vitejs-vite-ydyvyjav?file=src%2Fmain.js

```js
import initWasm, { parseSync } from '@oxc-parser/wasm';

await initWasm();

const code = 'let foo';
const result = parseSync(code, { sourceFilename: 'test.ts' });
console.log(result.program);
```

## Notes

The AST returned conforms to the [ESTree](https://github.com/estree/estree) spec for JS syntax.

For TypeScript code, the AST is broadly aligned with
[typescript-eslint](https://typescript-eslint.io/packages/parser/)'s format, though there may be some
differences.
