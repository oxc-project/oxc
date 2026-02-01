# @oxlint/plugins

Plugin utilities for [Oxlint](https://oxc.rs/docs/guide/usage/linter.html).

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

const myRule = defineRule({
  create(context) {
    return {
      Program(node) {
        // Rule logic here
      },
    };
  },
});

export default definePlugin({
  name: "my-plugin",
  rules: { "my-rule": myRule },
});
```

### Types

This package also includes types for plugins and rules.

```typescript
import type { Context, Rule, ESTree } from "@oxlint/plugins";

const myRule: Rule = {
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

If your plugin uses Oxlint's [alternative `createOnce` API](https://oxc.rs/docs/guide/usage/linter/js-plugins.html#alternative-api),
use `eslintCompatPlugin` to convert the plugin so it will also work with ESLint.

```typescript
import { eslintCompatPlugin } from "@oxlint/plugins";

const myRule = {
  createOnce(context) {
    return {
      Program(node) {
        // Rule logic here
      },
    };
  },
};

export default eslintCompatPlugin({
  name: "my-plugin",
  rules: { "my-rule": myRule },
});
```

## Docs

For full documentation, see [Oxlint JS Plugins docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
