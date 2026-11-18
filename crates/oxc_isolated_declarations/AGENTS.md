# AGENTS.md

Add local regressions to `tests/fixtures/*.{ts,tsx}`, not the imported
`tests/deno/` suite. `tests/mod.rs` snapshots emitted declarations and diagnostics
under `tests/snapshots/`.

For compatibility fixes, compare TypeScript declaration output with matching
options. Check the actual emitted declarations with TypeScript when validity is
in question; this crate infers types without a full type checker.

Focused snapshots:

```sh
just test -p oxc_isolated_declarations --test mod snapshots
```

Follow [verification.md](../../.agents/verification.md) for final checks and the
root snapshot workflow. Binding behavior belongs in `napi/transform/test/id.test.ts`:

```sh
pnpm -C napi/transform run build-test
pnpm -C napi/transform test
```
