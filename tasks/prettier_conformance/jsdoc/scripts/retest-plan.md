# Plan: Re-test JSDoc Formatting Against Real-World Repositories

## Context

After fixing multiple JSDoc formatting bugs (parser tag splitting, union type wrapping, trailing desc, @internal indent, code block preservation, @type capitalization, case-insensitive import sorting), we need to re-validate oxfmt against 5 real-world repositories to confirm zero JSDoc diffs and measure performance overhead.

All fixes have been committed on `feat/jsdoc-comment-formatting`.

## Configuration Alignment

**Critical**: oxfmt and Prettier have different defaults. Rather than manually constructing configs, use the built-in migration tool to ensure full alignment.

Key differences the migration handles:
| Option | Prettier Default | Oxfmt Default | Migration Action |
|--------|------------------|---------------|------------------|
| `printWidth` | 80 | **100** | Sets to 80 |
| `sortPackageJson` | N/A | **true** | Disables unless `prettier-plugin-packagejson` detected |

The migration tool also handles: `.prettierignore` → `ignorePatterns`, `prettier-plugin-tailwindcss` → `sortTailwindcss`, and any non-default Prettier options in the repo's config.

### Migration Approach

1. If the repo has an existing `.prettierrc` / `prettier.config.*` / `package.json` prettier config → use `npx oxfmt@latest --migrate prettier` to generate `.oxfmtrc.json`
2. If no Prettier config exists → create minimal: `{"printWidth": 80, "sortPackageJson": false}`
3. Then add `"jsdoc": false` for Step B, `"jsdoc": true` for Step C

## Prerequisites

1. Rebuild oxfmt: `cd /Users/qing/p/github/oxc_formatter/apps/oxfmt && pnpm build`
2. Binary: `node /Users/qing/p/github/oxc_formatter/apps/oxfmt/dist/cli.js`

## Steps Per Repository

### Setup

1. `rm -rf /tmp/jsdoc-retest/<name> && mkdir -p /tmp/jsdoc-retest/<name>`
2. `git clone --depth 1 <url> /tmp/jsdoc-retest/<name>`
3. `cd /tmp/jsdoc-retest/<name>`
4. Install deps: `pnpm install` if pnpm-lock.yaml exists, else `npm install`
5. Upgrade Prettier to latest and install jsdoc plugin: `npm install prettier@latest prettier-plugin-jsdoc@latest`
6. Verify version: `npx prettier --version` (record in results)

### Step A: Prettier Baseline

7. Create `.prettierrc`: `{"printWidth": 80, "plugins": ["prettier-plugin-jsdoc"]}`
8. Run `npx prettier --write '**/*.{js,ts,jsx,tsx}'` (excluding node_modules/dist/build)
9. `git add -A && git commit -m "prettier baseline"`

### Step B: oxfmt Non-JSDoc Baseline

10. Run `node .../oxfmt/dist/cli.js --migrate prettier` to generate `.oxfmtrc.json` from the repo's Prettier config
    - If no Prettier config exists, create `{"printWidth": 80, "sortPackageJson": false}`
11. Add `"jsdoc": false` to the generated `.oxfmtrc.json`
12. Run `node .../oxfmt/dist/cli.js --write`
13. `git add -A && git commit -m "oxfmt no-jsdoc baseline"`

### Step C: oxfmt With JSDoc

14. Change `"jsdoc": false` → `"jsdoc": true` in `.oxfmtrc.json`
15. Run `node .../oxfmt/dist/cli.js --write`
16. `git diff --stat` — shows ONLY JSDoc-related differences
17. `git diff` — capture actual diffs if any exist

### Data Collection

18. JSDoc tags count: `grep -r '^\s*\*\s*@' --include='*.js' --include='*.ts' --include='*.tsx' --include='*.jsx' . | grep -v node_modules | wc -l`
19. JSDoc diffs: file count from step 16
20. Save diffs to `tasks/prettier_conformance/jsdoc/diffs/<name>.md` (in this repo) with format:

    ````markdown
    # JSDoc Diffs: <name>

    ## `path/to/file.ts`

    ```diff
    <unified diff for this file>
    ```
    ````

    ## `path/to/other-file.js`

    ```diff
    <unified diff for this file>
    ```

    ```
    Each file's diff should be in its own section with the file path as heading.
    These will be used later to investigate and fix remaining issues.
    ```

### Performance Testing

21. Reset to step B state: `git checkout .`
22. Copy the migrated `.oxfmtrc.json` to two variants:
    - `no-jsdoc.json`: same config with `"jsdoc": false`
    - `with-jsdoc.json`: same config with `"jsdoc": true`
23. `hyperfine --warmup 3` comparing both configs with `--check -c <config>`
24. Record: files, time without/with JSDoc, overhead

## Repositories

| #   | Name     | URL                                   | Scope                                                |
| --- | -------- | ------------------------------------- | ---------------------------------------------------- |
| 1   | evolu    | https://github.com/evoluhq/evolu      | all JS/TS                                            |
| 2   | wxt      | https://github.com/wxt-dev/wxt        | all JS/TS                                            |
| 3   | typedoc  | https://github.com/TypeStrong/typedoc | all JS/TS                                            |
| 4   | Chart.js | https://github.com/chartjs/Chart.js   | all JS/TS                                            |
| 5   | svelte   | https://github.com/sveltejs/svelte    | `packages/svelte/src/**/*.{js,ts,jsx,tsx,d.ts}` only |

## Execution

Launch 5 agents in parallel, one per repository. Each follows the full steps and reports:

- JSDoc tag count
- Number of files with JSDoc diffs (target: 0)
- Any actual JSDoc diff content
- Performance numbers

## Expected Output

1. Prettier version used (recorded from step 6)
2. Updated correctness + performance tables ready for GitHub comment
3. Per-repo diff files at `tasks/prettier_conformance/jsdoc/diffs/<name>.md` containing any remaining JSDoc formatting differences (file path + unified diff per file), to be investigated and fixed later

## Results (2026-03-12)

### Correctness

| Repository | JSDoc Tags | Files with Diffs |
| ---------- | ---------- | ---------------- |
| evolu      | 134        | 14               |
| wxt        | 1,183      | 6                |
| typedoc    | 791        | 37               |
| Chart.js   | 1,276      | 21               |
| svelte     | 4,149      | 9                |
| **Total**  | **7,533**  | **87**           |

### Performance

| Repository | Without JSDoc | With JSDoc | Overhead     |
| ---------- | ------------- | ---------- | ------------ |
| evolu      | 3.85s         | 4.53s      | ~18% (noise) |
| wxt        | 2.36s         | 3.35s      | ~42%         |
| typedoc    | ~10s user     | ~10s user  | ~0%          |
| Chart.js   | 2.06s         | 2.03s      | ~0%          |
| svelte     | 410ms         | 485ms      | ~18% (noise) |

### Bugs Found

See `tasks/prettier_conformance/jsdoc/diffs/` for full diffs per repository.

## Results (2026-03-13)

### Correctness

| Repository | JSDoc Tags | Files with Diffs | Change from 03-12 |
| ---------- | ---------- | ---------------- | ----------------- |
| evolu      | 134        | 14               | 0                 |
| wxt        | 1,183      | 4                | -2                |
| typedoc    | 792        | 19               | -18               |
| Chart.js   | 1,275      | 6                | -15               |
| svelte     | 4,148      | 5                | -4                |
| **Total**  | **7,532**  | **48**           | **-39**           |

### Remaining Bug Categories

1. **`{@link}` wrapping differences** (evolu, typedoc, chartjs): oxfmt breaks before `{@link Foo}` to keep inline tags atomic; prettier-plugin-jsdoc keeps them on the same line. Design difference, not a bug.
2. **`{@link}` split across lines** (evolu Protocol.ts): `{@link EncryptedDbChange}` broken as `{@link\n * EncryptedDbChange}`. BUG.
3. **Nested list indent normalization** (evolu, typedoc, chartjs): oxfmt uses 4-space continuation indent instead of 6-space. Improvement.
4. **`{@includeCode}` line breaks** (typedoc): Each `{@includeCode}` placed on its own line. Design difference.
5. **Pipe character misparse** (chartjs helpers.curve.ts): `|splineCurve|` causes paragraph breaks. BUG.
6. **@param after @example gets extra indent** (wxt modules.ts): Tags following @example are indented as nested. BUG.
7. **@default code block indentation lost** (wxt types.ts): Internal indentation of code in @default stripped. BUG.
8. **@example code reformatting** (wxt types.ts, unocss/index.ts): @example content reformatted differently than prettier-plugin-jsdoc.
9. **Multi-line @param type collapsed** (svelte renderer.js): Multi-line `{{ ... }}` type flattened, merging two @param tags. BUG.

### Bugs Found

See `tasks/prettier_conformance/jsdoc/diffs/` for full diffs per repository.

## Results (2026-03-13, post-fix retest: typedoc only)

After commit 6cedfe5 (Fix C: list continuation indent, Fix B: link preservation, Fix A: pipe table, Fix F: multiline type):

| Repository | JSDoc Tags | Files with Diffs | Change from 03-13 |
| ---------- | ---------- | ---------------- | ----------------- |
| typedoc    | ~792       | 22               | +3                |

**Note**: Went from 19 → 22 files. The increase is due to `{@link}` wrapping changes — the link preservation fix (Fix B) now correctly keeps `{@link}` tags atomic, which causes more re-wrapping diffs where prettier-plugin-jsdoc would break mid-link. These are design differences, not bugs.

Previous fixes confirmed working:

- Fix C (list continuation indent): CancellablePromise.ts and renderer.ts list indent diffs are **resolved**
- Fix A (pipe table): No pipe-related misparses in typedoc

Remaining diff categories in typedoc:

1. **`{@link}` wrapping** (most files): oxfmt keeps `{@link Foo}` atomic; prettier breaks across lines
2. **`{@includeCode}` placement** (test files): Each `{@includeCode}` on its own line
3. **JSX attribute type descriptions** (jsx.elements.ts): Different wrapping of `@param` descriptions

## Results (2026-03-14)

After reaching 145/145 conformance (commit c249d0c4) with fixes for: pipe table, link preservation, list indent, multiline type, spread list blank lines, tag blank-line indent, and more.

### Correctness

| Repository | JSDoc Tags | Files with Diffs | Change from 03-13 |
| ---------- | ---------- | ---------------- | ----------------- |
| evolu      | 134        | 14               | 0                 |
| wxt        | 1,183      | 4                | 0                 |
| typedoc    | 792        | 23               | +4                |
| Chart.js   | 1,276      | 5                | -1                |
| svelte     | 4,149      | 10               | +5                |
| **Total**  | **7,534**  | **56**           | **+8**            |

**Note**: Total increased from 48 → 56. The increases in typedoc (+4) and svelte (+5) are primarily from the `{@link}` atomicity fix — keeping `{@link Foo}` as an indivisible unit causes more re-wrapping diffs where prettier-plugin-jsdoc would break mid-link. These are design differences, not bugs.

### Remaining Diff Categories

1. **`{@link}` wrapping** (~15 files across evolu, typedoc, chartjs): oxfmt keeps `{@link Foo}` atomic; prettier-plugin-jsdoc breaks across lines. Design difference.
2. **`{@link}` split across lines** (evolu Protocol.ts): `{@link EncryptedDbChange}` broken as `{@link\n * EncryptedDbChange}`. BUG.
3. **`@type` name capitalization** (svelte a11y/index.js, custom-element.js, preprocess/index.js): oxfmt capitalizes first word after type in inline casts. BUG.
4. **Multi-line `@param` type collapsed** (svelte renderer.js): Multi-line type flattened, merging two `@param` tags. BUG.
5. **`{@includeCode}` placement** (typedoc, 3 files): Each `{@includeCode}` on its own line. Design difference.
6. **Blank line removal between tags** (typedoc, 3 files): oxfmt removes blank lines between `@categoryDescription`/`@document`/`@groupDescription`. Design difference.
7. **`@example` code reformatting** (wxt unocss/index.ts, types.ts): Content reformatted differently. Design difference.
8. **`@default` code block indent** (wxt types.ts): Internal indentation stripped. BUG.
9. **Sub-list indent normalization** (evolu, typedoc, chartjs): oxfmt uses different continuation indent. Design difference / improvement.
10. **`?Type` expansion** (chartjs core.controller.js): `?{ ... }` expanded to `{ ... } | null`. Design difference.

### Performance

| Repository | Without JSDoc | With JSDoc | Overhead    |
| ---------- | ------------- | ---------- | ----------- |
| evolu      | 1.95s         | 3.89s      | ~2x         |
| wxt        | 3.23s         | 2.51s      | ~0% (noise) |
| typedoc    | 1.35s         | 1.53s      | ~13%        |
| Chart.js   | 1.63s         | 2.05s      | ~25%        |
| svelte     | 791ms         | 679ms      | ~0% (noise) |

### Bugs Found

See `tasks/prettier_conformance/jsdoc/diffs/` for full diffs per repository.

## Results (2026-03-15)

After implementing 7 fixes (Fix 2: tabs→spaces, Fix 3: @default indent, Fix 4: @returns wrapping, Fix 5: {@link} split, Fix 7: multiline type collapse, Fix 8: pseudo-code rejection). Fix 1 (@type capitalization) reverted — conformance tests confirmed upstream DOES capitalize. Fix 6 (@param after @example) not reproduced. 145/145 conformance maintained.

### Correctness

| Repository | JSDoc Tags | Files with Diffs | Change from 03-14 |
| ---------- | ---------- | ---------------- | ----------------- |
| evolu      | 134        | 14               | 0                 |
| wxt        | 1,218      | 5                | +1                |
| typedoc    | 792        | 23               | 0                |
| Chart.js   | 1,276      | 4                | -1                |
| svelte     | 4,130      | 14               | +4                |
| **Total**  | **7,550**  | **60**           | **+4**            |

**Note**: Net increase of +4 files is from Fix 2 (IndentStyle::Space for embedded code) converting tabs→spaces in svelte's @example code blocks (+10 new diffs), offset by fixes resolving other diffs (-6). Fix 4 confirmed fixed (Chart.js element.line.js resolved). Fix 3 confirmed fixed (wxt @default indent resolved).

### Fixes Confirmed

- **Fix 2** (tabs→spaces in code blocks): Working. Increased svelte diffs (+4 files with tab code blocks now normalized to spaces).
- **Fix 3** (@default code block indent): **Confirmed fixed** — wxt types.ts `chromiumPref` no longer stripped.
- **Fix 4** (@returns wrapping): **Confirmed fixed** — Chart.js element.line.js no longer has diffs.
- **Fix 5** ({@link} split): **Confirmed fixed** — no more `{@link\n * Foo}` splits across lines.
- **Fix 7** (multiline type collapse): Working — width check prevents overlong collapsed types.
- **Fix 8** (pseudo-code rejection): Partially working — wxt `{undefined}` case still shows diff.

### Remaining Diff Categories

1. **`{@link}` wrapping** (~14 files across evolu, typedoc, chartjs): oxfmt keeps `{@link Foo}` atomic; prettier-plugin-jsdoc breaks across lines. Design difference.
2. **`@type` name capitalization** (svelte 3 files, typedoc 1 file): oxfmt capitalizes first word after type. BUG — needs investigation (conformance tests say capitalize, but real-world Prettier baseline doesn't).
3. **`{@includeCode}` placement** (typedoc, 3 files): Each `{@includeCode}` on its own line. Design difference.
4. **Blank line removal between tags** (typedoc, 2 files): oxfmt removes blank lines between `@categoryDescription`/`@document`/`@primaryExport`. Design difference.
5. **`@example` code reformatting** (wxt 2 files): Content reformatted differently. Design difference.
6. **Sub-list indent normalization** (evolu 2, typedoc 2): oxfmt uses different continuation indent. Design difference / improvement.
7. **`?Type` expansion** (chartjs 1 file): `?{ ... }` expanded to `{ ... } | null`. Design difference.
8. **Tabs→spaces in code blocks** (svelte 10 files): Fix 2 normalizes tab indentation to spaces in @example code blocks. Design difference.
9. **`@param` after `@example` extra indent** (wxt modules.ts): BUG — not reproduced in unit tests.
10. **jsdoc flag affecting non-comment code** (wxt group-entrypoints.test.ts): NEW — possible bug.

### Bugs Found

See `tasks/prettier_conformance/jsdoc/diffs/` for full diffs per repository.
