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

Property mangling is opt-in and independent from identifier mangling. `include` is a JavaScript
`RegExp` and is required when `mangleProps` is present. Its source and flags are compiled with
[Rust's regex engine](https://docs.rs/regex/latest/regex/#syntax). Flags `i`, `m`, `s`, and `u` are
supported. Other JavaScript flags and unsupported syntax are reported in `errors`.

```javascript
const result = minifySync("component.js", source, {
  mangle: false,
  mangleProps: {
    include: /^_/,
    exclude: /^__public/,
    reserved: ["_externalApi"],
    quoted: false,
    cache: previousResult?.mangleCache,
  },
});

saveCache(result.mangleCache);
```

The returned `mangleCache` contains the input cache plus newly assigned names when parsing
finishes without errors, sorted by original name. A `false` cache value keeps that property
unchanged. Feeding the cache back keeps recorded mappings stable for later unminified input, but
the cache does not discover unchanged names that exist only in another input. Callers coordinating
separate files must pin or reserve those names explicitly. Property mappings are
single-application: do not minify already-mangled output with the same cache. Custom target names
may be shared deliberately, but must be valid JavaScript `IdentifierName` values and cannot be
`__proto__`, `constructor`, or `prototype`. The original name `__proto__` is always reserved and
cannot be used as a cache key.

With `quoted: false`, quoting is handled per occurrence: `obj._field` is eligible while
`obj["_field"]` is not. Property mangling assumes matching properties are never accessed through
arbitrary dynamic strings. Use `/* @__KEY__ */ "_field"` for a string that semantically names a
property, such as a reflective API argument. Names reached through direct `eval`, the `Function`
constructor, or `with` must be reserved or excluded explicitly.

Properties owned by unminified code, imported module namespaces, globals, DOM objects, or other
host APIs must also be excluded or reserved.

## Assumptions

`oxc-minify` makes some assumptions about the source code.

See https://github.com/oxc-project/oxc/blob/main/crates/oxc_minifier/README.md#assumptions for details.

### Supports WASM

See https://stackblitz.com/edit/oxc-minify for usage example.
