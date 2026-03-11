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

## Adding a plugin to conformance tests

- Add code to `init.sh` to clone the plugin's repo.
- Add a file to `src/groups` directory (copy pattern used for other plugins).
- Add the group to `src/groups/index.ts`.
