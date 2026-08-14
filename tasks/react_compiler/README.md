# React Compiler comparison

Recursively compares the published `babel-plugin-react-compiler` pipeline with
the local `oxc-transform-react` NAPI package for every `.jsx` and `.tsx` file in
a directory. Both pipelines run React Compiler first, remove TypeScript syntax,
and lower JSX with the automatic runtime.

Both React Compiler implementations receive the same explicit options. The
comparison uses the v1 defaults for ESLint suppression rules and disables
exhaustive manual-memo dependency validation, which was not enabled by default
in v1. The scanned directory is passed as `sources` so dependency directories
are handled consistently by both implementations. Babel's TypeScript transform
also enables `allowDeclareFields` so uninitialized class fields use the same
emit behavior as Oxc.

Before comparing, the Babel output is parsed and printed by `oxc-transform`
with no transforms enabled. This ensures both outputs use Oxc code generation
and removes printer-only differences.

Build the native bindings and pass a directory to scan:

```sh
pnpm --dir napi/transform build-test
pnpm --dir napi/transform-react build-test
pnpm --filter react_compiler compare ./path/to/source
```

The command prints each differing path relative to the scanned directory, one
per line. Transform diagnostics and the final summary are written to stderr.
It exits with status 1 when outputs differ or either transform fails.
