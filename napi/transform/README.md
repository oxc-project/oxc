# Oxc Transform

This is alpha software and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md).

## TypeScript and React JSX Transform

```javascript
import assert from 'assert';
import { transform } from 'oxc-transform';

const { code, declaration, errors } = transform(
  'test.ts',
  'class A<T> {}',
  {
    typescript: {
      declaration: true, // With isolated declarations in a single step.
    },
  },
);
// or `await transformAsync(filename, code, options)`

assert.equal(code, 'class A {}\n');
assert.equal(declaration, 'declare class A<T> {}\n');
assert(errors.length == 0);
```

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#isolated-declarations)

Conforms to TypeScript compiler's `--isolatedDeclarations` `.d.ts` emit.

### Usage

```javascript
import assert from 'assert';
import { isolatedDeclaration } from 'oxc-transform';

const { map, code, errors } = isolatedDeclaration('test.ts', 'class A {}');

assert.equal(code, 'declare class A {}\n');
assert(errors.length == 0);
```

### API

See `index.d.ts`.

```typescript
export declare function transform(
  filename: string,
  sourceText: string,
  options?: TransformOptions,
): TransformResult;

export function isolatedDeclaration(
  filename: string,
  sourceText: string,
  options?: IsolatedDeclarationsOptions,
): IsolatedDeclarationsResult;
```

### Supports WASM

See https://stackblitz.com/edit/oxc-transform for usage example.
