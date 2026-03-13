# Upstream prettier-plugin-jsdoc Bugs

Bugs found in [prettier-plugin-jsdoc](https://github.com/hosseinmd/prettier-plugin-jsdoc) during
real-world repository testing. These cause incorrect Prettier output that oxfmt intentionally
does not replicate.

## 1. HTML comments in description break line wrapping

**Severity**: Medium — produces lines exceeding printWidth

When a JSDoc description contains an HTML comment (e.g. `<!--[-->`), the markdown parser
(`mdast-util-from-markdown`) creates a block-level `html` AST node instead of keeping it inline.
The plugin's stringifier returns block-level `html` nodes raw via `mdAst.value` without running
any wrapping logic, producing lines that exceed printWidth.

**Root cause**: In `descriptionFormatter.ts`, the `stringyfy` function handles `paragraph` nodes
with wrapping logic, but block-level `html` nodes bypass this entirely — they're returned as-is
from `stringifyASTWithoutChildren` (line ~211).

**Example** (from svelte `packages/svelte/src/internal/client/dom/hydration.js`):

```js
// Input (two lines, fits within printWidth=100):
/**
 * The node that is currently being hydrated. This starts out as the first node inside the opening
 * <!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
 */

// prettier-plugin-jsdoc output (191 chars, exceeds printWidth=100):
/**
 * The node that is currently being hydrated. This starts out as the first node inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
 */
```

**Trigger**: A `\n` before `<!--` in the description text (after `comment-parser` strips `* `
prefixes) causes `fromMarkdown()` to parse it as a block-level HTML node rather than inline HTML
within a paragraph.

**Found in**: svelte (hydration.js)

## 2. Em-dash + backtick-code parsed as markdown list

**Severity**: High — corrupts description text

When a description line ends with an em-dash (`—`) followed by backtick-quoted code containing
`+ b`, the plugin's markdown parser interprets the wrapped continuation as a list item, corrupting
the text.

**Example** (from svelte, found in previous test run with Prettier 3.2.4):

```js
// Input:
/**
 * Wraps an `await` expression ... reapplied afterwards — `await a
 * + b` becomes `(await $.save(a))() + b`
 */

// prettier-plugin-jsdoc output (corrupted):
/**
 * Wraps an `await` expression ... reapplied afterwards — `await
 * a
 *
 * - B`becomes`(await $.save(a))() + b`
 */
```

**Note**: This was fixed in prettier-plugin-jsdoc between versions — not reproducible with v1.8.0.

**Found in**: svelte (async.js, context.js)

## 3. Extra whitespace in types not normalized

**Severity**: Low — cosmetic

The plugin preserves extra whitespace inside JSDoc type expressions (e.g. double spaces in union
types). A formatter should normalize these.

**Example** (from svelte `packages/svelte/src/compiler/phases/nodes.js`):

```js
// Input (double space before AST.SvelteComponent):
/**
 * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
 */

// prettier-plugin-jsdoc output (preserves double space):
/**
 * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
 */

// oxfmt output (normalizes to single space):
/**
 * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
 */
```

**Root cause**: The plugin does not normalize whitespace inside `{...}` type expressions.

**Found in**: svelte (nodes.js)
