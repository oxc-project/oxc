# Oxc Minify

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

```javascript
import { minify } from 'oxc-minify';

const filename = 'test.js';
const code = "const x = 'a' + 'b'; console.log(x);";
const options = {
  compress: {
    target: 'esnext',
  },
  mangle: {
    toplevel: false,
  },
  codegen: {
    removeWhitespace: true,
  },
  sourcemap: true,
};
const result = minify(filename, code, options);

console.log(result.code);
console.log(result.map);
```

## Assumptions

`oxc-minify` makes some assumptions about the source code.

See https://github.com/oxc-project/oxc/blob/main/crates/oxc_minifier/README.md#assumptions for details.

### Supports WASM

See https://stackblitz.com/edit/oxc-minify for usage example.
