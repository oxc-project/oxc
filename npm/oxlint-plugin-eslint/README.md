# oxlint-plugin-eslint

ESLint's built-in rules as an Oxlint plugin.

This package exports all of ESLint's built-in rules as a JS plugin that Oxlint users can use.

Allows using ESLint rules that Oxlint doesn't implement natively yet.

More details in [Oxlint docs](https://oxc.rs/docs/guide/usage/linter/js-plugins).

## Usage

Install the package:

```sh
npm install --save-dev oxlint-plugin-eslint
```

Add to your Oxlint config:

```json
{
  "jsPlugins": ["oxlint-plugin-eslint"],
  "rules": {
    "eslint-js/no-restricted-syntax": [
      "error",
      {
        "selector": "ThrowStatement > CallExpression[callee.name=/Error$/]",
        "message": "Use `new` keyword when throwing an `Error`."
      }
    ]
  }
}
```

All rules are prefixed with `eslint-js/`, to distinguish from Oxlint's Rust implementation of ESLint rules.
