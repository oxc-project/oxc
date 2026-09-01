# `tsgolint` over files routed to an external parser

A file oxlint only knows how to lint because a config override gave it a
`languageOptions.parser` — here Ember's `.gts` — still reaches `tsgolint` for type-aware
linting, under its original path, and diagnostics land at offsets in that original file
rather than in the mapper's generated TypeScript.

`typescript-go` does the mapping, via the `contentMappers` entry in `tsconfig.json`
pointing at [`ember-content-mapper`](https://www.npmjs.com/package/ember-content-mapper)
(a TypeScript 7 content mapper wrapping Glint). oxlint sends the `.gts` path unrewritten.
`@glimmer/component` is a real dependency, not scaffolding: a bare class carrying
`<template>` fails Glint's `HasContext` check.

| file | finding | needs |
| --- | --- | --- |
| `files/widget.gts` | `no-unnecessary-condition` on `this.always` inside `<template>` | `--type-aware` |
| `files/counter.gts` | TS2551 on `this.cuont` inside `<template>` | `--type-aware --type-check` |
| `files/fixable.gts` | `no-unnecessary-type-assertion` in the script section | `--type-aware --fix` |
| `files/plain.ts` | `no-unnecessary-condition` — control, a natively lintable extension | `--type-aware` |

Fixes only survive where the mapping is verbatim, so `--fix` rewrites `fixable.gts`'s
script section and never touches `<template>`. A `.gts` diagnostic arriving with no fix
attached is expected.

Expect fewer findings than the rules would suggest: diagnostics anchored on Glint's
`__glintDSL__` scaffolding are dropped upstream.

## Running it

Not part of any automated suite. It needs an `npm install` here, Node >= 22.21.1 on
`PATH` at lint time (the mapper's `exec` is a bare `node`), and a `tsgolint` built with
content-mapper support — the `oxlint-tsgolint` this repo pins (0.24.0) predates it.

```sh
cd apps/oxlint/fixtures/cli/tsgolint_external_parser
npm install
OXLINT_TSGOLINT_PATH=/path/to/tsgolint \
  node ../../../dist/cli.js --type-aware --type-check files
```

Expected: `files/counter.gts:9:18` on `cuont`, `files/plain.ts:3:18`,
`files/widget.gts:10:11` on `this.always` — each anchored in the original source.

`OXLINT_TSGOLINT_DISABLE_CONTENT_MAPPERS=true` turns mapping off.

## When the mapper is missing or misspelled

Both land as `typescript(unsupported-file-extension)` on line 1 of each `.gts`. Whether
you also get the tsconfig error that explains *why* depends on the config's other files:

- A typo'd `package` **with** a natively-parseable file in the same tsconfig also reports
  `typescript(tsconfig-error)` — "The content mapper package '…' could not be resolved" —
  anchored at the offending line in `tsconfig.json`.
- A typo'd `package` in a `.gts`-only tsconfig reports nothing extra. An unresolvable
  mapper unregisters its extensions, so no file matches that config, so no program is
  built and the tsconfig error never surfaces. Indistinguishable from having no
  `contentMappers` entry at all.

Real Ember apps normally have `.ts` in the same tsconfig, so the diagnosable case is the
common one.

Promote this to a snapshot test in `apps/oxlint/src/lint.rs` once a released
`oxlint-tsgolint` carries content-mapper support, so CI has a `tsgolint` that can run it.
