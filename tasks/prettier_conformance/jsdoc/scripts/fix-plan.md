# Plan: Fix JSDoc Formatting Differences from Real-World Retest

## Context

After running oxfmt with JSDoc formatting against 5 real-world repos (evolu, wxt, typedoc, Chart.js, svelte), 87 files showed diffs vs prettier-plugin-jsdoc baseline. This plan fixes each bug category (one commit per fix, each with a test).

## Key Files (oxfmt)

- `crates/oxc_formatter/src/formatter/jsdoc/serialize.rs` — Tag routing, blank line logic
- `crates/oxc_formatter/src/formatter/jsdoc/tag_formatters.rs` — `format_example_tag`, `format_generic_tag`, `format_type_name_comment_tag`
- `crates/oxc_formatter/src/formatter/jsdoc/mdast_serialize.rs` — `needs_mdast_parsing`, `normalize_legacy_ordered_list_markers`, link rendering
- `crates/oxc_formatter/src/formatter/jsdoc/normalize.rs` — `capitalize_first`
- `crates/oxc_formatter/src/formatter/jsdoc/wrap.rs` — Text wrapping, `{@link}` tokenization
- `crates/oxc_formatter/src/formatter/jsdoc/embedded.rs` — `format_embedded_js()`

## Key Files (upstream: `../prettier-plugin-jsdoc`)

- `src/stringify.ts` — Tag formatting, inline vs new-line decision (line 147-166)
- `src/descriptionFormatter.ts` — Capitalization, table detection, markdown, list markers
- `src/parser.ts` — Tag parsing, default value extraction, tag grouping
- `src/roles.ts` — Tag classifications (TAGS_NAMELESS, TAGS_TYPELESS, etc.)
- `src/utils.ts` — `capitalizer()`, `formatCode()`

## Fixes (one commit each, with test)

### Fix 1: `@remarks` routed through code formatter

**Bug**: `@remarks` shares `format_example_tag` with `@example` (serialize.rs:261), so its text is parsed as JavaScript — adds semicolons (`Remarks` → `Remarks;`), uses 4-space code indent.
**Upstream**: `@remarks` is paragraph text. In `stringify.ts:151`, `[REMARKS, PRIVATE_REMARKS].includes(tag)` forces description to next line. It is NOT formatted as code.
**Repos**: typedoc (8 occurrences)
**Fix**: In `serialize.rs:261`, remove `|| normalized_kind == "remarks"` so `@remarks` falls through to `format_generic_tag`.
**Test**: `tests/fixtures/js/jsdoc/remarks-tag.js` — `@remarks` with single-word body (no semicolon) and multi-line body (correct indent).

### Fix 2: `@example` TypeScript generics mangled

**Bug**: `format_embedded_js()` tries JSX first, then TSX. Generics like `<number>` parse as JS comparisons: `await storage.getItem<number>(...)` → `(await storage.getItem) < number > ...`
**Upstream**: In `utils.ts:248-259`, uses `options.parser` (respects parent config). When the parent file is TypeScript, the TypeScript parser handles generics correctly.
**Repos**: wxt (2 files), Chart.js (1 file)
**Fix**: In `embedded.rs`, try TSX first (supports generics), then JSX. TSX is a superset of JSX so correct JSX code will also parse fine as TSX.
**Test**: `tests/fixtures/js/jsdoc/example-generics.js` — `@example` with `getItem<number>(...)` and `override<{opt: string}>({...})`.

### Fix 3: `normalize_legacy_ordered_list_markers` false positives

**Bug**: Pattern `<digits>-<text>` converts `64-bit` → `64. Bit` and `1--->a` → `1. -->a`.
**Upstream**: In `descriptionFormatter.ts:84-85`, regex `^(\d+)[-][\s|]+` requires **whitespace or pipe** after dash, not word chars. `64-bit` won't match because `b` follows `-`.
**Repos**: evolu (1 file), Chart.js (1 file)
**Fix**: In `mdast_serialize.rs:275`, require a space after the dash before treating as a list marker. Change condition to check `trimmed.as_bytes().get(dash_pos + 1) == Some(&b' ')`.
**Test**: `tests/fixtures/js/jsdoc/legacy-list-markers.js` — `64-bit` and `1--->a--->2` preserved; `1- foo` still converted.

### Fix 4: Wrapped lines creating false markdown list markers

**Bug**: After wrapping, lines can start with `+` or `-` (e.g., em-dash `—` + `` `await a + b` ``), triggering false list detection → text corruption.
**Upstream**: In `descriptionFormatter.ts`, wrapped continuation lines are prefixed with `beginningSpace` (indentation), preventing false list detection at the markdown parser level.
**Repos**: svelte (2 files), Chart.js (1 file)
**Fix**: In `needs_mdast_parsing()` for `-` and `+` at line boundaries, don't trigger when the character appears mid-paragraph (not preceded by a blank line). Or: apply mdast parsing BEFORE wrapping so the parser has structural context.
**Test**: `tests/fixtures/js/jsdoc/false-list-markers.js` — text with em-dash + backtick code containing `+ b` wraps without corruption.

### Fix 5: Code block destruction in indented context

**Bug**: 4-space-indented markdown code blocks inside JSDoc descriptions are collapsed into single lines.
**Upstream**: Uses `mdast-util-from-markdown` which correctly parses indented code blocks and preserves them as `code` nodes in the AST.
**Repos**: typedoc (1 file: comment.ts)
**Fix**: In `mdast_serialize.rs`, ensure indented code blocks (`code` nodes from markdown AST) are preserved as-is with their indentation, not collapsed.
**Test**: `tests/fixtures/js/jsdoc/indented-code-block.js` — 4-space indented code block inside a JSDoc comment preserved.

### Fix 6: Block tag description placement

**Bug**: `@privateRemarks`, `@categoryDescription`, `@groupDescription`, `@summary` put description on same line; upstream puts on next line.
**Upstream**: In `stringify.ts:151`, only `@remarks` and `@privateRemarks` FORCE new line. Other tags wrap normally based on printWidth. BUT the description for `@categoryDescription`/`@groupDescription` starts on next line because these tags have a "name" component (category/group name) that takes up the first line.
**Repos**: typedoc (15+ files)
**Fix**: Two changes:

1. For `@remarks` and `@privateRemarks`: force description on next line (matches upstream `stringify.ts:151`).
2. For `@categoryDescription`, `@groupDescription`: ensure the name goes on the tag line and description wraps to next line when it exceeds printWidth (standard wrapping, but respecting name vs description boundary).
   **Test**: `tests/fixtures/js/jsdoc/block-tag-placement.js` — `@privateRemarks`, `@categoryDescription` with descriptions.

### Fix 7: Blank line removal between consecutive `@typedef`

**Bug**: Blank lines between consecutive `@typedef` declarations removed.
**Upstream**: In `parser.ts:194-209`, `SPACE_TAG_DATA` is inserted between tag groups, producing blank lines. Consecutive `@typedef` tags within the same group get blank line separators.
**Repos**: Chart.js (12 files), typedoc (2 files)
**Fix**: In `serialize.rs:236-241`, also add blank line between consecutive same-kind `@typedef`/`@callback` tags.
**Test**: `tests/fixtures/js/jsdoc/typedef-blank-lines.js` — consecutive `@typedef` tags with blank lines preserved.

### Fix 8: Capitalization inside type expressions

**Bug**: `@fires {CustomEvent<{ id: string }>}` capitalizes `id` → `Id`.
**Upstream**: In `descriptionFormatter.ts:287-292`, capitalization is applied to the description text AFTER the type expression is extracted by comment-parser. The type `{...}` is never capitalized.
**Repos**: typedoc (1 file)
**Fix**: In `format_generic_tag` (tag_formatters.rs), when description starts with `{`, skip past the balanced `}` before applying `capitalize_first()`.
**Test**: `tests/fixtures/js/jsdoc/capitalize-skip-types.js` — `@fires {CustomEvent<{ id: string }>}` preserves lowercase `id`.

### Fix 9: `@template`/`@param` default value duplication

**Bug**: `@template {TypedArray} [T=Typed<TypedArray>] Desc. Default is \`X\`` → duplicates "Default is \`X\`".
**Upstream**: In `parser.ts:663-670`, default value is extracted from `[name=value]` as a single property by comment-parser. No duplication occurs.
**Repos**: typedoc (2 files)
**Fix**: In `format_type_name_comment_tag`, investigate the `[name=value]` extraction and ensure the description text after the default value is not duplicated during wrapping.
**Test**: `tests/fixtures/js/jsdoc/param-default-value.js` — `@template` with `[T=X]` and description → no duplication.

### Fix 10: Pipe `|` in prose treated as table separator

**Bug**: `|splineCurve|` triggers mdast table parsing.
**Upstream**: In `descriptionFormatter.ts:87-112`, table detection regex requires `\n|` or `^|` at line boundaries AND the pattern must span multiple lines. Inline `|word|` does NOT trigger table detection.
**Repos**: Chart.js (1 file), typedoc (1 file)
**Fix**: In `needs_mdast_parsing()`, for `|` at line start, require at least 2 `|` on the same line (minimum table cell pattern `| x |`).
**Test**: `tests/fixtures/js/jsdoc/pipe-in-prose.js` — `|splineCurve|` preserved as-is.

### Fix 11: `@default` unmatched quote conversion

**Bug**: `@default 'circle;` → `@default "circle;` (converts unmatched quote).
**Upstream**: In `parser.ts:648-686`, regex `'.*'` requires matched quotes. `'circle;` won't match, preserved as-is.
**Repos**: Chart.js (2 occurrences)
**Fix**: In `format_default_value` (serialize.rs:718), verify the value has matching opening/closing quotes before treating as JSON-like.
**Test**: `tests/fixtures/js/jsdoc/default-unmatched-quote.js` — `@default 'circle;` preserved.

### Fix 12: Markdown link simplification

**Bug**: `[https://url](https://url)` simplified to just `https://url`.
**Upstream**: In `descriptionFormatter.ts:412-415`, links are always rendered as `[text](url)`. No simplification to bare URLs.
**Repos**: wxt (2 occurrences)
**Fix**: In `mdast_serialize.rs`, remove the `link_text == link.url` simplification logic. Always emit `[text](url)` format.
**Test**: `tests/fixtures/js/jsdoc/markdown-links.js` — `[url](url)` preserved as markdown link.

### Fix 13: `{@link}` line wrapping differences (LOWEST PRIORITY)

**Bug**: oxfmt wraps before `{@link Foo}` to stay within printWidth; upstream is more lenient.
**Repos**: evolu (12 files), typedoc (10 files), Chart.js (2 files)
**Fix**: Consider measuring `{@link}` width as just the link text (excluding `{@link }` wrapper), matching rendered width. Both outputs are valid; this is cosmetic.
**Test**: `tests/fixtures/js/jsdoc/link-wrapping.js` — `{@link Foo}` at line end with slight overflow stays on same line.

## Excluded (not oxfmt bugs)

- **Inline `@type` cast reformatting in ternary expressions** (svelte): Cosmetic formatting difference.
- **`?` nullable → `| null` conversion**: Both formatters normalize; minor expansion differences.
- **`@example` object key quote removal**: Standard JS formatting of example code.
- **Double space normalization**: Correct behavior.

## Implementation Order

Each fix = 1 commit with test. Commit format: `fix(jsdoc): <description>`

1. Fix 1 — `@remarks` routing (trivial, 8 diffs)
2. Fix 3 — legacy list markers (small, 2 diffs)
3. Fix 4 — false list markers from wrapping (3 diffs)
4. Fix 2 — `@example` TSX-first (3 diffs)
5. Fix 7 — `@typedef` blank lines (14 diffs)
6. Fix 6 — block tag description placement (15+ diffs)
7. Fix 10 — pipe in prose (2 diffs)
8. Fix 8 — capitalization in types (1 diff)
9. Fix 5 — code block preservation (1 diff)
10. Fix 9 — default value duplication (2 diffs)
11. Fix 11 — unmatched quote (2 diffs)
12. Fix 12 — link simplification (2 diffs)
13. Fix 13 — `{@link}` wrapping (cosmetic, 24 diffs)

## Testing Pattern

Per fix:

1. Create test input file in `tests/fixtures/js/jsdoc/<name>.js`
2. Run `cargo insta test --accept -p oxc_formatter --test mod` to generate initial snapshot
3. Verify snapshot shows the bug (wrong output)
4. Implement the fix
5. Re-run `cargo insta test --accept -p oxc_formatter --test mod` to update snapshot
6. Verify snapshot now shows correct output
7. `cargo test -p oxc_formatter` — all tests pass
8. `cargo run -p oxc_prettier_conformance` — no regression
9. Commit

After all fixes: full 5-repo retest.
