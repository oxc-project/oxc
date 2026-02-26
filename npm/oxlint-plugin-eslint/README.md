# oxlint-plugin-eslint

ESLint's built-in rules as an Oxlint plugin.

This package exports all of ESLint's built-in rules as a JS plugin that Oxlint users can use.

## Usage

Install the package:

```sh
npm install --save-dev oxlint-plugin-eslint eslint
```

Add to your Oxlint config:

```json
{
  "jsPlugins": ["oxlint-plugin-eslint"],
  "rules": {
    "eslint-js/no-unused-vars": "error"
  }
}
```

The plugin's `meta.name` is `eslint-js`, so all rules are prefixed with `eslint-js/`.

## Why?

This package is useful when:

1. You want to use an ESLint rule that Oxlint doesn't implement natively yet.
2. You encounter a bug in Oxlint's native implementation of an ESLint rule and need a temporary fix.

All rules from ESLint are included, even those that Oxlint already implements natively. You can switch to the JS plugin version as a "hotfix" if needed.

## License

MIT
