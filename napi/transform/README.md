# Oxc Transform

This is alpha software and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md).

## TypeScript and React JSX Transform

```javascript
import assert from 'assert';
import { transformSync } from 'oxc-transform';

const { code, declaration, errors } = transformSync(
  'test.ts',
  'class A<T> {}',
  {
    typescript: {
      declaration: true, // With isolated declarations in a single step.
    },
  },
);
// or `await transform(filename, code, options)`

assert.equal(code, 'class A {}\n');
assert.equal(declaration, 'declare class A<T> {}\n');
assert(errors.length == 0);
```

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#isolated-declarations)

Conforms to TypeScript compiler's `--isolatedDeclarations` `.d.ts` emit.

### Usage

```javascript
import assert from 'assert';
import { isolatedDeclarationSync } from 'oxc-transform';

const { map, code, errors } = isolatedDeclarationSync('test.ts', 'class A {}');
// or `await isolatedDeclaration(filename, code, options)`

assert.equal(code, 'declare class A {}\n');
assert(errors.length == 0);
```

### API

#### Transform Functions

```typescript
// Synchronous transform
transformSync(
  filename: string,
  sourceText: string,
  options?: TransformOptions,
): TransformResult

// Asynchronous transform
transform(
  filename: string,
  sourceText: string,
  options?: TransformOptions,
): Promise<TransformResult>
```

#### Isolated Declaration Functions

```typescript
// Synchronous isolated declaration
isolatedDeclarationSync(
  filename: string,
  sourceText: string,
  options?: IsolatedDeclarationsOptions,
): IsolatedDeclarationsResult

// Asynchronous isolated declaration
isolatedDeclaration(
  filename: string,
  sourceText: string,
  options?: IsolatedDeclarationsOptions,
): Promise<IsolatedDeclarationsResult>
```

Use the `Sync` versions for synchronous operations. Use async versions for asynchronous operations, which can be beneficial in I/O-bound or concurrent scenarios, though they add async overhead.

See `index.d.ts` for complete type definitions.

### Supports WASM

See https://stackblitz.com/edit/oxc-transform for usage example.
