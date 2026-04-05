# `oxfmt --migrate prettier` compatibility notes

This file documents the current migration contract for Prettier configs, with special attention to Svelte projects.

## Lossless or near-lossless cases

These cases are expected to round-trip into `.oxfmtrc.json` without dropping the plugin intent:

- string plugin specs such as `"prettier-plugin-svelte"`
- override-scoped string plugin specs in JSON, YAML, and JS Prettier configs
- imported plugin objects from JS configs when Oxfmt can recover a stable plugin spec from:
  - the original import / require path
  - plugin metadata such as `name`, `packageName`, `meta.name`, or `meta.packageName`
  - the structural `prettier-plugin-svelte` fallback

For Svelte configs, that means all of the common forms below should migrate cleanly:

- `plugins: ["prettier-plugin-svelte"]`
- imported `prettier-plugin-svelte` objects in `prettier.config.js|mjs|cjs`
- override-scoped Svelte plugin usage in JSON / YAML / JS configs

## Converted or defaulted cases

Some Prettier concepts are intentionally migrated into the closest Oxfmt representation instead of being copied literally.

- `prettier-plugin-packagejson` becomes `sortPackageJson`
- if `printWidth` is omitted, migration writes `printWidth: 80` so the generated Oxfmt config keeps Prettier's default behavior
- Tailwind plugin options move into `sortTailwindcss`

These are not byte-for-byte copies, but they preserve the practical formatting behavior Oxfmt can express.

## Warning-only / partial-support cases

Some settings are copied, but migration emits a warning because Oxfmt support is still partial.

- `embeddedLanguageFormatting` values other than `"off"`
- `experimentalTernaries`
- `experimentalOperatorPosition`

Those warnings are there to make it clear the generated config may still need manual review.

## Skipped cases

Some inputs cannot be represented safely in `.oxfmtrc.json`, so migration leaves them out and prints a warning.

- custom plugin objects that do not expose a stable package spec Oxfmt can preserve
- `endOfLine: "auto"`

The most common skipped case is a JS config that constructs or mutates a plugin object inline instead of referencing it via a stable package spec.

## Practical guidance for Svelte projects

If you want the most reliable migration result today:

- prefer `plugins: ["prettier-plugin-svelte"]` or an imported `prettier-plugin-svelte` object from a normal package import
- keep override-scoped Svelte plugin config explicit for `*.svelte`
- expect a short manual review whenever your Prettier config uses experimental options or inline custom plugin objects
