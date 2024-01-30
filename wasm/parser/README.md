## About

Experimental wasm package for the oxc parser, with full TypeScript typings support.

This package is built with `wasm-pack build --release --target web` for bundler (webpack / vite) consumption.
Checkout [oxc-parser](https://www.npmjs.com/package/oxc-parser) for usage in node.js.

Source code: https://github.com/oxc-project/oxc/tree/main/wasm/parser

## 🚴 Usage

```js
import initWasm, { parseSync } from "@oxc-parser/wasm";

async function main() {
  await initWasm();

  const code = "let foo";
  const result = parseSync(code, { filename: "test.ts" });
  console.log(result);
}

main();
```
