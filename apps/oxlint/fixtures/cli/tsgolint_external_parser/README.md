# `tsgolint` over files routed to an external parser

A file oxlint only knows how to lint because a config override gave it a
`languageOptions.parser` — here Ember's `.gts` — still reaches `tsgolint` for type-aware
linting, under its original path, and diagnostics land at offsets in that original file
rather than in the mapper's generated TypeScript.

`typescript-go` does the mapping, via the `contentMappers` entry in `tsconfig.json`
pointing at [`ember-content-mapper`](https://www.npmjs.com/package/ember-content-mapper)
(a TypeScript 7 content mapper wrapping Glint). oxlint sends the `.gts` path unrewritten.

| file | finding | needs |
| --- | --- | --- |
| `files/widget.gts` | `no-unnecessary-condition` on `this.always` inside `<template>` | `--type-aware` |
| `files/counter.gts` | TS2551 on `this.cuont` inside `<template>` | `--type-aware --type-check` |
| `files/plain.ts` | `no-unnecessary-condition` — control, a natively lintable extension | `--type-aware` |

## Running it

Not part of any automated suite: it needs an `npm install` here, and a `tsgolint` built
with content-mapper support. The `oxlint-tsgolint` version this repo pins (0.24.0)
predates that and panics on a `.gts` with `Unknown script kind for file`.

```sh
cd apps/oxlint/fixtures/cli/tsgolint_external_parser
npm install
OXLINT_TSGOLINT_PATH=/path/to/tsgolint \
  node ../../../dist/cli.js --type-aware --type-check files
```

Expected: three findings, each anchored in the original source —
`files/counter.gts:9:18` on `cuont`, `files/plain.ts:3:18`, `files/widget.gts:10:11`
on `this.always`.

Promote this to a snapshot test in `apps/oxlint/src/lint.rs` once a released
`oxlint-tsgolint` carries content-mapper support, so CI has a `tsgolint` that can run it.
