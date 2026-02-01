# @oxlint/plugins-dev

Development and testing utilities for [Oxlint](https://oxc.rs/docs/guide/usage/linter.html) JS plugins.

This package provides the `RuleTester` class for testing custom Oxlint rules.

## Installation

```bash
npm install --save-dev @oxlint/plugins-dev
```

## Usage

API is identical to [ESLint's `RuleTester`](https://eslint.org/docs/latest/integrate/nodejs-api#ruletester).

```typescript
import { RuleTester } from "@oxlint/plugins-dev";

const ruleTester = new RuleTester();

ruleTester.run("my-rule", myRule, {
  valid: ["const x = 1;"],
  invalid: [
    {
      code: "var x = 1;",
      errors: [{ message: "Use const instead of var" }],
    },
  ],
});
```

## Docs

For full documentation, see [Oxlint JS Plugins docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
