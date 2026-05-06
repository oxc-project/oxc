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

## 4. `{@link}` placeholder width off-by-one causes lines to exceed printWidth

**Severity**: Low — lines exceed printWidth by 1 character

When wrapping description text containing `{@link ...}` tags, the plugin replaces them with
underscore placeholders for width calculation. The placeholder collapses the space between the
tag name and the link content (`{@link Converter}` → `{@link_________}`), making it 1 character
shorter than the original token. Lines that barely fit with the placeholder exceed printWidth by 1
after the original `{@link}` tag is restored.

**Root cause**: In `descriptionFormatter.ts`, the replacement regex
`/{@(link|linkcode|linkplain)[\s](([^{}])*)}/g` strips the space between the tag keyword and the
content, replacing it with underscores that start immediately after the tag name. The placeholder
is `{@<tag><underscores>}` (1 char shorter than original `{@<tag> <content>}`).

**Example** (from typedoc `src/lib/application.ts`):

```js
// prettier-plugin-jsdoc output (81 chars including " * " prefix, exceeds printWidth=80):
/**
 * This class holds the two main components of TypeDoc, the {@link Converter} and
 * the {@link Renderer}. ...
 */

// oxfmt output (correctly stays within printWidth=80):
/**
 * This class holds the two main components of TypeDoc, the {@link Converter}
 * and the {@link Renderer}. ...
 */
```

**Width breakdown**:

- `{@link Converter}` is 17 characters
- Placeholder `{@link_________}` is 16 characters (space removed)
- Line with placeholder: 77 chars → fits in 80
- After restoration: 78 chars + `*` prefix = 81 → exceeds 80

**Found in**: typedoc (application.ts, converter.ts, renderer.ts, and ~15 other files)

## 5. `{@includeCode}` tag split across lines during wrapping

**Severity**: Medium — breaks inline tag syntax

When wrapping long description lines, the plugin breaks `{@includeCode file#region}` tags across
lines, unlike `{@link}` tags which it correctly keeps atomic. This corrupts the tag syntax.

**Example** (from typedoc `src/test/converter/exports/duplicateRegion.ts`):

```js
// prettier-plugin-jsdoc output (tag broken across lines):
/**
 * {@includeCode
 * duplicateRegion.ts#region}
 */

// oxfmt output (tag kept atomic):
/**
 * {@includeCode duplicateRegion.ts#region}
 */
```

**Root cause**: The plugin's link-protection regex only handles `{@link}`, `{@linkcode}`, and
`{@linkplain}` tags. `{@includeCode}` is not protected, so the markdown wrapping logic treats
its content as normal text and breaks it at word boundaries.

**Found in**: typedoc (duplicateRegion.ts, invalidLineRanges.ts, missingRegion.ts)

## 6. `@default` content not wrapped at printWidth

**Severity**: Low — lines exceed printWidth

When `@default` tag has a blank line followed by prose, the plugin doesn't wrap the prose even
when it exceeds printWidth. The prose line is preserved as-is.

**Example** (from typedoc `src/test/converter/inheritance/inherit-doc.ts`):

```js
// prettier-plugin-jsdoc output (83 chars, exceeds printWidth=80):
/**
 * @default
 *
 * This part of the commentary will not be inherited (this is an abuse of this tag)
 */

// oxfmt output (correctly wraps at 80):
/**
 * @default
 *
 * This part of the commentary will not be inherited (this is an abuse of this
 * tag)
 */
```

**Found in**: typedoc (inherit-doc.ts)

## 7. `@type` name/description split causes inconsistent capitalization

**Severity**: Low — cosmetic, inconsistent behavior

For `@type` tags, comment-parser arbitrarily splits the text after the type into a `name` field
(first word) and `description` field (remaining words). The plugin capitalizes descriptions but
not names, producing inconsistent results where only the second word gets capitalized.

**Root cause**: comment-parser's generic tag parsing always extracts the first word as `name`,
even for `@type` where the entire text is semantically a description. The plugin's `capitalizer()`
is applied to the `description` field but not the `name` field (`stringify.ts:99`).

**Example** (from conformance tests):

```js
// Input:
/** @type {import('eslint').Linter.Config} should be single line */

// prettier-plugin-jsdoc output (inconsistent capitalization):
/** @type {import("eslint").Linter.Config} should Be single line */
// "should" (name) stays lowercase, "be" (description) is capitalized

// oxfmt output (consistent capitalization):
/** @type {import("eslint").Linter.Config} Should be single line */
// Entire text capitalized as description
```

**Found in**: conformance tests (comment-line-strategy 001-004)

## 8. `@description` tag content split to new line

**Severity**: Low — cosmetic

The plugin splits inline `@description` content to a new line with a blank separator, even when
the content would fit on the same line.

**Example** (from conformance tests):

```js
// Input:
/** @description This is a description tag */

// prettier-plugin-jsdoc output (unnecessary split):
/**
 * @description
 *
 * This is a description tag
 */

// oxfmt output (keeps inline):
/** @description This is a description tag */
```

**Found in**: conformance tests (description-tag/002-existing-desc-tag)

## 9. JSON key quoting in `@example`

**Severity**: Low — cosmetic

The plugin parses `@example` content as JSON and re-serializes it, adding quotes around
unquoted object keys. This changes the semantics of the example code.

**Example** (from conformance tests):

```js
// Input:
/**
 * @example { testArr: [1, 2] }
 */

// prettier-plugin-jsdoc output (adds quotes):
/**
 * @example { "testArr": [1, 2] }
 */

// oxfmt output (preserves original):
/**
 * @example { testArr: [1, 2] }
 */
```

**Found in**: conformance tests (keep-unparseable-indent/001-json-example, 004-indent-stripped)

## 10. Lowercase after `@typedef` name and missing type spacing

**Severity**: Medium — incorrect capitalization and spacing

The plugin has two bugs with `@typedef{type}` (no space before brace):

1. It lowercases the first word after the name, misidentifying the sentence boundary.
2. It inserts a space before the brace (`@typedef {type}`), not preserving the original
   no-space format.

**Example** (from conformance tests):

```js
// Input:
/** @typedef{import('...').InitialOptions} name A description */

// prettier-plugin-jsdoc output (wrong):
/** @typedef {import("...").InitialOptions} name a description */
// Adds space before {, lowercases "A" to "a"

// oxfmt output (correct):
/** @typedef{import('...').InitialOptions} name A description */
// Preserves no-space and correct capitalization
```

**Found in**: conformance tests (main/011-bad-defined-name)
