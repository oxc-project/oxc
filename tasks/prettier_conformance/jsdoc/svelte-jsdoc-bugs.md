# Svelte JSDoc Formatting Bugs

Bugs found by running oxfmt on the Svelte codebase (`/private/tmp/svelte-jsdoc-test`).
Config: `useTabs: true, singleQuote: true, printWidth: 100, jsdoc: {}`.

## CRITICAL (Change semantics / break code)

### Bug 10: Smart quote corruption in JSDoc types
- `'"'` → `"` (curly/smart quote) inside JSDoc type annotations
- Files: `packages/svelte/src/compiler/phases/1-parse/utils/bracket.js`
- Status: TODO

### Bug 7: Capitalization change in inline `@type` casts
- `/** @type {Element} node */` → `/** @type {Element} Node */`
- `/** @type {ARIARoleDefinitionKey} current_role */` → `Current_role`
- `/** @type {string | null} last Part` → `Last Part`
- Files:
  - `packages/svelte/src/internal/client/dom/elements/custom-element.js`
  - `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js`
  - `packages/svelte/src/compiler/preprocess/index.js`
- Status: WON'T FIX — expected behavior of `jsdocCapitalizeDescription: true` (default).
  Users can disable with `"jsdocCapitalizeDescription": false`. See README.md "Mid-sentence capitalization bug".

### Bug 4: Multiple `@param` tags merged onto one line
- Separate `@param` tags joined into a single line, destroying tag boundaries
- `@param {X} a @param {Y} b @param {Z} c` on one line
- Files:
  - `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/utils.js`
  - `packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js`
  - `packages/svelte/src/compiler/phases/3-transform/utils.js`
- Status: TODO

### Bug 9: ProxyHandler generic type formatting produces malformed syntax
- Complex `@type {ProxyHandler<{...}>}` reformatted with extra `}` on separate line
- Produces invalid JSDoc type expressions
- Files:
  - `packages/svelte/src/internal/client/reactivity/props.js` (3 occurrences)
- Status: TODO

## HIGH (Wrong output, Prettier divergence)

### Bug 2: Single quotes changed to double quotes in JSDoc type expressions
- `AST.Fragment['nodes']` → `AST.Fragment["nodes"]`
- `Binding['kind']` → `Binding["kind"]`
- Violates `singleQuote: true` setting inside JSDoc types
- Files:
  - `packages/svelte/src/compiler/phases/1-parse/index.js`
  - `packages/svelte/src/compiler/phases/1-parse/read/options.js`
  - `packages/svelte/src/compiler/phases/2-analyze/index.js`
  - `packages/svelte/src/compiler/phases/3-transform/client/visitors/ConstTag.js`
  - `packages/svelte/src/compiler/phases/3-transform/client/visitors/shared/element.js`
  - `packages/svelte/src/compiler/phases/3-transform/server/visitors/shared/utils.js`
  - `packages/svelte/src/compiler/phases/3-transform/shared/transform-async.js`
  - `packages/svelte/src/compiler/phases/nodes.js`
  - `packages/svelte/src/compiler/phases/scope.js`
  - `packages/svelte/src/compiler/print/index.js`
  - `packages/svelte/src/compiler/utils/ast.js`
  - `packages/svelte/src/compiler/utils/builders.js`
  - `packages/svelte/src/internal/client/context.js`
- Status: TODO

### Bug 3/15/16: Multi-line types collapsed to single line
- Multi-line `@typedef` with object types collapsed: `{ start: number; end: number }` on one line
- Multi-line `@type` union types collapsed onto single line
- Object properties inside types collapsed: `specificity: { bumped: boolean }`
- Prettier preserves multi-line formatting for these
- Files:
  - `packages/svelte/src/compiler/phases/1-parse/acorn.js`
  - `packages/svelte/src/compiler/phases/2-analyze/css/css-analyze.js`
  - `packages/svelte/src/compiler/phases/3-transform/css/index.js`
  - `packages/svelte/src/compiler/phases/3-transform/utils.js`
  - `packages/svelte/src/compiler/phases/scope.js`
  - `packages/svelte/src/compiler/state.js`
  - `packages/svelte/src/internal/client/dev/tracing.js`
  - `packages/svelte/src/internal/client/dom/elements/transitions.js`
  - `packages/svelte/src/legacy/legacy-client.js`
  - `packages/svelte/tests/runtime-browser/assert.js`
- Status: WON'T FIX — expected behavior of `jsdocCommentLineStrategy: "singleLine"` (default).
  Single-tag comments are collapsed to one line when they fit within printWidth.

### Bug 8: `@internal` tag merged with description
- `@internal` standalone tag + blank line + description → `@internal Description`
- Files:
  - `packages/svelte/src/internal/client/dom/elements/custom-element.js`
- Status: TODO

## MEDIUM (Cosmetic, Prettier incompatible)

### Bug 1/11: Tab-to-spaces conversion in JSDoc continuation lines
- Union type `|` continuation lines use spaces instead of tabs in `useTabs: true` mode
- `@example` code block indentation also converted from tabs to spaces
- Most frequent bug across the diff
- Files:
  - `packages/svelte/src/compiler/migrate/index.js`
  - `packages/svelte/src/compiler/phases/1-parse/utils/bracket.js`
  - `packages/svelte/src/compiler/phases/2-analyze/css/css-prune.js`
  - `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/element.js`
  - `packages/svelte/src/compiler/phases/scope.js`
  - `packages/svelte/src/compiler/utils/builders.js`
  - `packages/svelte/src/html-tree-validation.js`
  - `packages/svelte/src/index-client.js`
  - `playgrounds/sandbox/scripts/download.js`
- Status: TODO

### Bug 17/18: Union type line-breaking differs from Prettier
- Bug 17: Long `@returns` union kept on one line by Prettier but broken by oxfmt
- Bug 18: `null |` at type start broken differently than Prettier
- Opposite direction from Bug 3/15/16 (those collapse, these expand)
- Files:
  - `packages/svelte/src/compiler/phases/2-analyze/css/css-prune.js`
  - `packages/svelte/src/compiler/phases/scope.js`
- Status: TODO

### Bug 6: `{@link}` tag wrapping differences
- Line breaks around `{@link}` tags land at different positions
- Space inserted: `{@link collect}ed` → `{@link collect} ed`
- Files:
  - `packages/svelte/src/attachments/public.d.ts`
  - `packages/svelte/src/internal/client/dom/hydration.js`
  - `packages/svelte/src/internal/server/renderer.js`
- Status: TODO

### Bug 5: `@see` URL wrapping
- Long URL after `@see` broken onto next line; Prettier keeps on one line
- Files:
  - `packages/svelte/elements.d.ts`
- Status: TODO

## LOW (Minor cosmetic)

### Bug 12: Spurious blank line inserted mid-paragraph
- Empty `*` line inserted in middle of multi-line description
- Files:
  - `packages/svelte/src/compiler/phases/nodes.js`
- Status: TODO

### Bug 13: Blank line inserted between `@template` and `@typedef`
- Files:
  - `packages/svelte/src/compiler/validate-options.js`
- Status: TODO

### Bug 14: Import sorting inside JSDoc comment
- `@import` references alphabetically reordered inside JSDoc block
- Files:
  - `packages/svelte/src/internal/server/renderer.js`
- Status: WON'T FIX — intentional feature in `imports.rs`, matches prettier-plugin-jsdoc behavior

### Bug 19: Double space normalized (correct behavior)
- `Component |  AST` → `Component | AST` (extra space removed)
- Files:
  - `packages/svelte/src/compiler/phases/nodes.js`
- Status: WON'T FIX — correct normalization of extra whitespace
