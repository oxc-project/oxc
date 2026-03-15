# JSDoc Diffs: svelte

Total files with diffs: 15 (in packages/svelte/src/), 85 total
JSDoc tag count: 4149

## `packages/svelte/src/action/public.d.ts`

```diff
diff --git a/packages/svelte/src/action/public.d.ts b/packages/svelte/src/action/public.d.ts
index 608d815..7a68c67 100644
--- a/packages/svelte/src/action/public.d.ts
+++ b/packages/svelte/src/action/public.d.ts
@@ -52,10 +52,10 @@ export interface ActionReturn<
  *
  * ```ts
  * export const myAction: Action<
- * 	HTMLDivElement,
- * 	{ someProperty: boolean } | undefined
+ *   HTMLDivElement,
+ *   { someProperty: boolean } | undefined
  * > = (node, param = { someProperty: true }) => {
- * 	// ...
+ *   // ...
  * };
  * ```
  *
```

## `packages/svelte/src/ambient.d.ts`

```diff
diff --git a/packages/svelte/src/ambient.d.ts b/packages/svelte/src/ambient.d.ts
index a3300c9..c43a57b 100644
--- a/packages/svelte/src/ambient.d.ts
+++ b/packages/svelte/src/ambient.d.ts
@@ -220,11 +220,11 @@ declare namespace $derived {
 	 *
 	 * ```ts
 	 * let total = $derived.by(() => {
-	 * 	let result = 0;
-	 * 	for (const n of numbers) {
-	 * 		result += n;
-	 * 	}
-	 * 	return result;
+	 *   let result = 0;
+	 *   for (const n of numbers) {
+	 *     result += n;
+	 *   }
+	 *   return result;
 	 * });
 	 * ```
 	 *
@@ -394,13 +394,13 @@ declare namespace $effect {
  *
  * ```ts
  * let {
- * 	optionalProp = 42,
- * 	requiredProp,
- * 	bindableProp = $bindable(),
+ *   optionalProp = 42,
+ *   requiredProp,
+ *   bindableProp = $bindable(),
  * }: {
- * 	optionalProp?: number;
- * 	requiredProps: string;
- * 	bindableProp: boolean;
+ *   optionalProp?: number;
+ *   requiredProps: string;
+ *   bindableProp: boolean;
  * } = $props();
  * ```
  *
@@ -500,7 +500,7 @@ declare namespace $bindable {
  * ```ts
  * $inspect(x).with(console.trace);
  * $inspect(x, y).with(() => {
- * 	debugger;
+ *   debugger;
  * });
  * ```
  *
```

## `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js`

```diff
diff --git a/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js b/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js
index b49ffae..2ba6bd3 100644
--- a/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js
+++ b/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js
@@ -177,7 +177,7 @@ export function check_element(node, context) {
 				}
 				for (const c_r of value.split(regex_whitespaces)) {
 					const current_role =
-						/** @type {ARIARoleDefinitionKey} current_role */ (c_r);
+						/** @type {ARIARoleDefinitionKey} Current_role */ (c_r);
 
 					if (current_role && is_abstract_role(current_role)) {
 						w.a11y_no_abstract_role(attribute, current_role);
```

## `packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js`

```diff
diff --git a/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js b/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
index 3e9346c..34d648e 100644
--- a/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
+++ b/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
@@ -698,11 +698,11 @@ export function build_style_directives_object(
  * ```js
  * let value;
  * $.template_effect(() => {
- * 	if (value !== (value = "new value")) {
- * 		element.property = value;
- * 		// or
- * 		$.set_attribute(element, property, value);
- * 	}
+ *   if (value !== (value = "new value")) {
+ *     element.property = value;
+ *     // or
+ *     $.set_attribute(element, property, value);
+ *   }
  * });
  * ```
  *
```

## `packages/svelte/src/compiler/phases/nodes.js`

```diff
diff --git a/packages/svelte/src/compiler/phases/nodes.js b/packages/svelte/src/compiler/phases/nodes.js
index 1d838bb..0d9ef9c 100644
--- a/packages/svelte/src/compiler/phases/nodes.js
+++ b/packages/svelte/src/compiler/phases/nodes.js
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
diff --git a/packages/svelte/src/compiler/preprocess/index.js b/packages/svelte/src/compiler/preprocess/index.js
index 9069268..95e1dae 100644
--- a/packages/svelte/src/compiler/preprocess/index.js
+++ b/packages/svelte/src/compiler/preprocess/index.js
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

```diff
diff --git a/packages/svelte/src/index-client.js b/packages/svelte/src/index-client.js
index 11cfcea..002dcc2 100644
--- a/packages/svelte/src/index-client.js
+++ b/packages/svelte/src/index-client.js
@@ -164,9 +164,9 @@ function create_custom_event(
  *
  * ```ts
  * const dispatch = createEventDispatcher<{
- *  loaded: null; // does not take a detail argument
- *  change: string; // takes a detail argument of type string, which is required
- *  optional: number | null; // takes an optional detail argument of type number
+ *   loaded: null; // does not take a detail argument
+ *   change: string; // takes a detail argument of type string, which is required
+ *   optional: number | null; // takes an optional detail argument of type number
  * }>();
  * ```
  *
```

## `packages/svelte/src/index.d.ts`

```diff
diff --git a/packages/svelte/src/index.d.ts b/packages/svelte/src/index.d.ts
index 508f0a2..d4c1156 100644
--- a/packages/svelte/src/index.d.ts
+++ b/packages/svelte/src/index.d.ts
@@ -246,8 +246,8 @@ export type ComponentEvents<Comp extends SvelteComponent> =
  * import MyComponent from "./MyComponent.svelte";
  *
  * function withProps<TComponent extends Component<any>>(
- * 	component: TComponent,
- * 	props: ComponentProps<TComponent>,
+ *   component: TComponent,
+ *   props: ComponentProps<TComponent>,
  * ) {}
  *
  * // Errors if the second argument is not the correct props expected by the component in the first argument.
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
diff --git a/packages/svelte/src/internal/client/dom/elements/custom-element.js b/packages/svelte/src/internal/client/dom/elements/custom-element.js
index 553ba05..3ca9a6a 100644
--- a/packages/svelte/src/internal/client/dom/elements/custom-element.js
+++ b/packages/svelte/src/internal/client/dom/elements/custom-element.js
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
diff --git a/packages/svelte/src/internal/client/dom/hydration.js b/packages/svelte/src/internal/client/dom/hydration.js
index 0b276e5..d25e045 100644
--- a/packages/svelte/src/internal/client/dom/hydration.js
+++ b/packages/svelte/src/internal/client/dom/hydration.js
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

## `packages/svelte/src/internal/client/runtime.js`

```diff
diff --git a/packages/svelte/src/internal/client/runtime.js b/packages/svelte/src/internal/client/runtime.js
index 72a67d2..26a94da 100644
--- a/packages/svelte/src/internal/client/runtime.js
+++ b/packages/svelte/src/internal/client/runtime.js
@@ -798,10 +798,10 @@ export function safe_get(signal) {
  *
  * ```ts
  * $effect(() => {
- * 	// this will run when `data` changes, but not when `time` changes
- * 	save(data, {
- * 		timestamp: untrack(() => time),
- * 	});
+ *   // this will run when `data` changes, but not when `time` changes
+ *   save(data, {
+ *     timestamp: untrack(() => time),
+ *   });
  * });
  * ```
  *
```

## `packages/svelte/src/reactivity/create-subscriber.js`

```diff
diff --git a/packages/svelte/src/reactivity/create-subscriber.js b/packages/svelte/src/reactivity/create-subscriber.js
index f71f7d2..8318a21 100644
--- a/packages/svelte/src/reactivity/create-subscriber.js
+++ b/packages/svelte/src/reactivity/create-subscriber.js
@@ -32,28 +32,28 @@ import { queue_micro_task } from "../internal/client/dom/task.js";
  * import { on } from "svelte/events";
  *
  * export class MediaQuery {
- * 	#query;
- * 	#subscribe;
+ *   #query;
+ *   #subscribe;
  *
- * 	constructor(query) {
- * 		this.#query = window.matchMedia(`(${query})`);
+ *   constructor(query) {
+ *     this.#query = window.matchMedia(`(${query})`);
  *
- * 		this.#subscribe = createSubscriber((update) => {
- * 			// when the `change` event occurs, re-run any effects that read `this.current`
- * 			const off = on(this.#query, "change", update);
+ *     this.#subscribe = createSubscriber((update) => {
+ *       // when the `change` event occurs, re-run any effects that read `this.current`
+ *       const off = on(this.#query, "change", update);
  *
- * 			// stop listening when all the effects are destroyed
- * 			return () => off();
- * 		});
- * 	}
+ *       // stop listening when all the effects are destroyed
+ *       return () => off();
+ *     });
+ *   }
  *
- * 	get current() {
- * 		// This makes the getter reactive, if read in an effect
- * 		this.#subscribe();
+ *   get current() {
+ *     // This makes the getter reactive, if read in an effect
+ *     this.#subscribe();
  *
- * 		// Return the current state of the query, whether or not we're in an effect
- * 		return this.#query.matches;
- * 	}
+ *     // Return the current state of the query, whether or not we're in an effect
+ *     return this.#query.matches;
+ *   }
  * }
  * ```
  *
```

## `packages/svelte/src/store/index-client.js`

```diff
diff --git a/packages/svelte/src/store/index-client.js b/packages/svelte/src/store/index-client.js
index 7b0d291..dad87de 100644
--- a/packages/svelte/src/store/index-client.js
+++ b/packages/svelte/src/store/index-client.js
@@ -43,8 +43,8 @@ export { derived, get, readable, readonly, writable } from "./shared/index.js";
  * let count = $state(0);
  *
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v),
+ *   () => count,
+ *   (v) => (count = v),
  * );
  * ```
  *
```

## `packages/svelte/src/store/index-server.js`

```diff
diff --git a/packages/svelte/src/store/index-server.js b/packages/svelte/src/store/index-server.js
index 7987a3e..e56fa2d 100644
--- a/packages/svelte/src/store/index-server.js
+++ b/packages/svelte/src/store/index-server.js
@@ -31,8 +31,8 @@ export { derived, get, readable, readonly, writable } from "./shared/index.js";
  * let count = $state(0);
  *
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v),
+ *   () => count,
+ *   (v) => (count = v),
  * );
  * ```
  *
```

## `packages/svelte/src/utils.js`

```diff
diff --git a/packages/svelte/src/utils.js b/packages/svelte/src/utils.js
index d9478aa..19d6f92 100644
--- a/packages/svelte/src/utils.js
+++ b/packages/svelte/src/utils.js
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
