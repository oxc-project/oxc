# Triage Report: #29582

## Issue claim

`react/immutability` falsely reports a hoisted async function declaration used by a deferred callback.

- Reported on: Oxlint 1.93.0, Node 26.13.1, Windows x64
- Reference: eslint-plugin-react-hooks 7.1.1 / react-hooks/immutability (no diagnostic)
- Affected subsystem: `oxc_react_compiler` (the `react/immutability` linter rule delegates to `ErrorCategory::Immutability` from the React Compiler)

## Reproduction input

`.oxlintrc.json`:

```json
{
  "categories": { "correctness": "off" },
  "plugins": ["react"],
  "rules": { "react/immutability": "error" }
}
```

`repro.tsx`:

```tsx
import { useGuard } from "./guard.mjs";
export function View() {
  const guard = useGuard(() => save());
  async function save() {
    await Promise.resolve();
    guard.markSaved();
    return true;
  }
  return <button onClick={save}>Save</button>;
}
```

`guard.mjs`:

```js
export function useGuard(save) {
  return { run: save, markSaved() {} };
}
```

## Commands and revisions

```bash
git checkout -B triagebot/fix-29582
```

Current HEAD: `6e31b456fcd5d0468ef263e78a906b2739015f3c`

```bash
cargo run -p oxlint -- -c .oxlintrc.json repro.tsx
```

## Observed output

```text
  x react(immutability): Cannot access variable while it is being initialized
   ,-[repro.tsx:3:32]
 2 | export function View() {
 3 |   const guard = useGuard(() => save());
   :                                ^^|^
   :                                  `-- `save` is read during its own initialization
 4 |   async function save() {
   `----
  help: `save` is read while its declaration is still being initialized. Move the access after initialization. For a recursive callback, use a named function expression or restructure the callback so it does not capture itself during initialization
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

Found 0 warnings and 1 error.
Finished in 11ms on 1 file with 1 rules using 4 threads.

Command exited with code 1
```

## Expected behavior

No initialization diagnostic. `save` is a function declaration, initialized on entering `View`'s scope. The arrow `() => save()` is passed to `useGuard` and stored during render; it is invoked later, after `save` has been initialized. This is not a const-arrow self-reference / TDZ case.

The reporter notes that `eslint-plugin-react-hooks` 7.1.1 (react-hooks/immutability, error severity) does not report a diagnostic on the same source.

## Conclusion

REPRODUCED.

The false positive is reproduced on the checked-out revision. The diagnostic mistakenly treats the reference inside the deferred arrow callback as reading `save` during its own initialization, even though `save` is a hoisted function declaration.

## Diagnosis

### Responsible code

- `crates/oxc_react_compiler/src/react_compiler_inference/infer_mutation_aliasing_effects.rs`
  - `find_hoisted_context_declarations` (around line 1025) records all hoisted context declarations (`HoistedConst`, `HoistedFunction`, `HoistedLet`) and the first use of each.
  - `apply_effect_ref` / `AliasingEffect::Mutate` handling (around line 2015) converts a `MutationResult::MutateFrozen` on a hoisted declaration into `diagnostics::variable_accessed_before_declaration`.
  - `InstructionValue::StoreContext` handling (around line 2430) emits a `Mutate` aliasing effect for the initializer of any hoisted context declaration.

### Specific defect

Function declarations are lowered as a hoisted `DeclareContext { kind: HoistedFunction }` followed by a `StoreContext { lvalue: kind: Function }` that assigns the function expression to the hoisted binding. The hoisting pre-pass records `save` because it is referenced inside the arrow `() => save()` passed to `useGuard`. The capture freezes `save`; the later `StoreContext` is then treated as a mutation of a frozen value. Because `save` is also in `hoisted_context_declarations`, the compiler reports the access as a TDZ/self-initialization violation.

This is incorrect: function declarations are initialized on scope entry, so a reference inside a deferred callback is not executed during `save`'s initialization. The analogous `error.invalid-hoisting-setstate.js` case involves `const [state, setState]`, where TDZ detection is legitimate; `HoistedFunction` should be excluded from that diagnostic.

### Verified fix approach

A targeted change in `infer_mutation_aliasing_effects.rs` avoids the false positive while preserving the aliasing effects needed by downstream passes:

1. Extend `find_hoisted_context_declarations` to remember the `InstructionKind` of each hoisted declaration (e.g. change the map value to `(InstructionKind, Option<Place>)`).
2. In the `MutateFrozen && hoisted_context_declarations.contains_key(...)` branch, skip the `variable_accessed_before_declaration` diagnostic when the recorded kind is `InstructionKind::HoistedFunction`. Emit the original `Mutate` effect instead so that `PruneHoistedContexts` and other passes still see the initialization.

I temporarily implemented this, confirmed the reproducer produces `Found 0 warnings and 0 errors`, and verified:

- `cargo test -p oxc_react_compiler --test snapshot` passes (no snapshot churn).
- `cargo test -p oxc_linter immutability` passes.
- `cargo test -p oxc_react_compiler` (unit tests) passes.

The change was reverted before continuing diagnosis.

### Affected consumers

- `react/immutability` lint rule (delegates to `ErrorCategory::Immutability`).
- Potentially any React Compiler analysis that lowers hoisted function declarations captured by deferred callbacks.

### Remaining uncertainty / open questions

- Upstream `babel-plugin-react-compiler` behavior on the same input was not verified locally; the fix assumes aligning with `eslint-plugin-react-hooks` 7.1.1 is correct.
- Interaction with function redeclarations (`function f(){} function f(){}`) was not explicitly tested; the proposed fix preserves the existing `Mutate` aliasing effect and only suppresses the TDZ diagnostic, so redeclarations should still be modeled as mutations.
- The async keyword on `save` appears incidental; the same analysis likely fails for a non-async hoisted function declaration.

### Confidence

High — the exact effect sequence was traced, the offending branch was identified, and a minimal targeted fix was verified to resolve the reproducer without breaking the React Compiler test suite.

## Verification

### Claim being verified

The `react/immutability` diagnostic is a false positive; the corresponding upstream `react-hooks/immutability` rule does not report on the same source.

### Upstream comparison

- Tool: `eslint-plugin-react-hooks` 7.1.1 with ESLint 9.x
- Config: `.agents/triage/29582/eslint.config.mjs` enabling `react-hooks/immutability` at error severity
- Command: `npx eslint repro.tsx`
- Result: exit status 0, no diagnostics

```bash
cd .agents/triage/29582
npx eslint repro.tsx
# (no output, exit=0)
```

### Oxlint comparison (re-run)

```bash
cargo run -p oxlint -- -c .oxlintrc.json repro.tsx
```

Result: exit status 1, one `react(immutability)` error:

```text
  x react(immutability): Cannot access variable while it is being initialized
   ,-[repro.tsx:3:32]
 2 | export function View() {
 3 |   const guard = useGuard(() => save());
   :                                ^^|^
   :                                  `-- `save` is read during its own initialization
 4 |   async function save() {
   `----
```

### Specification / contract evidence

- Function declarations in ECMAScript are `HoistableDeclaration`s whose binding is initialized on scope entry (ECMA-262 15.2.6 / 10.3).
- The arrow `() => save()` is only evaluated for side-effect of creating a function object during render; its body is not executed until `guard.run()` is invoked later, at which point `save` is initialized.
- `crates/oxc_react_compiler/fixtures/error.invalid-hoisting-setstate.js` tests the legitimate TDZ case for `const`/`let` hoisted declarations, which is different from `HoistedFunction`.
- `crates/oxc_react_compiler/AGENTS.md` requires upstream parity with `eslint-plugin-react-hooks` and documents that divergences are expected only when upstream has a false positive, crash, or hang.

### Verdict

**bug** — The diagnostic violates JavaScript hoisting semantics and the upstream `react-hooks/immutability` compatibility contract. The reported code is a supported use case (hoisted function declaration captured by a deferred callback) and should not produce an initialization/TDZ error.

### Confidence

High — both tools were run on the same source with the same rule and severity; ESLint reports nothing while Oxlint reports the false positive, and the root cause was isolated in `infer_mutation_aliasing_effects.rs` during diagnosis.

## Fix

### Change made

`crates/oxc_react_compiler/src/react_compiler_inference/infer_mutation_aliasing_effects.rs`:

- `find_hoisted_context_declarations` now records the `InstructionKind` of each hoisted declaration alongside the first access `Place`.
- When a frozen mutation is detected on a hoisted declaration, the code now checks whether the declaration is a function declaration (`InstructionKind::HoistedFunction`). For function declarations, it emits the original `Mutate` aliasing effect instead of the `variable_accessed_before_declaration` diagnostic. This preserves the aliasing information required by downstream passes such as `PruneHoistedContexts` while avoiding the false TDZ/initialization error.

### Regression added

- `crates/oxc_react_compiler/fixtures/hoisted-function-captured-in-deferred-callback.tsx` (issue reproduction).
- `crates/oxc_react_compiler/tests/snapshots/hoisted-function-captured-in-deferred-callback.snap` — verifies the compiler optimizes the component and reports no diagnostics.

### Verification results after fix

```bash
cargo run -p oxlint -- -c .agents/triage/29582/.oxlintrc.json .agents/triage/29582/repro.tsx
# Found 0 warnings and 0 errors
```

```bash
just lint -p oxc_react_compiler    # ok
just test -p oxc_react_compiler     # ok (snapshot, transform, unit tests)
just lint -p oxc_linter             # ok
just test -p oxc_linter              # ok
just fmt                             # ok
```

### Notes

- The `eslint.config.mjs` and npm scratch project used for upstream verification were removed; only the issue reproduction files, config, and `report.md` remain in `.agents/triage/29582/`.
- A scoped Conventional Commit message for this fix would be: `fix(react_compiler): avoid false TDZ diagnostic for hoisted function declarations`.
