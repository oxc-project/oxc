# New Oxlint Rules & Infrastructure — Branch Summary

## Overview

This branch adds **39 new linter rules** under the `oxc` plugin, introduces **JSON file linting support**, and adds **plugin aliasing** for ESLint config compatibility.

---

## Infrastructure Changes

### JSON File Linting Support

- New span-preserving JSON parser (`json_parser.rs`) that maintains exact source positions for all elements
- JSON files added to partial loader — oxlint can now lint `.json` files directly
- JSON files only run `oxc` plugin rules (ESLint, TypeScript, etc. plugins are skipped)

### Full Source Text API

- New `full_source_text()` API on `LintContext` for rules operating on embedded/non-JS files (Vue, Astro, Svelte, JSON)
- `RuleFixer` gains `full_source_range()` and `replace_full_source_range()` for applying fixes to full file content
- Enables rules that need access to the original file content, not just the parsed JS section

### Plugin Aliasing

Maps external ESLint plugin names to oxc's native implementations for seamless config migration:

| External Plugin                          | Mapped To                                          |
| ---------------------------------------- | -------------------------------------------------- |
| `react-refresh/only-export-components`   | `react/only-export-components`                     |
| `react-dom/no-dangerously-set-innerhtml` | `react/no-danger`                                  |
| `i18next/no-literal-string`              | `oxc/no-literal-string`                            |
| `package-json/*` (15 rules)              | `oxc/package-json-*`                               |
| `json/*`                                 | `oxc/valid-json`                                   |
| `i18n-json/*` (4 rules)                  | `oxc/identical-keys`, `oxc/sorted-json-keys`, etc. |
| `optimize-regex/optimize-regex`          | `oxc/optimize-regex`                               |

### New Dependency

- `spdx` crate (v0.13.4) for SPDX license expression validation in `package-json-valid-license`

---

## New Rules (39 total)

All rules are under the **oxc** plugin in **nursery** category.

### Package.json Validation (16 rules)

| Rule                                       | Description                                    |
| ------------------------------------------ | ---------------------------------------------- |
| `package-json-no-empty-fields`             | Warns about empty string fields                |
| `package-json-no-redundant-publish-config` | Detects redundant publishConfig settings       |
| `package-json-order-properties`            | Enforces property ordering                     |
| `package-json-repository-shorthand`        | Validates repository field shorthand format    |
| `package-json-require-type`                | Ensures "type" field is present (ESM/CJS)      |
| `package-json-require-version`             | Requires version field                         |
| `package-json-sort-collections`            | Sorts dependencies, devDependencies, etc.      |
| `package-json-valid-bin`                   | Validates the "bin" field                      |
| `package-json-valid-description`           | Validates description field                    |
| `package-json-valid-license`               | Validates license field using SPDX expressions |
| `package-json-valid-man`                   | Validates "man" field                          |
| `package-json-valid-name`                  | Validates npm package name format              |
| `package-json-valid-private`               | Validates "private" field boolean type         |
| `package-json-valid-repository`            | Validates repository field structure           |
| `package-json-valid-type`                  | Validates type field (module/commonjs)         |
| `package-json-valid-version`               | Validates semantic version format              |

### JSON & i18n (4 rules)

| Rule                   | Description                                                   |
| ---------------------- | ------------------------------------------------------------- |
| `valid-json`           | Validates JSON file syntax                                    |
| `sorted-json-keys`     | Enforces alphabetical key ordering in JSON                    |
| `identical-keys`       | Validates i18n files have consistent keys across translations |
| `valid-message-syntax` | Validates ICU message syntax in i18n files                    |

### Code Organization (3 rules)

| Rule                      | Description                                      |
| ------------------------- | ------------------------------------------------ |
| `avoid-barrel-files`      | Warns about barrel files (re-export index files) |
| `avoid-re-export-all`     | Detects problematic `export *` patterns          |
| `boundaries-dependencies` | Enforces architectural dependency boundaries     |

### Naming & Format (7 rules)

| Rule                         | Description                                |
| ---------------------------- | ------------------------------------------ |
| `filename-naming-convention` | Enforces naming patterns for files         |
| `folder-naming-convention`   | Enforces naming patterns for directories   |
| `no-block-in-inline`         | Prevents block elements inside inline JSX  |
| `no-inline-type-annotations` | Prevents inline type annotations           |
| `no-literal-string`          | Detects hardcoded strings (for i18n)       |
| `no-secrets`                 | Detects potential secrets/API keys in code |
| `no-unknown`                 | Flags unknown file types/extensions        |

### Sorting & Style (5 rules)

| Rule                      | Description                              |
| ------------------------- | ---------------------------------------- |
| `sort-interfaces`         | Sorts TypeScript interface members       |
| `sort-switch-case`        | Sorts switch cases                       |
| `sort-union-types`        | Sorts union type members                 |
| `optimize-regex`          | Optimizes regular expressions            |
| `detect-object-injection` | Detects unsafe object injection patterns |

---

## Architecture Notes

- **JSON rules use `run_once()`** rather than `run()` since they analyze the entire file structure
- **File type detection**: JSON rules implement `should_run()` to check file extension and whether it's the first sub-host
- **Configurable rules**: Complex rules like `avoid-barrel-files`, `boundaries-dependencies`, and `sort-switch-case` use `JsonSchema`/`Deserialize` for configuration
- **All rules include snapshot tests** via `Tester::new().test_and_snapshot()`

## Testing

```bash
cargo test -p oxc_linter          # 1091 tests pass
cargo clippy -p oxc_linter        # 0 warnings in new rule files
cargo lintgen                     # 785 rule variants generated
```
