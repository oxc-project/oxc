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

if (result.errors.length > 0) {
  console.error(result.errors);
}

console.log(result.code);
```

The React Compiler options use the same names as
`babel-plugin-react-compiler`/`react-compiler-napi`, including
`compilationMode`, `panicThreshold`, `target`, `gating`, `outputMode`,
suppression controls, and the supported `environment` flags.

Callback-valued options such as `logger`, function-valued `sources`, and type
provider callbacks are not accepted by the native binding. `sources` accepts
an array of filename substrings instead.
