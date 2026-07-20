# Oxc Minify

See [usage instructions](https://oxc.rs/docs/guide/usage/minifier).

This is alpha software and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md).

### Performance and Compression Size

See [minification-benchmarks](https://github.com/privatenumber/minification-benchmarks) for details.

The current version already outperforms `esbuild`,
but it still lacks a few key minification techniques
such as constant inlining and dead code removal,
which we plan to implement next.

## Caveats

To maximize performance, `oxc-minify` assumes the input code is semantically correct.
It uses `oxc-parser`'s fast mode to parse the input code,
which does not check for semantic errors related to symbols and scopes.

## API

### Functions

```typescript
// Synchronous minification
minifySync(
  filename: string,
  sourceText: string,
  options?: MinifyOptions,
): MinifyResult

// Asynchronous minification
minify(
  filename: string,
  sourceText: string,
  options?: MinifyOptions,
): Promise<MinifyResult>
```

Use `minifySync` for synchronous minification. Use `minify` for asynchronous minification, which can be beneficial in I/O-bound or concurrent scenarios, though it adds async overhead.

### Example

```javascript
import { minifySync } from "oxc-minify";

const filename = "test.js";
const code = "const x = 'a' + 'b'; console.log(x);";
const options = {
  compress: {
    target: "esnext",
  },
  mangle: {
    toplevel: false,
  },
  codegen: {
    removeWhitespace: true,
  },
  sourcemap: true,
};
const result = minifySync(filename, code, options);
// Or use async version: const result = await minify(filename, code, options);

console.log(result.code);
console.log(result.map);
```

### Property-name mangling

Property mangling is opt-in and independent from identifier mangling. `include` is a
[Rust regex](https://docs.rs/regex/latest/regex/#syntax) source string and is required when
`mangleProps` is present.

```javascript
const result = minifySync("component.js", source, {
  mangle: false,
  mangleProps: {
    include: "^_",
    exclude: "^__public",
    reserved: ["_externalApi"],
    quoted: false,
    cache: previousResult?.mangleCache,
  },
});

saveCache(result.mangleCache);
```

The returned `mangleCache` contains the input cache plus newly assigned names when parsing
finishes without errors. A `false` cache value keeps that property unchanged. Cache keys and
targets are never used for automatically generated names, so the returned cache is safe to feed
back under the same options. Custom target names may be shared deliberately, but must be valid
JavaScript `IdentifierName` values and cannot be `__proto__`.

With `quoted: false`, quoting is handled per occurrence: `obj._field` is eligible while
`obj["_field"]` is not. Property mangling assumes matching properties are never accessed through
arbitrary dynamic strings. Use `/* @__KEY__ */ "_field"` for a string that semantically names a
property, such as a reflective API argument.

## Assumptions

`oxc-minify` makes some assumptions about the source code.

See https://github.com/oxc-project/oxc/blob/main/crates/oxc_minifier/README.md#assumptions for details.

### Supports WASM

See https://stackblitz.com/edit/oxc-minify for usage example.
