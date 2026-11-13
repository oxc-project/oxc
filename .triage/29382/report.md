# Triage Report: Issue 29382

## Issue claim

`oxc_lexer` does not combine a surrogate pair written as two separate `\u` escapes into canonical WTF-8 unless both escapes are exactly `\uXXXX` (4-digit, no braces, no line continuation between them). As a result, the cooked bytes contain two lone surrogates (e.g. `ED A0 B4 ED BC 86`) and `lone_surrogates` is reported, instead of the single supplementary character (e.g. `F0 9D 8C 86`).

## Affected subsystem

`crates/oxc_lexer` (string/template literal cooking).

## Reproduction

### Checkout

Revision: `514fb91393a2b3988e4ec687cab6d0f6fcd08e47`

### Minimal input

`"\u{D834}\u{DF06}"; "\uD834\u{DF06}"; "\uD834\
\uDF06"; "\uD834\uDF06";`

This tests four forms:

1. Both escapes use braced form (`\u{D834}\u{DF06}`).
2. Mixed braced/unbraced (`\uD834\u{DF06}`).
3. Two 4-digit escapes separated by a line continuation (`\uD834\<LF>\uDF06`).
4. Two plain 4-digit escapes (`\uD834\uDF06`).

### Commands

```bash
cd .triage/29382/repro
cargo run
```

### Observed output

```
[ED, A0, B4, ED, BC, 86] true
[ED, A0, B4, ED, BC, 86] true
[ED, A0, B4, ED, BC, 86] true
[F0, 9D, 8C, 86] false
```

Exit status: `0` (no crash).

### Expected output

Per the issue, all four strings represent `U+1D306` (𝌆), so the expected output for all four is `[F0, 9D, 8C, 86] false`.

## Result

**REPRODUCED**: The observed output matches the issue report exactly. The first three forms produce non-canonical WTF-8 with `lone_surrogates = true`, while only the plain 4-digit-without-continuation form produces the correct combined supplementary character.

## Diagnosis

### Responsible functions

- `oxc_lexer/src/pipeline/lanes.rs:961` — `cook_decode` (string/template cooking).
- `oxc_lexer/src/pipeline/lanes.rs:873` — `push_cp` (emits a code point to the cooked byte buffer, encoding lone surrogates as 3-byte WTF-8 sequences and setting `lone`).
- `oxc_lexer/src/pipeline/lanes.rs:1390` — `cook_decode_surrogate_pairs` (existing test that encodes the current behavior).

### Specific defect

`cook_decode` only combines surrogate pairs during the handling of a plain 4-digit `\uXXXX` escape whose value is a lead surrogate (`0xD800..=0xDBFF`). The lookahead at `lanes.rs:1140-1160` requires the next six source bytes to be exactly `\u` plus four hex digits, and does not allow:

1. A braced trail surrogate (`\u{DF06}`).
2. A line continuation (`\<LF>` or `\<LS/PS>`) between the lead and trail escapes.

For these cases, `push_cp` emits the lead surrogate as 3-byte WTF-8 (`ED A0 B4`) and sets `lone = true`; on a later iteration it emits the trail surrogate as another 3-byte WTF-8 sequence (`ED BC 86`). The resulting six bytes are two lone surrogates instead of the canonical supplementary-character encoding `F0 9D 8C 86`.

A line continuation contributes zero bytes to the cooked output (handled at `lanes.rs:1046-1052` and `lanes.rs:1137-1148`), so it does not interrupt pairing semantically — it only foils the source-text lookahead. This execution evidence was confirmed by the reproduction script: the third string (`"\uD834\<LF>\uDF06"`) produced `[ED, A0, B4, ED, BC, 86] true`, just like the braced forms.

### Why the current behavior is incorrect

`JSStrBuilder::push_js_char` in `oxc_str/src/js_str_builder.rs:91` resolves a pending lead surrogate against a trailing surrogate that is pushed immediately after it, including across empty appends. That means the rest of the Oxc pipeline assumes lexed string values are in canonical WTF-8 (a lead surrogate followed directly by a trail surrogate is combined into one supplementary character). The new SIMD lexer does not yet guarantee that invariant for all escape forms, so downstream consumers that create `JSStr` from lexer output would need extra fixup or unsafe reliance on the lexer's undocumented invariants.

### Existing test encoding the bug

`cook_decode_surrogate_pairs` at `lanes.rs:1390` explicitly asserts that `"\u{D834}\u{DF06}"` stays as two lone surrogates, under the comment "Braced escapes never pair (parser parity)". As the issue notes, this parity claim predates `JSStr`, which now canonicalizes such pairs. Updating this test is part of the fix.

### Minimal fix approach

In the lexer’s existing `cook_decode` flow, after processing a string/template body, if `lone` was set, run a cold pass over the emitted cooked bytes. Scan for a 3-byte lead surrogate (`ED A0 B4`..`ED AF BF`) immediately followed by a 3-byte trail surrogate (`ED B0 80`..`ED BF BF`) and replace the pair with the equivalent 4-byte supplementary-character encoding. Recompute `lone` from the final buffer. This leaves the hot path unchanged and matches the issue author’s suggested approach.

### Affected consumers

Anything that consumes `Tokens::strings()`/`templates()`/`cooked_bytes()` and builds a `JSStr` from the cooked range, e.g. the new parser using `TokensReader` and any tooling assumed by the issue. The fix lets the lexer emit canonical WTF-8 so those consumers do not need to re-canonicalize.

### Confidence

High: the reproduction output matches the issue exactly, and code inspection traced the defect to the specific lookahead condition in `cook_decode` and the corresponding test assertion.

## Verification

### Verdict

**Bug**

### Evidence

- **Oxc subsystem contract**: `JSStr` in `crates/oxc_str/src/js_str.rs` stores [canonical WTF-8] and explicitly states: "a lead surrogate encoding cannot be immediately followed by a trail surrogate encoding" — i.e. surrogate pairs must use one four-byte supplementary character. The current lexer output `[ED A0 B4 ED BC 86]` (two 3-byte surrogate encodings) violates that invariant.

- **ECMAScript semantics**: All four input forms (`\u{D834}\u{DF06}`, `\uD834\u{DF06}`, `\uD834\<LF>\uDF06`, `\uD834\uDF06`) describe the same UTF-16 code-unit sequence (`U+D834 U+DF06`), which forms the single supplementary character `U+1D306`. The lexer output differs between forms, which is not a documented intentional divergence.

- **Existing parity claim is outdated**: `cook_decode`'s comment and test at `lanes.rs:1390` say "braced escapes never pair (parser parity)", but this parity claim predates `JSStrBuilder`, whose `push_js_char`/`push_js_str` canonicalizes a lead surrogate followed immediately by a trail surrogate regardless of escape form. The rest of the Oxc pipeline (e.g. codegen, ESTree serialization, minifier) depends on `JSStr`'s canonical WTF-8 invariant.

- **Not intentional**: The current behavior is not listed as an intentional difference in any lexer `AGENTS.md` or `README.md`; it was implemented as a local optimization of plain adjacent 4-digit escapes, with the other documented forms overlooked.

### Confidence

High: the bug is a direct violation of `JSStr`'s documented canonical WTF-8 invariant, and the diagnosis traced it to the specific source decoder logic.

## Fix

### Changed source

- `crates/oxc_lexer/src/pipeline/lanes.rs`
  - Added `canonicalize_surrogate_pairs` (`#[cpu_version(Generic)] #[cold] #[inline(never)]`): after `cook_decode` emits a value containing surrogates, scan the cooked bytes for an adjacent lead-surrogate encoding (`ED A0..AF 80..BF`) directly followed by a trail-surrogate encoding (`ED B0..BF 80..BF`), replace the pair with the equivalent 4-byte supplementary-character encoding, and recompute the `lone` flag.
  - Called the new helper in `cook_decode` just before returning, only when `lone` was already set.
  - Updated `cook_decode_surrogate_pairs` to assert that braced, mixed-form, and line-continuation-split surrogate pairs all combine into canonical form.
  - Added `push_template_combines_surrogate_pairs` for the template literal regression.

### Regressions added

- `crates/oxc_lexer/src/pipeline/lanes.rs::cook_decode_surrogate_pairs` — now covers plain, mixed, braced, and line-continuation-split pairs, plus a non-pairing high-then-non-low control case.
- `crates/oxc_lexer/src/pipeline/lanes.rs::push_template_combines_surrogate_pairs` — verifies templates share the same canonicalization.

### Commands and results

```bash
# Reproduction after the fix
cd .triage/29382/repro
cargo run
```

Output:

```
[F0, 9D, 8C, 86] false
[F0, 9D, 8C, 86] false
[F0, 9D, 8C, 86] false
[F0, 9D, 8C, 86] false
```

```bash
just lint -p oxc_lexer
cargo test -p oxc_lexer
```

Both pass.

### Final diff summary

```
crates/oxc_lexer/src/pipeline/lanes.rs | 112 +++++++++++++++++++---
 1 file changed, 103 insertions(+), 9 deletions(-)
```

No generated files, snapshots, or conformance updates were required.
