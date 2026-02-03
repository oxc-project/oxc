# @oxlint/plugins

Plugin utilities for [Oxlint](https://oxc.rs/docs/guide/usage/linter/js-plugins).

This package provides optional functions to assist in creating Oxlint JS plugins and rules.

## Installation

```bash
npm install @oxlint/plugins
```

## Usage

### Define functions

Use `definePlugin` and `defineRule` if authoring your plugin in TypeScript for type safety.

```typescript
import { definePlugin, defineRule } from "@oxlint/plugins";

const rule = defineRule({
  create(context) {
    return {
      Program(node) {
        // Rule logic here
      },
    };
  },
});

export default definePlugin({
  meta: { name: "oxlint-plugin-amazing" },
  rules: { amazing: rule },
});
```

### Types

This package also includes types for plugins and rules.

```typescript
import type { Context, Rule, ESTree } from "@oxlint/plugins";

const rule: Rule = {
  create(context: Context) {
    return {
      Program(node: ESTree.Program) {
        // Rule logic here
      },
    };
  },
};
```

### ESLint compatibility

If your plugin uses Oxlint's [alternative `createOnce` API](https://oxc.rs/docs/guide/usage/linter/js-plugins#alternative-api),
use `eslintCompatPlugin` to convert the plugin so it will also work with ESLint.

```typescript
import { eslintCompatPlugin } from "@oxlint/plugins";

const rule = {
  createOnce(context) {
    return {
      Program(node) {
        // Rule logic here
      },
    };
  },
};

export default eslintCompatPlugin({
  meta: { name: "oxlint-plugin-amazing" },
  rules: { amazing: rule },
});
```

## Node.js version

This package requires Node.js 12.22.0+, 14.17.0+, 16.0.0+, or later.
This matches the minimum Node.js version required by ESLint 8.

This package provides both ESM and CommonJS entry points.

So a plugin which depends on `@oxlint/plugins` can be:

- Used with any version of Oxlint.
- Used with ESLint 8+.
- Published as either ESM or CommonJS.

## Docs

For full documentation, see [Oxlint JS Plugins docs](https://oxc.rs/docs/guide/usage/linter/js-plugins).
