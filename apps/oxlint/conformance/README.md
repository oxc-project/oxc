# Oxlint JS plugins conformance tests

Conformance tests run all ESLint's tests for its built-in rules as Oxlint JS plugins.

They do this by substituting ESLint's `RuleTester` for Oxlint's version, and then `require()`ing the ESLint test files
to run the tests.

The results are saved in `conformance/snapshot.md`.

## Setup

Build Oxlint in debug mode:

```sh
cd apps/oxlint
pnpm run build-test
```

Initialize ESLint submodule:

```sh
pnpm run init-conformance
```

## Run conformance tests

```sh
pnpm run conformance
```
