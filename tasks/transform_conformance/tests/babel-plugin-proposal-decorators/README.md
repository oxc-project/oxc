# Standard decorator execution tests

[`upstream/decorator-tests.ts`](./upstream/decorator-tests.ts) is an unmodified
copy of [`scripts/decorator-tests.ts` from esbuild](https://github.com/evanw/esbuild/blob/f6058f8364fe7ab91ca57a83e02577ed74c9cae4/scripts/decorator-tests.ts),
revision `f6058f8364fe7ab91ca57a83e02577ed74c9cae4`.

Esbuild is copyright (c) 2020 Evan Wallace and distributed under the MIT license.
The original, complete [license](./upstream/LICENSE.md) is retained beside the
upstream source.

On `--exec`, the conformance runner parses the upstream `tests` object and extracts
each named arrow callback by its AST source span. It generates individual
`exec.ts` files under the gitignored `tasks/transform_conformance/fixtures/sources`
directory, then processes them through the existing `TestCase` transformer and
Vitest runner. The metadata polyfill, assertion helpers, and callback bodies are
copied from the upstream source. Each callback is awaited, and its recorded
assertion failures fail the Vitest module. One failed case does not prevent the
other cases from executing.

All 147 upstream cases are included, including the two asynchronous decorator
expression cases. The runner checks the case count and rejects colliding fixture
names or unsupported property shapes to prevent silently losing tests.

The `esbuild` and `esbuild-native` configuration directories contain only
`options.json`. Both strip TypeScript and use the `2023-11` decorator proposal.
`esbuild` also enables class-property, private-method, and static-block lowering;
`esbuild-native` leaves those plugins disabled. Each case runs in both
configurations, for 294 executions with stable per-case names in `oxc_exec.snap.md`.

Run the suite with:

```sh
cargo run -p oxc_transform_conformance -- --exec --filter esbuild
```

The filter also accepts individual case names, such as
`esbuild-native/decorator-list-evaluation-await-class-statement`.
