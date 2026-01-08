# Oxc Parser

## Features

### Supports WASM

See https://stackblitz.com/edit/oxc-parser for usage example.

### ESTree

When parsing JS or JSX files, the AST returned is fully conformant with the
[ESTree standard](https://github.com/estree/estree), the same as produced by
[Acorn](https://www.npmjs.com/package/acorn).

When parsing TypeScript, the AST conforms to [@typescript-eslint/typescript-estree](https://www.npmjs.com/package/@typescript-eslint/typescript-estree)'s TS-ESTree format.

If you need all ASTs in the same with-TS-properties format, use the `astType: 'ts'` option.

The only differences between Oxc's AST and ESTree / TS-ESTree are:

- Support for Stage 3 [decorators](https://github.com/tc39/proposal-decorators).
- Support for Stage 3 ECMA features [`import defer`](https://github.com/tc39/proposal-defer-import-eval)
  and [`import source`](https://github.com/tc39/proposal-source-phase-imports).
- In TS-ESTree AST, `import.defer(...)` and `import.source(...)` are represented as an `ImportExpression`
  with `'defer'` or `'source'` in `phase` field (as in ESTree spec), where TS-ESLint represents these
  as a `CallExpression` with `MetaProperty` as its `callee`.
- Addition of a non-standard `hashbang` field to `Program`.

That aside, the AST should completely align with Acorn's ESTree AST or TS-ESLint's TS-ESTree.
Any deviation would be considered a bug.

### AST Types

[@oxc-project/types](https://www.npmjs.com/package/@oxc-project/types) can be used. For example:

```typescript
import { Statement } from "@oxc-project/types";
```

### Visitor

An AST visitor is provided. See example below.

This package also exports visitor keys which can be used with any other ESTree walker.

```js
import { visitorKeys } from "oxc-parser";
```

### Fast Mode

By default, Oxc parser does not produce semantic errors where symbols and scopes are needed.

To enable semantic errors, apply the option `showSemanticErrors: true`.

For example,

```js
let foo;
let foo;
```

Does not produce any errors when `showSemanticErrors` is `false`, which is the default behavior.

Fast mode is best suited for parser plugins, where other parts of your build pipeline has already checked for errors.

Please note that turning off fast mode ​incurs​ a small performance overhead.

### Returns ESM information.

It is likely that you are writing a parser plugin that requires ESM information.

To avoid walking the AST again, Oxc Parser returns ESM information directly.

This information can be used to rewrite import and exports with the help of [`magic-string`](https://www.npmjs.com/package/magic-string),
without any AST manipulations.

```ts
export interface EcmaScriptModule {
  /**
   * Has ESM syntax.
   *
   * i.e. `import` and `export` statements, and `import.meta`.
   *
   * Dynamic imports `import('foo')` are ignored since they can be used in non-ESM files.
   */
  hasModuleSyntax: boolean;
  /** Import statements. */
  staticImports: Array<StaticImport>;
  /** Export statements. */
  staticExports: Array<StaticExport>;
  /** Dynamic import expressions. */
  dynamicImports: Array<DynamicImport>;
  /** Span positions` of `import.meta` */
  importMetas: Array<Span>;
}
```

## API

### Functions

```typescript
// Synchronous parsing
parseSync(filename: string, sourceText: string, options?: ParserOptions): ParseResult

// Asynchronous parsing
parse(filename: string, sourceText: string, options?: ParserOptions): Promise<ParseResult>
```

Use `parseSync` for synchronous parsing. Use `parse` for asynchronous parsing, which can be beneficial in I/O-bound or concurrent scenarios, though it adds async overhead.

### Example

```javascript
import { parseSync, Visitor } from "oxc-parser";

const code = "const url: String = /* 🤨 */ import.meta.url;";

// File extension is used to determine which dialect to parse source as.
const filename = "test.tsx";

const result = parseSync(filename, code);
// Or use async version: const result = await parse(filename, code);

// An array of errors, if any.
console.log(result.errors);

// AST and comments.
console.log(result.program, result.comments);

// ESM information - imports, exports, `import.meta`s.
console.log(result.module);

// Visit the AST
const visitations = [];

const visitor = new Visitor({
  VariableDeclaration(decl) {
    visitations.push(`enter ${decl.kind}`);
  },
  "VariableDeclaration:exit"(decl) {
    visitations.push(`exit ${decl.kind}`);
  },
  Identifier(ident) {
    visitations.push(ident.name);
  },
});

visitor.visit(result.program);

// Logs: [ 'enter const', 'url', 'String', 'import', 'meta', 'url', 'exit const' ]
console.log(visitations);
```

### Options

All options are optional.

- `lang`: `'js'` | `'jsx'` | `'ts'` | `'tsx'`. Set language of source. If omitted, language is deduced from file extension.
- `sourceType`: `'script'` | `'module'` | `'unambiguous'`. Set source type. Defaults to `'module'`.
- `astType`: `'js'` | `'ts'`. Set to `'ts'` if you want ASTs of plain JS/JSX files to contain TypeScript-specific properties.
- `range`: `true` | `false`. If `true`, AST nodes contain a `range` field. Defaults to `false`.
- `preserveParens`: `true` | `false`. If `true`, parenthesized expressions are represented by (non-standard) `ParenthesizedExpression` and `TSParenthesizedType` AST nodes. Defaults to `true`.
- `showSemanticErrors`: `true` | `false`. If `true`, check file for semantic errors which parser does not otherwise emit e.g. `let x; let x;`. Has a small performance cost. Defaults to `false`.
