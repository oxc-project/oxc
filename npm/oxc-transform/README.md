# Oxc Transform

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-beta/#isolated-declarations)

Based on Oxc and conforms to TypeScript Compiler's `--isolated-declaration` `.d.ts` emit.

This is still in alpha and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md&title=isolated-declarations:).

### Usage

```javascript
import assert from 'assert';
import oxc from 'oxc-transform';

const { sourceMap, sourceText, errors } = oxc.isolatedDeclaration("test.ts", "class A {}", { sourcemap: true });

assert.equal(sourceText, "declare class A {}\n");
assert.deepEqual(ret.sourceMap, {
  mappings: "AAAA,cAAM,EAAE,CAAE",
  names: [],
  sources: ["test.ts"],
  sourcesContent: ["class A {}"],
});
assert(errors.length == 0);
```

### API

```typescript
export function isolatedDeclaration(filename: string, sourceText: string, options: IsolatedDeclarationsOptions): IsolatedDeclarationsResult

export interface IsolatedDeclarationsOptions {
  sourcemap: boolean
}

export interface IsolatedDeclarationsResult {
  sourceText: string
  errors: Array<string>
}
```
