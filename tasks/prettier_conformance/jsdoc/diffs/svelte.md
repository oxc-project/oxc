# JSDoc Diffs: svelte (round 6, 2026-03-17)

8 files with diffs, 7,541 JSDoc tags

## `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js`

```diff
@@ -177,7 +177,7 @@ export function check_element(node, context) {
 				}
 				for (const c_r of value.split(regex_whitespaces)) {
 					const current_role =
-						/** @type {ARIARoleDefinitionKey} current_role */ (c_r);
+						/** @type {ARIARoleDefinitionKey} Current_role */ (c_r);

 					if (current_role && is_abstract_role(current_role)) {
 						w.a11y_no_abstract_role(attribute, current_role);
```

## `packages/svelte/src/compiler/phases/nodes.js`

```diff
@@ -42,7 +42,7 @@ export function is_element_node(node) {
  * Returns true for all component-like nodes
  *
  * @param {AST.SvelteNode} node
- * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
+ * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
  */
 export function is_component_node(node) {
 	return ["Component", "SvelteComponent", "SvelteSelf"].includes(node.type);
```

## `packages/svelte/src/compiler/preprocess/index.js`

```diff
@@ -57,7 +57,7 @@ class PreprocessResult {
 	dependencies = [];

 	/**
-	 * @type {string | null} last Part of the filename, as used for `sources` in
+	 * @type {string | null} Last Part of the filename, as used for `sources` in
 	 *   sourcemaps
 	 */
 	file_basename = /** @type {any} */ (undefined);
```

## `packages/svelte/src/index-client.js`

````diff
@@ -164,9 +164,9 @@ function create_custom_event(
  *
  * ```ts
  * const dispatch = createEventDispatcher<{
- *  loaded: null; // does not take a detail argument
- *  change: string; // takes a detail argument of type string, which is required
- *  optional: number | null; // takes an optional detail argument of type number
+ * 	loaded: null; // does not take a detail argument
+ * 	change: string; // takes a detail argument of type string, which is required
+ * 	optional: number | null; // takes an optional detail argument of type number
  * }>();
  * ```
  *
````

## `packages/svelte/src/index.d.ts`

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
 		element?: typeof HTMLElement;
 	};
```

## `packages/svelte/src/internal/client/dom/elements/custom-element.js`

```diff
@@ -295,7 +295,7 @@ function get_custom_elements_slots(element) {
 	/** @type {Record<string, true>} */
 	const result = {};
 	element.childNodes.forEach((node) => {
-		result[/** @type {Element} node */ (node).slot || "default"] = true;
+		result[/** @type {Element} Node */ (node).slot || "default"] = true;
 	});
 	return result;
 }
```

## `packages/svelte/src/internal/client/dom/hydration.js`

```diff
@@ -24,7 +24,8 @@ export function set_hydrating(value) {

 /**
  * The node that is currently being hydrated. This starts out as the first node
- * inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
+ * inside the opening<!--[--> comment, and updates each time a component calls
+ * `$.child(...)` or `$.sibling(...)`.
  *
  * When entering a block (e.g. `{#if ...}`), `hydrate_node` is the block opening
  * comment; by the time we leave the block it is the closing comment, which
```

## `packages/svelte/src/utils.js`

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
 const ATTRIBUTE_ALIASES = {
 	// no `class: 'className'` because we handle that separately
```
