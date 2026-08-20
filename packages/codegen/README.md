# oxc-codegen

Fast, synchronous code generation for JavaScript and TypeScript ASTs.

`oxc-codegen` turns an [ESTree](https://github.com/estree/estree) or
[TS-ESTree](https://typescript-eslint.io/packages/typescript-estree/) AST into formatted source
code. It supports JavaScript, JSX, TypeScript, and TSX.

The printer is a port of Oxc's Rust `oxc_codegen` crate. With the default options, both printers
produce byte-identical output: tab indentation, double-quoted strings, and no comments.

## Installation

```sh
npm install oxc-codegen
```

`oxc-codegen` is ESM-only and requires Node.js `^20.19.0` or `>=22.12.0`.

## Quick start

Pair it with [`oxc-parser`](https://www.npmjs.com/package/oxc-parser) to parse and print source code:

```js
import { printSync } from "oxc-codegen";
import { parseSync } from "oxc-parser";

const { program } = parseSync("input.js", "const answer=6*7");
const { code } = printSync(program);

console.log(code);
// const answer = 6 * 7;
```

You can also print a manually constructed AST:

```js
const program = {
  type: "Program",
  sourceType: "script",
  body: [
    {
      type: "ExpressionStatement",
      expression: {
        type: "CallExpression",
        callee: {
          type: "MemberExpression",
          object: { type: "Identifier", name: "console" },
          property: { type: "Identifier", name: "log" },
          computed: false,
          optional: false,
        },
        arguments: [{ type: "Literal", value: "Hello!" }],
        optional: false,
      },
    },
  ],
};

console.log(printSync(program).code);
// console.log("Hello!");
```

### TypeScript and TSX

Set `ts` when the AST can contain TypeScript nodes. For TSX, set both `ts` and `jsx`:

```js
const { program } = parseSync("component.tsx", "const Box = <T,>(value: T) => <div>{value}</div>");

const { code } = printSync(program, {
  ts: true,
  jsx: true,
});
```

## API

### `printSync(node, options?)`

```ts
function printSync(
  node: ESTree.Program | ESTree.Statement,
  options?: Options,
): {
  code: string;
  map: SourceMap | null;
};
```

Prints a complete `Program` or a single statement and returns the generated source code,
and (when requested) a standard Source Map v3 object.

```js
import { printSync } from "oxc-codegen";
import { parseSync } from "oxc-parser";

const sourceText = "const answer=6*7";
const { program } = parseSync("input.js", sourceText);
const { code, map } = printSync(program, {
  sourcemap: true,
  sourceFilename: "input.js",
  sourceText,
});
```

Source-map mappings require `sourceText` and nodes with valid Oxc `start` / `end` offsets.
A manually constructed AST without offsets can still be printed, but its source map has
an empty `mappings` string.

### Options

| Option                | Type      | Default | Description                                                      |
| :-------------------- | :-------- | :------ | :--------------------------------------------------------------- |
| `indent`              | `string`  | `"\t"`  | Non-empty string of spaces and/or tabs used for one indent level |
| `startingIndentLevel` | `number`  | `0`     | Starting indent level, from `0` to `1000`                        |
| `jsx`                 | `boolean` | `false` | Enable TSX-safe printing for ambiguous TypeScript syntax         |
| `ts`                  | `boolean` | `false` | Select the printer that supports TypeScript nodes                |
| `sourcemap`           | `boolean` | `false` | Return a Source Map v3 object in `map`                           |
| `sourceFilename`      | `string`  | `""`    | Original source filename recorded in the source map              |
| `sourceText`          | `string`  | -       | Original text required for source-map mappings and content       |

## Why pure JavaScript?

Most Oxc packages use native bindings. This package deliberately does not: when an AST already
lives in JavaScript, passing the entire object graph across a JS/native boundary can cost more than
printing it in place. `oxc-codegen` avoids that serialization and uses specialized printer builds
for JavaScript and TypeScript workloads.

See [DESIGN.md](https://github.com/oxc-project/oxc/blob/main/packages/codegen/DESIGN.md) for the
implementation details and performance constraints.

## Current limitations

- Comments are not printed.
- Minified output is not supported.

## Benchmarks

Representative time per `printSync` call:

| Fixture                      |     Bytes |       Time |
| :--------------------------- | --------: | ---------: |
| `tiny.js`                    |        26 |  0.0001 ms |
| `RadixUIAdoptionSection.jsx` |     2,518 |  0.0033 ms |
| `react.development.js`       |    72,141 |  0.1138 ms |
| `binder.ts`                  |   193,077 |  0.2472 ms |
| `App.tsx`                    |   415,340 |  0.7490 ms |
| `lodash.js`                  |   544,096 |  0.4995 ms |
| `kitchen-sink.tsx`           |   732,222 |  2.5682 ms |
| `antd.js`                    | 6,683,633 | 11.3914 ms |

These figures come from one machine and are illustrative, not a regression baseline. Results—most
noticeably for large fixtures such as `antd.js`—vary between runs.
