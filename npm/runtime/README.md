# Oxc's modular runtime helpers

Vendored from the complete published `@babel/runtime@8.0.8` package, including all 95 CommonJS/ESM helper pairs and Babel's MIT license. All credits to the Babel team.

[upstream.json](upstream.json) records the upstream version, SHA-256 of every upstream helper file, and the local patches. The tests check these hashes so unrecorded edits cannot silently diverge from Babel.

## Differences from Babel

- `setFunctionName.js` and `esm/setFunctionName.js` preserve `"[]"` for `Symbol("")`, matching native computed method names. Babel 8.0.8 returns an empty name. These files are unchanged from Oxc's existing helpers and are the only patches to the upstream helper tree.
- Oxc retains its three TypeScript legacy decorator helpers: `decorate`, `decorateParam`, and `decorateMetadata`. They are adapted from TypeScript and are absent from Babel 8.
- The 26 older Babel helpers removed in Babel 8 remain available for compatibility with previously generated code. Their names are listed in `upstream.json`; they are outside the Babel 8 source snapshot.
- Package identity, Oxc's version and Node.js requirements, and the `src/` layout remain Oxc-specific. Babel's current conditional exports and `sideEffects: false` are synchronized; explicit `helpers/esm/*` and older package exports remain supported.

## Updating

Download and extract a specific published version of `@babel/runtime`, then run from this directory:

```sh
node scripts/sync.mjs /path/to/extracted/babel-runtime
```

The script copies every unpatched upstream helper and the license, updates exports and the hash manifest, and preserves compatibility helpers and recorded local patches. Before writing, it checks that each patched helper still matches its recorded local hash and that its upstream source is unchanged. If upstream changes a patched helper, the script stops so its local changes can be reviewed against the new version.

Run the import, integrity, and runtime behavior tests with:

```sh
node --run test
```
