# Oxc Parser

## Features

### ESTree

The returned JavaScript AST follows the [ESTree](https://github.com/estree/estree) specification.

It is fully aligned with Acorn's AST, and any deviation would be considered a bug.

The returned TypeScript AST will conform to [@typescript-eslint/typescript-estree](https://www.npmjs.com/package/@typescript-eslint/typescript-estree) in the near future.

### AST Types

[@oxc-project/types](https://www.npmjs.com/package/@oxc-project/types) can be used. For example:

```typescript
import { Statement } from '@oxc-project/types';
```

### Visitor

[oxc-walker](https://www.npmjs.com/package/oxc-walker) or [estree-walker](https://www.npmjs.com/package/estree-walker) can be used.

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

### ESTree compatibility

When parsing JS or JSX files, the AST returned is fully conformant with the
[ESTree standard](https://github.com/estree/estree).

When parsing TS or TSX files, the AST has additional properties related to TypeScript syntax.
These extra properties are broadly (but not entirely) in line with
[TypeScript ESLint](https://typescript-eslint.io/packages/parser/)'s AST.

If you need all ASTs in the same with-TS-properties format, use the `astType: 'ts'` option.

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

```javascript
import oxc from 'oxc-parser';

const code = 'const url: String = /* 🤨 */ import.meta.url;';

// File extension is used to determine which dialect to parse source as.
const filename = 'test.tsx';

const result = oxc.parseSync(filename, code);
// or `await oxc.parseAsync(filename, code)`

// An array of errors, if any.
console.log(result.errors);

// AST and comments.
console.log(result.program, result.comments);

// ESM information - imports, exports, `import.meta`s.
console.log(result.module);
```
