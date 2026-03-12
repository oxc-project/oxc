# JSDoc Diffs: svelte

Total JSDoc tags: 4149
Files with diffs: 9

## `packages/svelte/src/compiler/phases/2-analyze/index.js`

Inline `@type` cast with union type gets reformatted differently. Prettier keeps it compact; oxfmt breaks the union across lines.

```diff
@@ -1316,9 +1316,10 @@ function calculate_blockers(instance, analysis) {
 		const reads_writes = new Set();
 		const init =
 			fn.type === "VariableDeclarator"
-				? /** @type {ESTree.FunctionExpression | ESTree.ArrowFunctionExpression} */ (
-						fn.init
-					)
+				? /**
+					 * @type {ESTree.FunctionExpression
+					 * 	| ESTree.ArrowFunctionExpression}
+					 */ (fn.init)
 				: fn;
```

## `packages/svelte/src/compiler/phases/3-transform/client/visitors/shared/utils.js`

Hyphen-dash removed between param type and description continuation.

```diff
@@ -566,8 +566,8 @@ export function build_expression(
  * @param {Expression} expression - The function call to wrap (e.g., $.if,
  *   $.each, etc.)
  * @param {{ start?: number }} node - AST node for location info
- * @param {"component" | "if" | "each" | "await" | "key" | "render"} type - Type
- *   of block/component
+ * @param {"component" | "if" | "each" | "await" | "key" | "render"} type
+ *   Type of block/component
  * @param {Record<string, number | string>} [additional] - Any additional
  *   properties to add to the dev stack entry
  * @returns {ExpressionStatement} - Statement with or without dev stack wrapping
```

## `packages/svelte/src/compiler/phases/nodes.js`

Double space collapsed to single space in union type.

```diff
@@ -42,7 +42,7 @@ export function is_element_node(node) {
  * Returns true for all component-like nodes
  *
  * @param {AST.SvelteNode} node
- * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
+ * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
  */
```

## `packages/svelte/src/index.d.ts`

Minor rewrap of description text.

```diff
@@ -295,8 +295,8 @@ export type ComponentType<Comp extends SvelteComponent = SvelteComponent> =
 		>,
 	) => Comp) & {
 		/**
-		 * The custom element version of the component. Only present if compiled with
-		 * the `customElement` compiler option
+		 * The custom element version of the component. Only present if compiled
+		 * with the `customElement` compiler option
 		 */
```

## `packages/svelte/src/internal/client/dom/css.js`

Inline `@type` cast reformatted differently in ternary expression.

```diff
@@ -14,8 +14,9 @@ export function append_styles(anchor, css) {

 		var target = /** @type {ShadowRoot} */ (root).host
 			? /** @type {ShadowRoot} */ (root)
-			: /** @type {Document} */ ((root).head ??
-				/** @type {Document} */ (root.ownerDocument).head);
+			: /** @type {Document} */ (
+					root.head ?? /** @type {Document} */ (root.ownerDocument).head
+				);
```

## `packages/svelte/src/internal/client/dom/hydration.js`

Minor rewrap of long description line.

```diff
@@ -24,7 +24,8 @@ export function set_hydrating(value) {

 /**
  * The node that is currently being hydrated. This starts out as the first node
- * inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
+ * inside the opening<!--[--> comment, and updates each time a component calls
+ * `$.child(...)` or `$.sibling(...)`.
```

## `packages/svelte/src/internal/client/reactivity/async.js`

**BUG**: Em-dash + code fragment `a + b` is incorrectly parsed as markdown list. The text `— \`await a + b\`` becomes a broken list item.

```diff
@@ -147,8 +147,10 @@ export function capture() {

 /**
  * Wraps an `await` expression in such a way that the effect context that was
- * active before the expression evaluated can be reapplied afterwards — `await a
- * + b` becomes `(await $.save(a))() + b`
+ * active before the expression evaluated can be reapplied afterwards — `await
+ * a
+ *
+ * - B`becomes`(await $.save(a))() + b`
```

## `packages/svelte/src/internal/server/context.js`

**BUG**: Same em-dash + code fragment issue as above.

```diff
@@ -114,9 +114,11 @@ function get_parent_context(ssr_context) {

 /**
  * Wraps an `await` expression in such a way that the component context that was
- * active before the expression evaluated can be reapplied afterwards — `await a
- * + b()` becomes `(await $.save(a))() + b()`, meaning `b()` will have access to
- * the context of its component.
+ * active before the expression evaluated can be reapplied afterwards — `await
+ * a
+ *
+ * - B()`becomes`(await $.save(a))() + b()`, meaning `b()` will have access to the
+ *   context of its component.
```

## `packages/svelte/src/utils.js`

Minor rewrap of `@type` tag description.

```diff
@@ -188,9 +188,9 @@ export function is_boolean_attribute(name) {
 }

 /**
- * @type {Record<string, string>} List of attribute names that should be
- *   aliased to their property names because they behave differently between
- *   setting them as an attribute and setting them as a property.
+ * @type {Record<string, string>} List of attribute names that should be aliased
+ *   to their property names because they behave differently between setting
+ *   them as an attribute and setting them as a property.
  */
```
