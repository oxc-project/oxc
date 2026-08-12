# oxc-codegen

A fast JavaScript code generator from an [ESTree](https://github.com/estree/estree)-compliant AST,
written in TypeScript.

It is a faithful port of Oxc's Rust [`oxc_codegen`](https://github.com/oxc-project/oxc/tree/main/crates/oxc_codegen)
crate (pretty-printing mode). Output is byte-identical to `oxc_codegen` with default options
(tab indentation, double quotes, comments off).

Unlike the rest of Oxc's JS packages, this package contains no Rust and no native bindings.
It is pure JavaScript, and consumes an AST which already lives on the JS side, so doesn't need
to serialize it across a JS/native boundary. This is the key to `oxc-codegen`'s speed -
along with many optimizations to hit JS engines' fast paths.

See [DESIGN.md](https://github.com/oxc-project/oxc/blob/main/packages/codegen/DESIGN.md) for more details
on the implementation, and what makes it fast.

100% tests passing against Test262, Acorn-JSX, and TypeScript conformance suites (62,000 tests in total).

## Usage

```js
import { parseSync } from "oxc-parser";
import { printSync } from "oxc-codegen";

const { program } = parseSync("foo.js", "let x = 1");
console.log(printSync(program));
```

## Supported language variants

- JS
- JSX
- TS
- TSX

## API

### `printSync(node, options?)`

Returns a string containing the code for the AST `node`.

`node` must be a whole AST (`Program` node) or a statement node.

### Options

| Option                | Type                 | Default | Description                                     |
| :-------------------- | :------------------- | :------ | :---------------------------------------------- |
| `indent`              | `string`             | `"\t"`  | Indentation - spaces and/or tabs only           |
| `startingIndentLevel` | `number`             | `0`     | Indent level to start from                      |
| `jsx`                 | `boolean`            | `false` | `.tsx` mode - lone type parameters print `<T,>` |
| `ts`                  | `boolean`            | `false` | AST may contain TypeScript syntax               |
| `sourceMap`           | `SourceMapGenerator` | -       | If present, source mappings are emitted into it |

## Missing features

- There is currently no support for printing comments.
- Pretty-printing only, no compact/minified output.
- Source map support is only lightly tested, and API is likely to change.

## Benchmarks

| fixture                    |   bytes |       Oxc |
| :------------------------- | ------: | --------: |
| tiny.js                    |      26 |  0.0001ms |
| RadixUIAdoptionSection.jsx |    2518 |  0.0033ms |
| react.development.js       |   72141 |  0.1138ms |
| binder.ts                  |  193077 |  0.2472ms |
| App.tsx                    |  415340 |  0.7490ms |
| lodash.js                  |  544096 |  0.4995ms |
| kitchen-sink.tsx           |  732222 |  2.5682ms |
| antd.js                    | 6683633 | 11.3914ms |

Numbers above are from one machine and are not a regression baseline - the `antd.js` figures in
particular move by over 10% run to run.
