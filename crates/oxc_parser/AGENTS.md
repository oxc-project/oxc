# Coding agent guides for `crates/oxc_parser`

## Testing parser behavior

Parser syntax regressions should usually be tested through coverage fixtures, not Rust unit tests in this crate.

Before adding a `misc` fixture, check whether the case is already covered by the normal `just coverage` suites. Do not duplicate coverage from Test262, Babel, TypeScript, or another existing fixture.

Use:

```text
tasks/coverage/misc/pass/
tasks/coverage/misc/fail/
```

Put valid syntax that must parse successfully in `pass/`, and invalid syntax or diagnostic regressions in `fail/`.

If the regression fixes a tracked OXC issue, name the fixture after the issue. Otherwise, choose a short descriptive name for the syntax or diagnostic category:

```text
tasks/coverage/misc/pass/oxc-12345.ts
tasks/coverage/misc/fail/oxc-12345.js
tasks/coverage/misc/fail/missing-conditional-alternative.js
```

Run a focused parser coverage case with:

```sh
cargo coverage parser --filter misc/pass/oxc-12345.ts
cargo coverage parser --filter misc/fail/oxc-12345.js
```

Run all parser coverage with:

```sh
cargo coverage parser
```

## When to use crate tests

Use Rust tests in `crates/oxc_parser` only for parser API behavior, lexer internals, or focused implementation details that cannot be represented as a source fixture.

Do not add ordinary syntax or diagnostic regressions there when a `tasks/coverage/misc` fixture can cover the behavior.
