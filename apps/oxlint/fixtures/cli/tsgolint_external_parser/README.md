# `tsgolint` over files routed to an external parser

Checks that a file oxlint only knows how to lint because a config override gave it a
`languageOptions.parser` — here `.gts` — still reaches `tsgolint` for type-aware linting,
under its original path, and that diagnostics land at offsets in that original file.

`files/component.gts` carries two problems:

- a floating promise at module level (a `tsgolint` *rule* diagnostic), and
- `label.toUpperCase` inside `<template>`, where `label` is a `number` (a TypeScript
  *semantic* diagnostic, reported under `--type-check`).

## Status

Not yet wired into `apps/oxlint/src/lint.rs` as a snapshot test. `tsgolint` 0.24.0 — the
version this repo pins — predates `contentMappers`, and panics on a `.gts`:

```
panic: Unknown script kind for file .../files/component.gts
```

The `contentMappers` entry in `tsconfig.json` names a package that does not exist yet.
Once a `tsgolint` with content-mapper support and an Ember content-mapper package are
available, point `OXLINT_TSGOLINT_PATH` at that build, add the mapper package, and
promote this to a real snapshot test.

## Verifying the oxlint half today

`scripts/fake-tsgolint.mjs` stands in for the binary. It records the payload oxlint sends
and replies with one diagnostic per `.gts`, anchored at the offset of `label.toUpperCase`
in the original file — so it exercises path selection, payload construction, diagnostic
decoding and source rendering without needing content-mapper support:

```sh
cd apps/oxlint/fixtures/cli/tsgolint_external_parser
OXLINT_TSGOLINT_PATH=./scripts/fake-tsgolint \
FAKE_TSGOLINT_CAPTURE=/tmp/payload.json \
  node ../../../dist/cli.js --type-aware files
```

Expected: the `.gts` appears in `/tmp/payload.json` under its absolute original path, and
the diagnostic renders at `files/component.gts:12:33`, on `label.toUpperCase`.
