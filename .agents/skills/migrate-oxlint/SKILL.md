---
name: migrate-oxlint
description: Guide for migrating a project from ESLint to Oxlint. Use when asked to migrate, convert, or switch a JavaScript/TypeScript project's linter from ESLint to Oxlint.
---

This skill guides you through migrating a JavaScript/TypeScript project from ESLint to [Oxlint](https://oxc.rs/docs/guide/usage/linter/).

## Overview

Oxlint is a high-performance linter that implements many popular ESLint rules natively in Rust. It can be used alongside ESLint or as a full replacement.

An official migration tool is available, and will be used by this skill: [`@oxlint/migrate`](https://github.com/oxc-project/oxlint-migrate)

## Prerequisites

Before migrating to Oxlint, ensure the following are completed if not already done:

### ESLint Flat Config Migration

The `@oxlint/migrate` tool requires an ESLint flat config (`eslint.config.js`). If the project still uses the legacy format (e.g., `.eslintrc.json`), migrate it first:

```bash
npx @eslint/migrate-config .eslintrc.json
```

This generates an `eslint.config.js` from the legacy config. Review and commit it before proceeding.

### TypeScript 6 Migration (type-aware rules only)

If you plan to use type-aware rules via `oxlint-tsgolint`, its TypeScript compiler requires a `tsconfig.json` compatible with TypeScript 6. If the project is still on TypeScript 5, migrate it first. See the [TypeScript 6 migration tracking issue](https://github.com/microsoft/TypeScript/issues/62508) for breaking changes and migration steps.

Use [`@andrewbranch/ts5to6`](https://github.com/andrewbranch/ts5to6) to automate the two main breaking tsconfig changes:

| Flag | What it does |
| --- | --- |
| `--fixBaseUrl` | Removes deprecated `baseUrl` (which existed solely to enable `paths`) and updates path mappings accordingly |
| `--fixRootDir` | Explicitly sets `rootDir` to preserve TypeScript 5.x inference behavior under TypeScript 6's changed defaults |

```bash
npx @andrewbranch/ts5to6 --fixBaseUrl .
npx @andrewbranch/ts5to6 --fixRootDir .
```

Run whichever fixes apply to the project — both can be run independently.

---

## Step 1: Run Automated Migration

Run the migration tool in the project root:

```bash
npx @oxlint/migrate
```

This reads your ESLint flat config (`eslint.config.js` for example) and generates a `.oxlintrc.json` file from it. It will find your ESLint config file automatically in most cases.

See options below for more info.

### Key Options

| Option                      | Description                                                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `--type-aware`              | Include type-aware rules from `@typescript-eslint` (will require the `oxlint-tsgolint` package to be installed after migrating) |
| `--with-nursery`            | Include experimental rules still under development, may not be fully stable or consistent with ESLint equivalents               |
| `--js-plugins [bool]`       | Enable/disable ESLint plugin migration via `jsPlugins` (default: enabled)                                                       |
| `--details`                 | List rules that could not be migrated                                                                                           |
| `--replace-eslint-comments` | Convert all `// eslint-disable` comments to `// oxlint-disable`                                                                 |
| `--output-file <file>`      | Specify a different output path (default: `.oxlintrc.json`)                                                                     |

If your ESLint config is not at the default location, pass the path explicitly:

```bash
npx @oxlint/migrate ./path/to/eslint.config.js
```

> **Type-aware rules**: If the source ESLint config used `@typescript-eslint/recommended-type-checked`, `strict-type-checked`, or any other type-aware rules, add `"typeAware": true` to the `options` section of `.oxlintrc.json` to preserve that coverage. Without it, all type-aware rules are silently skipped. Install `oxlint-tsgolint` after migrating (see Step 3). If you need to skip type-aware rules in specific invocations for speed (e.g., a pre-commit hook), omit `typeAware` from the config and pass `--type-aware` on the CLI only where full type checking is wanted.

## Step 2: Review Generated Config

After migration, review the generated `.oxlintrc.json`.

### Plugin Mapping

The migration tool automatically maps ESLint plugins to oxlint's built-in equivalents. The following table is for reference when reviewing the generated config:

| ESLint Plugin                                       | Oxlint Plugin Name |
| --------------------------------------------------- | ------------------ |
| `@typescript-eslint/eslint-plugin`                  | `typescript`       |
| `eslint-plugin-react` / `eslint-plugin-react-hooks` | `react`            |
| `eslint-plugin-import` / `eslint-plugin-import-x`   | `import`           |
| `eslint-plugin-unicorn`                             | `unicorn`          |
| `eslint-plugin-jsx-a11y`                            | `jsx-a11y`         |
| `eslint-plugin-react-perf`                          | `react-perf`       |
| `eslint-plugin-promise`                             | `promise`          |
| `eslint-plugin-jest`                                | `jest`             |
| `@vitest/eslint-plugin`                             | `vitest`           |
| `eslint-plugin-jsdoc`                               | `jsdoc`            |
| `eslint-plugin-next`                                | `nextjs`           |
| `eslint-plugin-node`                                | `node`             |
| `eslint-plugin-vue`                                 | `vue`              |

Default plugins (enabled when `plugins` field is omitted): `unicorn`, `typescript`, `oxc`.
Setting the `plugins` array explicitly overrides these defaults.

ESLint core rules are usable in oxlint without needing to configure a plugin in the config file.

### Rule Categories

Oxlint groups rules into categories for bulk configuration, though only `correctness` is enabled by default:

```json
{
  "categories": {
    "correctness": "error",
    "suspicious": "warn"
  }
}
```

Available categories: `correctness` (default: enabled), `suspicious`, `pedantic`, `perf`, `style`, `restriction`, `nursery`.

Individual rule settings in `rules` override category settings.

`@oxlint/migrate` will turn `correctness` off to avoid enabling additional rules that weren't enabled by your ESLint config. You can choose to enable additional categories after migration if desired.

### Check Unmigrated Rules

Run with `--details` to see which ESLint rules could not be migrated:

```bash
npx @oxlint/migrate --details
```

Review the output and decide whether to keep ESLint for those rules or not. Some rules may be mentioned in the output from `--details` as having equivalents in oxlint that were not automatically mapped by the migration tool. In those cases, consider enabling the equivalent oxlint rule manually after migration.

## Step 3: Install Oxlint

Install the core oxlint package (use `yarn install`, `pnpm install`, `vp install`, `bun install`, etc. depending on your package manager):

```bash
npm install -D oxlint
```

If you set `"typeAware": true` in `.oxlintrc.json`, you must also install `oxlint-tsgolint` — it provides the TypeScript compiler required for type-aware rules:

```bash
npm install -D oxlint-tsgolint
```

> **Diagnosing false positives**: If type-aware rules produce unexpected errors, run with `--type-check` to surface TypeScript compiler diagnostics alongside lint results. Unresolved imports or globals cause values to be typed as `any`, which triggers rules like `typescript/no-unsafe-call`. Resolve the underlying import/type issue before adding disable comments.

No other packages besides the above are needed by default, though you will need to keep/install any additional ESLint plugins that were migrated into `jsPlugins`. Do not add `@oxlint/migrate` to the package.json, it is meant for one-off usage.

## Step 4: Handle Unsupported Features

Some features require manual attention:

- Local plugins (relative path imports): Must be migrated manually to `jsPlugins`
- `eslint-plugin-prettier`: Supported, but very slow. It is recommended to use [oxfmt](https://oxc.rs/docs/guide/usage/formatter) instead, or switch to `prettier --check` as a separate step alongside oxlint.
- `settings` in override configs: Oxlint does not support `settings` inside `overrides` blocks.
- ESLint v9+ plugins: Not all work with oxlint's JS Plugins API, but the majority will.

### React Projects: JSX Transform and Version Setting

If your project uses the **new JSX transform** (`"jsx": "react-jsx"` in tsconfig), explicitly disable `react/react-in-jsx-scope` — the rule requires `import React` in every JSX file, which the new transform makes unnecessary. The migration tool does not disable this automatically.

Additionally, set the React version in `settings` so that version-dependent rules (e.g., `react/no-deprecated`) can evaluate correctly:

```json
{
  "settings": {
    "react": { "version": "19.1.0" }
  },
  "rules": {
    "react/react-in-jsx-scope": "off"
  }
}
```

### Local Plugins

If you have any custom ESLint rules in the project repo itself, you can migrate them manually after running the migration tool by adding them to the `jsPlugins` field in `.oxlintrc.json`:

```json
{
  "jsPlugins": ["./path/to/my-plugin.js"],
  "rules": {
    "local-plugin/rule-name": "error"
  }
}
```

### External ESLint Plugins

For ESLint plugins without a built-in oxlint equivalent, use the `jsPlugins` field to load them:

```json
{
  "jsPlugins": ["eslint-plugin-custom"],
  "rules": {
    "custom/my-rule": "warn"
  }
}
```

### ESLint Core Rules Not Yet in Oxlint

Some ESLint built-in rules are not yet natively implemented in oxlint (e.g., `no-restricted-syntax`). Rather than keeping a separate ESLint process for these, use [`oxlint-plugin-eslint`](https://oxc.rs/blog/2026-03-11-oxlint-js-plugins-alpha#migrating-from-eslint) — a JS plugin that bundles all ESLint core rules for use inside oxlint.

Install it:

```bash
npm install -D oxlint-plugin-eslint
```

Then reference it in `.oxlintrc.json` using the `eslint-js/` namespace:

```json
{
  "jsPlugins": ["oxlint-plugin-eslint"],
  "rules": {
    "eslint-js/no-restricted-syntax": [
      "error",
      {
        "selector": "TSEnumDeclaration",
        "message": "Don't declare enums, use unions or lookup objects instead."
      }
    ]
  }
}
```

This avoids maintaining a parallel ESLint config and keeps all linting in one tool.

## Step 5: Update CI and Scripts

Replace ESLint commands with oxlint. Path arguments are optional; oxlint defaults to the current working directory.

```bash
# Before
npx eslint src/
npx eslint --fix src/

# After
npx oxlint src/
npx oxlint --fix src/
```

### Common CLI Options

| ESLint                    | oxlint equivalent                              |
| ------------------------- | ---------------------------------------------- |
| `eslint .`                | `oxlint` (default: lints the cwd)              |
| `eslint src/`             | `oxlint src/`                                  |
| `eslint --fix`            | `oxlint --fix`                                 |
| `eslint --max-warnings 0` | `oxlint --deny-warnings` or `--max-warnings 0` |
| `eslint --format json`    | `oxlint --format json`                         |

Additional oxlint options:

- `--tsconfig <path>`: Specify tsconfig.json path, likely unnecessary unless you have a non-standard name for `tsconfig.json`.

## Step 6: Finalize

### Replace eslint-disable Comments

Bulk-convert all `// eslint-disable` and `// eslint-disable-next-line` comments to their `oxlint-disable` equivalents:

```bash
npx @oxlint/migrate --replace-eslint-comments
```

### Clean Up Remaining ESLint References

Search the entire project for leftover `eslint` strings:

```bash
grep -ri "eslint" . --exclude-dir=node_modules --exclude-dir=.git
```

For each match, decide whether to replace or remove it:

| What | Action |
| --- | --- |
| `package.json` scripts | Replace with `oxlint` equivalent or remove |
| `package.json` devDependencies | Remove all `eslint*` packages if no longer needed |
| CI config files | Replace `eslint` invocations with `oxlint` |
| `.eslintignore` | Migrate patterns into `ignorePatterns` in `.oxlintrc.json`, then delete the file |
| `eslint.config.js` / `.eslintrc*` | Delete if ESLint is fully removed |
| `// eslint-disable` comments | Should be gone after the `--replace-eslint-comments` step above; fix any remaining ones manually |
| Documentation / README | Update to mention oxlint instead of ESLint |

## Tips

- List available rules: Run `npx oxlint --rules` to see all supported rules, or refer to the [rule documentation](https://oxc.rs/docs/guide/usage/linter/rules.html).
- Schema support: Add `"$schema": "./node_modules/oxlint/configuration_schema.json"` to `.oxlintrc.json` for editor autocompletion if the migration tool didn't do it automatically.
- Output formats: `default`, `stylish`, `json`, `github`, `gitlab`, `junit`, `checkstyle`, `unix`
- Ignore files: `.eslintignore` is supported by oxlint if you have it, but it's recommended to move any ignore patterns into the `ignorePatterns` field in `.oxlintrc.json` for consistency and simplicity. All files and paths ignored via a `.gitignore` file will be ignored by oxlint by default as well.
- If you ran the migration tool multiple times, remove the `.oxlintrc.json.bak` backup file created by the migration tool once you've finished migrating.
- If you are not using any JS Plugins and have replaced your ESLint configuration, you can remove all ESLint packages from your project dependencies.
- Ensure your editor is configured to use oxlint instead of ESLint for linting and error reporting. You may want to install the Oxc extension for your preferred editor. See https://oxc.rs/docs/guide/usage/linter/editors.html for more details.

## References

- [CLI Reference](https://oxc.rs/docs/guide/usage/linter/cli.html)
- [Config File Reference](https://oxc.rs/docs/guide/usage/linter/config-file-reference.html)
- [Complete Oxlint rule list and docs](https://oxc.rs/docs/guide/usage/linter/rules.html)
