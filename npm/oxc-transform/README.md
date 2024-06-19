# Oxc Transform

## [Isolated Declarations for Standalone DTS Emit](https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-beta/#isolated-declarations)

Based on Oxc and conforms to TypeScript Compiler's `--isolated-declaration` `.d.ts` emit.

This is still in alpha and may yield incorrect results, feel free to [submit a bug report](https://github.com/oxc-project/oxc/issues/new?assignees=&labels=C-bug&projects=&template=bug_report.md&title=isolated-declarations:).

### Usage

```javascript
import assert from 'assert';
import oxc from 'oxc-transform';

const { sourceText, errors } = oxc.isolatedDeclaration("test.ts", "class A {}");

assert.equal(sourceText, "declare class A {}\n");
assert(errors.length == 0);
```

### API

```typescript
export function isolatedDeclaration(filename: string, sourceText: string): IsolatedDeclarationsResult

export interface IsolatedDeclarationsResult {
  sourceText: string
  errors: Array<string>
}
```
