# Coding agent guides for `crates/oxc_semantic`

## Testing semantic behavior

Choose the test location based on the behavior being protected. Do not add duplicate coverage if the case is already covered by `just coverage`, an existing semantic fixture, or an upstream-derived fixture.

For OXC-specific scope, symbol, binding, or reference regressions, add a snapshot fixture under:

```text
crates/oxc_semantic/tests/fixtures/oxc/<area>/<case>.<ext>
```

The semantic snapshot harness records the scope tree, symbols, and references for every `.{js,jsx,ts,tsx}` fixture. Use a short descriptive case name, or an issue-based name when the fixture fixes a tracked issue.

Run semantic snapshot tests with:

```sh
cargo test -p oxc_semantic --test main
```

Review intentional snapshot changes with:

```sh
cargo insta review -p oxc_semantic
```

## Integration tests

Use `crates/oxc_semantic/tests/integration/` when the regression is best asserted through the Rust API rather than a snapshot. This is usually right for exact enum values, module data, CFG shape, class table behavior, or direct `Scoping` API expectations.

Run integration tests with:

```sh
cargo test -p oxc_semantic --test integration
```

For CFG integration tests, enable the `cfg` feature:

```sh
cargo test -p oxc_semantic --test integration --features cfg cfg
```

## Conformance checks

Use `crates/oxc_semantic/tests/conformance/` for broad invariants that should be checked against every semantic fixture, such as reflexive symbol declarations or identifier reference wiring.

Do not add one-off bug regressions there unless the check is intentionally generalized across all fixtures.

## Coverage fixtures

Use `tasks/coverage/misc/pass/` or `tasks/coverage/misc/fail/` only when the bug is best represented as a parser/semantic coverage case, especially for diagnostics surfaced by `cargo coverage semantic`.

Run a focused coverage case with:

```sh
cargo coverage semantic --filter misc/fail/<case>.<ext>
```

Run all semantic coverage with:

```sh
cargo coverage semantic
```
