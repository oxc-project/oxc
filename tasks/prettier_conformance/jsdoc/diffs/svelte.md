# JSDoc Diffs: svelte

Prettier 3.8.1, prettier-plugin-jsdoc 1.8.0

Total JSDoc tags: 4,130
Files with diffs: 3

## `packages/svelte/src/compiler/phases/nodes.js`

Double space collapsed to single space in union type.

```diff
@@ -39,7 +39,7 @@ export function is_element_node(node) {
  * Returns true for all component-like nodes
  *
  * @param {AST.SvelteNode} node
- * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
+ * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
  */
```

oxfmt normalizes double-space to single space in union type. Prettier+jsdoc plugin preserves the double space.

## `packages/svelte/src/internal/client/dom/hydration.js`

```diff
@@ -22,7 +22,9 @@ export function set_hydrating(value) {
 /**
- * The node that is currently being hydrated. This starts out as the first node inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
+ * The node that is currently being hydrated. This starts out as the first node inside the
+ * opening<!--[--> comment, and updates each time a component calls `$.child(...)` or
+ * `$.sibling(...)`.
  *
```

oxfmt wraps the long description line. Prettier+jsdoc plugin leaves it as one line (exceeds printWidth=100).

## `packages/svelte/src/internal/server/renderer.js`

```diff
@@ -36,8 +36,8 @@
- * The `string` values within a renderer are always associated with the {@link type} of that
- * renderer. To switch types, call {@link child} with a different `type` argument.
+ * The `string` values within a renderer are always associated with the {@link type} of that renderer. To
+ * switch types, call {@link child} with a different `type` argument.
```

Different line-break position in wrapped description text. Both fit within printWidth=100, just different wrapping choices.
