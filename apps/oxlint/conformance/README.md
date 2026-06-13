# Oxlint JS plugins conformance tests

Conformance tests run:

- All ESLint's tests for its built-in rules as Oxlint JS plugins.
- Tests from various ESLint plugins, running in Oxlint.

They do this by substituting ESLint's `RuleTester` for Oxlint's version, and then `require()`ing the test files
to run the tests.

The results are saved in `snapshots` directory.

## Setup

Build Oxlint in conformance mode:

```sh
cd apps/oxlint
pnpm install
pnpm run build-conformance
```

Initialize plugin submodules:

```sh
pnpm run init-conformance
```

## Run conformance tests

```sh
pnpm run conformance
```

## Updating a pinned repo version

All upstream repo URLs, commit SHAs, and version labels are stored in `repos.json`.
Both `init.sh` and the TypeScript test groups read from this single file.

To update a repo to a newer version:

1. Edit `repos.json` — update the `commitSha` and `version` for the repo.
2. Re-initialize submodules: `pnpm run init-conformance`
3. Re-run the conformance tests: `pnpm run conformance`
4. Commit the updated `repos.json` and snapshot files.

## Adding a plugin to conformance tests

- Add an entry to `repos.json` with the repo's URL, commit SHA, and version.
- Add code to `init.sh` to clone and set up the plugin's repo.
- Add a file to `src/groups` directory (copy pattern used for other plugins).
- Add the group to `src/groups/index.ts`.

### Debugging

While working on adding a new plugin, you may find it useful to filter the test run, by editing
`conformance/src/filter.ts`. You can filter by plugin name, rule, or the code of a specific test case.
