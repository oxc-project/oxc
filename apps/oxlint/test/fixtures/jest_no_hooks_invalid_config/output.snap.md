# Exit code
1

# stdout
```
```

# stderr
```
thread 'tokio-runtime-worker' (3541907) panicked at crates/oxc_linter/src/config/rules.rs:109:34:
failed to parse rule configuration: Error("Invalid rule configuration: unknown field `foo`, expected `allow`", line: 0, column: 0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
node:internal/modules/run_main:107
    triggerUncaughtException(
    ^

[Error: Panic in async function] { code: 'GenericFailure' }

Node.js v24.12.0
```
