# Oxc Transform

This is alpha software and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md).

## TypeScript and React JSX Transform

```javascript
import assert from 'assert';
import oxc from 'oxc-transform';

const { code, declaration, errors } = oxc.transform(
  'test.ts',
  'class A<T> {}',
  {
    typescript: {
      declaration: true, // With isolated declarations in a single step.
    },
  },
);

assert.equal(code, 'class A {}\n');
assert.equal(declaration, 'declare class A<T> {}\n');
assert(errors.length == 0);
```

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-beta/#isolated-declarations)

Conforms to TypeScript Compiler's `--isolated-declaration` `.d.ts` emit.

### Usage

```javascript
import assert from 'assert';
import oxc from 'oxc-transform';

const { map, code, errors } = oxc.isolatedDeclaration('test.ts', 'class A {}');

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
