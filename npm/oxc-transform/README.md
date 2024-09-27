# Oxc Transform

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-beta/#isolated-declarations)

Based on Oxc and conforms to TypeScript Compiler's `--isolated-declaration` `.d.ts` emit.

This is still in alpha and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md&title=isolated-declarations:).

### Usage

```javascript
import assert from 'assert';
import oxc from 'oxc-transform';

const { map, code, errors } = oxc.isolatedDeclaration('test.ts', 'class A {}', { sourcemap: true });

assert.equal(code, 'declare class A {}\n');
assert.deepEqual(map, {
  mappings: 'AAAA,cAAM,EAAE,CAAE',
  names: [],
  sources: ['test.ts'],
  sourcesContent: ['class A {}'],
});
assert(errors.length == 0);
```

### API

```typescript
export function isolatedDeclaration(
  filename: string,
  sourceText: string,
  options?: IsolatedDeclarationsOptions,
): IsolatedDeclarationsResult;

export interface IsolatedDeclarationsOptions {
  stripInternal?: boolean;
  sourcemap?: boolean;
}

export interface IsolatedDeclarationsResult {
  code: string;
  map?: SourceMap;
  errors: Array<string>;
}
```
