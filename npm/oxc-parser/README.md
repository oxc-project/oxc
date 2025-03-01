# Oxc Parser

## Features

### Fast Mode

By default, Oxc parser does not produce semantic errors where symbols and scopes are needed.

To enable semantic errors, apply the option `showSemanticErrors: true`.

For example,

```js
let foo;
let foo;
```

Does not produce any errors when `showSemanticErrors` is `false`, which is the default behavior.

This mode is best suited for parser plugins, where other parts of your build pipeline has already checked for errors.

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
import oxc from './index.js';

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
