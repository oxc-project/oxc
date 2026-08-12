# Oxc React Transform

Native Node.js bindings for Oxc's experimental Rust port of React Compiler.

The API follows `oxc-transform`: pass a filename, source text, and optional
options to either `transformSync` or `transform`. React Compiler runs first,
then Oxc removes TypeScript syntax and applies the configured JSX transforms.

```javascript
import { transformSync } from "oxc-transform-react";

const result = transformSync(
  "Component.tsx",
  `
    export function Component({ name }: { name: string }) {
      return <div>Hello {name}</div>;
    }
  `,
  {
    reactCompiler: {
      target: "19",
    },
    jsx: {
      runtime: "automatic",
    },
  },
);

if (result.fatal) {
  console.error(result.errors);
} else {
  console.log(result.code);
}
```

`errors` contains every diagnostic reported by parsing, the React Compiler, and
the downstream transform. Recoverable React Compiler bail-outs are warnings;
compiler diagnostics retain error severity when `panicThreshold` makes the
transform fatal. Check `fatal` to decide whether the transform emitted usable
code.

`reactCompiler` defaults to `true`. Set it to `false` to skip React Compiler,
or pass an options object using the same names as
`babel-plugin-react-compiler`/`react-compiler-napi`, including
`compilationMode`, `panicThreshold`, `target`, `gating`, `outputMode`,
suppression controls, and the supported `environment` flags.

`jsx` accepts the same options as `oxc-transform`, including automatic or
classic runtime configuration and React Fast Refresh. Set it to `"preserve"`
to leave JSX syntax in the output.

Callback-valued options such as `logger`, function-valued `sources`, and type
provider callbacks are not accepted by the native binding. `sources` accepts
an array of filename substrings instead.
