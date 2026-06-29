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

## Property mangling

`oxc-minify` can rename object property names (`obj.longName` → `obj.e`) to
shrink output further. This is controlled by four options:

| Option          | Type                           | Meaning                                                                                                                             |
| --------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| `mangleProps`   | `string` (regex source)        | Only properties whose name matches this regular expression are renamed. **Setting this is what enables property mangling.**         |
| `reserveProps`  | `string` (regex source)        | Properties whose name matches this regular expression are never renamed, even if they also match `mangleProps`.                     |
| `reservedProps` | `string[]`                     | A literal list of property names that must never be renamed. Added to (never replaces) the built-in reserved set.                   |
| `mangleCache`   | `Record<string, string⎮false>` | A name cache for stable names across builds. Pass it in and read it back off the result. A value of `false` reserves that property. |

Both `mangleProps` and `reserveProps` take a regular expression **source
string** (e.g. `"^_"`), not a `RegExp` object.

### Off by default

Property mangling is **off unless you set `mangleProps`**. `mangle: true` alone
renames **zero** properties — it only mangles variable/function names. Nothing
is renamed until you supply a `mangleProps` regex.

### Unsafe — it can silently break your code

Unlike variable mangling, property mangling is **not safe in general**. The
minifier cannot see every place a property name is referenced, so renaming one
can silently break working code. Breakage modes include:

- **Computed / dynamic access** — `obj[expr]`, `obj["name"]`: the string is not
  renamed, so it no longer matches the renamed property.
- **Reflection** — `Object.keys`, `Object.entries`, `for…in`, and
  `JSON.stringify` / `JSON.parse` observe or produce the original names.
- **Strings outside JS** — names referenced from HTML, CSS, or framework
  templates (Vue/Angular/Svelte/etc.) are invisible to the minifier.
- **String-argument APIs** — `Object.defineProperty(obj, "name", …)`,
  `obj.hasOwnProperty("name")`, `"name" in obj`: the name is a string literal
  that is not renamed in sync with the property.
- **DOM / built-in names** — renaming `addEventListener`, `length`,
  `textContent`, etc. breaks calls into the host. The built-in reserved set is
  small (see limits below), so a broad regex **can** rename these.
- **Code split across files / chunks** — names are assigned per `minify` call,
  so a property renamed in one file will not match the same property in a
  separately-minified file unless you share a `mangleCache`.

### The safe convention

Only mangle properties **you own and never serialize or reflect over**. The
common idiom is to give such properties a leading underscore and mangle only
those:

```javascript
import { minifySync } from "oxc-minify";

const code = `
  class Counter {
    constructor() { this._count = 0; }
    _increment() { this._count++; }
    value() { return this._count; }
  }
`;

const result = minifySync("counter.js", code, {
  mangleProps: "^_", // only rename properties starting with "_"
});

console.log(result.code);
// `_count` and `_increment` become single-letter names (e, t, n, …);
// `value` and `constructor` are left untouched.
```

Mangled property names come out as `e`, `t`, `n`, … (ordered by frequency for
better gzip), **not** `a`, `b`, `c`.

To reserve specific names or carve exceptions out of a broad regex:

```javascript
minifySync("file.js", code, {
  mangleProps: "^_",
  reserveProps: "^_public", // keep anything starting with "_public"
  reservedProps: ["_legacyApi"], // keep this exact name
});
```

To keep names stable across builds (e.g. multiple entry points), pass a shared
`mangleCache`:

```javascript
const a = minifySync("a.js", codeA, { mangleProps: "^_", mangleCache: {} });
const b = minifySync("b.js", codeB, {
  mangleProps: "^_",
  mangleCache: a.mangleCache, // reuse so shared props get the same names
});
```

### v1 limitations

- **Single self-contained program only.** Names are assigned per `minify` call.
  Without a shared `mangleCache`, names are **not** kept consistent across
  separate `minify` calls.
- **No `mangleQuoted`.** Any property seen quoted (`obj["_x"]`, `{ "_x": 1 }`)
  is reserved program-wide, so quoting a property is a way to opt it out.
- **Small built-in reserved set, not the full DOM list.** Only a short protocol
  list is always reserved (`then`, `toJSON`, `toString`, `valueOf`, `length`,
  `name`, `message`, `constructor`, `prototype`, `__proto__`). This is **not**
  the full DOM/built-in name list, so a broad regex such as `"."` **can** rename
  DOM names like `addEventListener`. Prefer the underscore convention over a
  broad regex.
- Property mangling is **disabled for the whole input** if it contains `with` or
  a direct `eval` / `Function` constructor.

## Assumptions

`oxc-minify` makes some assumptions about the source code.

See https://github.com/oxc-project/oxc/blob/main/crates/oxc_minifier/README.md#assumptions for details.

### Supports WASM

See https://stackblitz.com/edit/oxc-minify for usage example.
