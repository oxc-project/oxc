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
    ```markdown
    # JSDoc Diffs: <name>

    ## `path/to/file.ts`
    ```diff
    <unified diff for this file>
    ```

    ## `path/to/other-file.js`
    ```diff
    <unified diff for this file>
    ```
    ```
    Each file's diff should be in its own section with the file path as heading.
    These will be used later to investigate and fix remaining issues.

### Performance Testing
21. Reset to step B state: `git checkout .`
22. Copy the migrated `.oxfmtrc.json` to two variants:
    - `no-jsdoc.json`: same config with `"jsdoc": false`
    - `with-jsdoc.json`: same config with `"jsdoc": true`
23. `hyperfine --warmup 3` comparing both configs with `--check -c <config>`
24. Record: files, time without/with JSDoc, overhead

## Repositories

| # | Name | URL | Scope |
|---|---|---|---|
| 1 | evolu | https://github.com/evoluhq/evolu | all JS/TS |
| 2 | wxt | https://github.com/wxt-dev/wxt | all JS/TS |
| 3 | typedoc | https://github.com/TypeStrong/typedoc | all JS/TS |
| 4 | Chart.js | https://github.com/chartjs/Chart.js | all JS/TS |
| 5 | svelte | https://github.com/sveltejs/svelte | `packages/svelte/src/**/*.{js,ts,jsx,tsx,d.ts}` only |

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
|---|---|---|
| evolu | 134 | 14 |
| wxt | 1,183 | 6 |
| typedoc | 791 | 37 |
| Chart.js | 1,276 | 21 |
| svelte | 4,149 | 9 |
| **Total** | **7,533** | **87** |

### Performance

| Repository | Without JSDoc | With JSDoc | Overhead |
|---|---|---|---|
| evolu | 3.85s | 4.53s | ~18% (noise) |
| wxt | 2.36s | 3.35s | ~42% |
| typedoc | ~10s user | ~10s user | ~0% |
| Chart.js | 2.06s | 2.03s | ~0% |
| svelte | 410ms | 485ms | ~18% (noise) |

### Bugs Found

See `tasks/prettier_conformance/jsdoc/diffs/` for full diffs per repository.
