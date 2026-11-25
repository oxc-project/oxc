# Sprout

Runtime conformance checks for Oxc's React Compiler fixtures. Run after installing
workspace dependencies:

```sh
just sprout
just sprout --filter sprout/
pnpm --dir tasks/sprout test-harness
```

The Rust bridge compiles the fixture sources using the same pragma parser and
type providers as the compiler snapshot suite. JavaScript workers lower the
original and compiled sources with the same Babel transforms, render them with
React, and compare output, exceptions, and console logs. Each lane has a separate
React environment. Each render runs in async `act`, and HTML is captured after
promise-driven React updates settle. Final unmounts and their promise
continuations finish before console capture ends. Timer callbacks run on a
fixture-local virtual clock starting at zero; finite timer work drains before
capturing each render and after unmounting. Non-terminating timer work and unhandled promise rejections
fail their own fixture. Evaluation has a ten-second timeout. A mismatch, unexpected compiler/evaluator error, or an
empty runnable selection fails the command.

Fixtures are discovered recursively under `crates/oxc_react_compiler/fixtures`.
Every executable fixture must export a usable `FIXTURE_ENTRYPOINT`. They support
`fn`, `params`, and optional `sequentialRenders`, following upstream Sprout.
Function entrypoints may return values or JSX. React classes and `memo`,
`forwardRef`, and `lazy` objects are rendered as components on both rendering paths.
Diagnostic and lint-only fixtures remain covered by the Rust snapshot suite.
Missing entrypoints and `TodoAdd` placeholders fail the command. Snapshot-only
fixtures need concrete reasons in `src/exclusions.ts`, such as an intentional
infinite loop or an unavailable runtime dependency. Compiler failures have no
expected-failure baseline.

Use `sequentialRenders` to cover unchanged inputs, changed inputs, and a return to
earlier inputs on the same mounted component. Each item supplies one props
argument. For functions with several arguments, use an uncompiled wrapper that
spreads an argument array:

```js
const first = [1, 2];
const second = [3, 4];
export const FIXTURE_ENTRYPOINT = {
  fn: (args) => {
    "use no memo";
    return Component(...args);
  },
  params: [first],
  sequentialRenders: [first, first, second, second, first],
};
```

Arrays containing JSX are rendered as React children. The value serializer omits
React's mutable internal state and normalizes function descriptions so parameter
lowering does not create formatting differences. Fixtures can still inspect
function `.length` explicitly. The upstream `@disableNonForgetInSprout` directive
retains its compiled-only behavior.

Ported from `react/react` revision
`cb7c59ecdb7b620ed3d6ce0bf669d7502daedf13`:

- `compiler/packages/snap/src/sprout/` supplies the evaluator and runtime helpers.
- `compiler/packages/snap/src/compiler.ts` supplies the evaluator transform setup.
- `compiler/packages/react-compiler-runtime/src/index.ts` supplies runtime support.

The MIT license and Meta copyright notices are retained. FBT's React element
symbol is adapted in memory; dependency files are not rewritten during install.
