---
name: migrate-oxfmt
description: Guide for migrating a project from Prettier or Biome to Oxfmt. Use when asked to migrate, convert, or switch a JavaScript/TypeScript project's formatter from Prettier or Biome to Oxfmt.
---

This skill guides you through migrating a JavaScript/TypeScript project from Prettier or Biome to [Oxfmt](https://oxc.rs/docs/guide/usage/formatter).

## Overview

Oxfmt is a high-performance, Prettier-compatible code formatter. Most Prettier options are supported directly.

An automated migration tool is built into oxfmt, supporting both Prettier and Biome as migration sources.

## Step 1: Run Automated Migration

First, decide whether the automated tool fits:

- **Static root config** (JSON/YAML, or a JS/TS config that just exports a plain object): use `--migrate` below.
- **Dynamic JS/TS config** (logic such as environment branches or computed values): `--migrate` writes only a resolved snapshot of the values — the logic does not survive. Migrate manually instead, porting the logic to `oxfmt.config.ts` with the option mappings in this guide.
- **Nested per-directory configs**: `--migrate` only handles the config found from the current directory. Migrate the nested ones manually (see "Nested Config" in Step 2).

### From Prettier

```bash
npx oxfmt@latest --migrate prettier
```

This will:

- Find and read your Prettier config (any format Prettier supports)
- Create `.oxfmtrc.json` with migrated options
- Migrate `.prettierignore` patterns to `ignorePatterns`
- Migrate `prettier-plugin-tailwindcss` options to `sortTailwindcss`
- Migrate `prettier-plugin-svelte` options to `svelte` (`svelteSortOrder` → `sortOrder`, `svelteAllowShorthand` → `allowShorthand`, `svelteIndentScriptAndStyle` → `indentScriptAndStyle`)
- Detect `prettier-plugin-packagejson` and enable `sortPackageJson`

### From Biome

```bash
npx oxfmt@latest --migrate biome
```

This will:

- Find and read `biome.json` or `biome.jsonc`
- Create `.oxfmtrc.json` with migrated options
- Migrate negated patterns from `files.includes` to `ignorePatterns`
- Map Biome's two-level config (`formatter.*` and `javascript.formatter.*`) to oxfmt options

Biome option mapping:

| Biome                                                       | oxfmt                             |
| ----------------------------------------------------------- | --------------------------------- |
| `formatter.indentStyle` (`"tab"`/`"space"`)                 | `useTabs` (`true`/`false`)        |
| `formatter.indentWidth`                                     | `tabWidth`                        |
| `formatter.lineWidth`                                       | `printWidth`                      |
| `javascript.formatter.quoteStyle`                           | `singleQuote`                     |
| `javascript.formatter.jsxQuoteStyle`                        | `jsxSingleQuote`                  |
| `javascript.formatter.quoteProperties` (`"asNeeded"`)       | `quoteProps` (`"as-needed"`)      |
| `javascript.formatter.trailingCommas`                       | `trailingComma`                   |
| `javascript.formatter.semicolons` (`"always"`/`"asNeeded"`) | `semi` (`true`/`false`)           |
| `javascript.formatter.arrowParentheses` (`"asNeeded"`)      | `arrowParens` (`"avoid"`)         |
| `javascript.formatter.bracketSameLine`                      | `bracketSameLine`                 |
| `formatter.bracketSpacing`                                  | `bracketSpacing`                  |
| `formatter.attributePosition` (`"multiline"`)               | `singleAttributePerLine` (`true`) |

Notes (Biome):

- For `formatter.*` options in the table, a `javascript.formatter.*` value of the same name takes precedence when present. `bracketSameLine` is read only from `javascript.formatter.bracketSameLine`.
- Options not set in `biome.json` are written to `.oxfmtrc.json` with Biome's default values explicitly (e.g. `printWidth: 80`, `useTabs: true`), so the output preserves Biome's formatting behavior.

Notes (both sources):

- Fails if `.oxfmtrc.json` or `.oxfmtrc.jsonc` already exists. Delete it first if you want to re-run.
- If no source config is found, creates a blank `.oxfmtrc.json` instead.
- `overrides` cannot be auto-migrated for either source and must be converted manually (a warning is printed if detected).

## Step 2: Review Generated Config

After migration, review the generated `.oxfmtrc.json` for these key differences:

### printWidth

Prettier and Biome default is 80, oxfmt default is 100. The migration tool sets `printWidth: 80` if not specified in your source config. Decide whether to keep 80 or adopt 100.

### Unsupported Options (Prettier only)

These Prettier options are skipped during migration:

| Option                          | Status                                           |
| ------------------------------- | ------------------------------------------------ |
| `endOfLine: "auto"`             | Not supported. Use `"lf"` or `"crlf"` explicitly |
| `experimentalTernaries`         | Not supported in JS/TS files yet                 |
| `experimentalOperatorPosition`  | Not supported in JS/TS files yet                 |
| `requirePragma`, `insertPragma` | Not supported                                    |
| `parser`, `filepath`            | Not applicable to oxfmt                          |

Regex values (e.g. `"/^my-/"`) in `tailwindFunctions` / `tailwindAttributes` are also skipped with a warning — oxfmt only supports literal strings there.

### sortPackageJson (Prettier only)

Enabled by default in oxfmt, but the migration tool disables it unless `prettier-plugin-packagejson` was detected. Review whether you want this enabled.

Note: Oxfmt's sorting algorithm differs from `prettier-plugin-packagejson`.

### embeddedLanguageFormatting (Prettier only)

Embedded language formatting (e.g., CSS-in-JS) generally works, but some formatting may differ from Prettier.

### overrides

The `overrides` field cannot be auto-migrated from either Prettier or Biome. Convert manually:

```json
{
  "overrides": [
    {
      "files": ["*.md"],
      "excludeFiles": ["CHANGELOG.md"],
      "options": { "tabWidth": 4 }
    }
  ]
}
```

(`excludeFiles` is optional.)

### Nested Config

Oxfmt supports nested configuration files: a `.oxfmtrc.json` in a subdirectory applies to files under that directory. Pass `--disable-nested-config` to opt out and use only the root config.

However, `--migrate` only migrates the config found from the current directory — it does not walk subdirectories. If the project has per-directory Prettier or Biome configs, you must migrate each one yourself: convert each nested config to a `.oxfmtrc.json` in the same directory manually (using the option mappings in this guide).

### Prettier-Compatible Options

These options transfer directly with the same behavior:
`printWidth`, `tabWidth`, `useTabs`, `semi`, `singleQuote`, `jsxSingleQuote`, `quoteProps`, `trailingComma`, `arrowParens`, `bracketSpacing`, `bracketSameLine`, `objectWrap`, `endOfLine`, `proseWrap`, `htmlWhitespaceSensitivity`, `singleAttributePerLine`, `vueIndentScriptAndStyle`, `embeddedLanguageFormatting`

## Step 3: Configure Oxfmt Extensions

Oxfmt offers features not available in Prettier:

### sortImports

Sort import statements, inspired by `eslint-plugin-perfectionist/sort-imports` (disabled by default):

```json
{
  "sortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
}
```

Other options: `partitionByComment`, `sortSideEffects`, `order`, `ignoreCase`, `internalPattern`, `groups`, `customGroups`.

### sortTailwindcss

Replaces `prettier-plugin-tailwindcss`. Auto-migrated with renamed options:

| Prettier (top-level)         | oxfmt (`sortTailwindcss.*`) |
| ---------------------------- | --------------------------- |
| `tailwindConfig`             | `config`                    |
| `tailwindStylesheet`         | `stylesheet`                |
| `tailwindFunctions`          | `functions`                 |
| `tailwindAttributes`         | `attributes`                |
| `tailwindPreserveWhitespace` | `preserveWhitespace`        |
| `tailwindPreserveDuplicates` | `preserveDuplicates`        |

### Other Extensions

| Option               | Default  | Description                                                                     |
| -------------------- | -------- | ------------------------------------------------------------------------------- |
| `insertFinalNewline` | `true`   | Whether to add a final newline at end of file                                   |
| `sortPackageJson`    | `true`   | Sort `package.json` keys. Set `{ "sortScripts": true }` to also sort scripts    |
| `jsdoc`              | disabled | Format JSDoc comments. Set `true` or an options object for fine-grained control |
| `svelte`             | disabled | Svelte formatting options, replacing `prettier-plugin-svelte` (auto-migrated)   |

## Step 4: Update CI and Scripts

Replace formatter commands with oxfmt:

```bash
# Before (Prettier)
npx prettier --write .
npx prettier --check .

# Before (Biome)
npx biome format --write .
npx biome check .

# After
npx oxfmt@latest
npx oxfmt@latest --check
```

### Common CLI Options

| Prettier / Biome                                | oxfmt                                        |
| ----------------------------------------------- | -------------------------------------------- |
| `prettier --write .` / `biome format --write .` | `oxfmt` (default: cwd, `--write` mode)       |
| `prettier --check .` / `biome check .`          | `oxfmt --check`                              |
| `prettier --list-different .`                   | `oxfmt --list-different`                     |
| `prettier --config path`                        | `oxfmt --config path`                        |
| `prettier --ignore-path .prettierignore`        | `oxfmt --ignore-path .prettierignore`        |
| `cat file \| prettier --stdin-filepath=file.ts` | `cat file \| oxfmt --stdin-filepath=file.ts` |

### File Type Coverage

- JS/TS, JSON/JSONC/JSON5, CSS/SCSS/Less, GraphQL: Formatted natively by oxfmt
- TOML: Formatted natively (via taplo)
- HTML, YAML, Markdown, Vue, Svelte, etc.: Delegated to Prettier internally (when using `npx oxfmt`)

## Tips

- Config file: `.oxfmtrc.jsonc` and `oxfmt.config.ts` are also supported as auto-discovered config file names, in addition to `.oxfmtrc.json`. `--migrate` and `--init` only generate `.oxfmtrc.json`; keep it as-is unless the user explicitly asks for another format or the source config had logic to preserve (see Step 1).
- EditorConfig: Oxfmt reads `.editorconfig` automatically for `useTabs`, `tabWidth`, `endOfLine`, `insertFinalNewline`, `printWidth`, and `singleQuote`. Options in `.oxfmtrc.json` take precedence.
- CI: Use `npx oxfmt@latest --check` to enforce formatting in CI.
- LSP: Run `oxfmt --lsp` for editor integration via Language Server Protocol.
- Schema support: Add `"$schema": "./node_modules/oxfmt/configuration_schema.json"` to `.oxfmtrc.json` for editor autocompletion.
- Init: Run `npx oxfmt@latest --init` to create a default `.oxfmtrc.json` without migration.

## References

- [CLI Reference](https://oxc.rs/docs/guide/usage/formatter/cli.html)
- [Config File Reference](https://oxc.rs/docs/guide/usage/formatter/config-file-reference.html)
- [Unsupported Features](https://oxc.rs/docs/guide/usage/formatter/unsupported-features.html)
