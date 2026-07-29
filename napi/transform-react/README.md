# Oxc React Transform

Native Node.js bindings for Oxc's experimental Rust port of React Compiler.

The API follows `oxc-transform`: pass a filename, source text, and optional
options to either `transformSync` or `transform`. React Compiler runs first,
then Oxc removes TypeScript syntax and lowers JSX.

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
    target: "19",
  },
);

if (result.fatal) {
  console.error(result.errors);
} else {
  console.log(result.code);
}
```

`errors` contains every diagnostic reported by parsing, the React Compiler, and
the downstream transform. Some React Compiler bail-outs have error severity but
are nonfatal under the default `panicThreshold`; check `fatal` to decide whether
the transform emitted usable code.

The React Compiler options use the same names as
`babel-plugin-react-compiler`/`react-compiler-napi`, including
`compilationMode`, `panicThreshold`, `target`, `gating`, `outputMode`,
suppression controls, and the supported `environment` flags.

Callback-valued options such as `logger`, function-valued `sources`, and type
provider callbacks are not accepted by the native binding. `sources` accepts
an array of filename substrings instead.
