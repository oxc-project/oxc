# Transformation Conformance Test Runner

## Execution

This test runner uses the transformation plugin test suite from the Babel repository.

Additional tests are in the [tests](./tests/) directory.

The failing test cases are listed in:

- [babel.snap.md](./snapshots/babel.snap.md)
- [oxc.snap.md](./snapshots/oxc.snap.md)

To get started, run

```bash
cargo run -p oxc_transform_conformance
```

or watch for changes

```bash
just watch 'cargo run -p oxc_transform_conformance'
```

## Options

### --filter

To filter for a specific test case, apply the `--filter path` option, e.g.

```bash
cargo run -p oxc_transform_conformance -- --filter react/arrow-functions
```

### --exec

The Babel test suite contains many `exec.js` files, which need to be executed by a runtime.

These tests are executed with Node.js and Vitest. Apply the `--exec` flag:

```bash
cargo run -p oxc_transform_conformance -- --exec
```

This is also available as:

```bash
just test-transform
```

The failing test cases are listed in:

- [babel_exec.snap.md](./snapshots/babel_exec.snap.md)
- [oxc_exec.snap.md](./snapshots/oxc_exec.snap.md)
