# JSDoc Diffs: svelte

## `packages/svelte/src/action/public.d.ts`

```diff
diff --git a/packages/svelte/src/action/public.d.ts b/packages/svelte/src/action/public.d.ts
index b09b491..80c0869 100644
--- a/packages/svelte/src/action/public.d.ts
+++ b/packages/svelte/src/action/public.d.ts
@@ -49,10 +49,10 @@ export interface ActionReturn<
  *
  * ```ts
  * export const myAction: Action<HTMLDivElement, { someProperty: boolean } | undefined> = (
- * 	node,
- * 	param = { someProperty: true }
+ *   node,
+ *   param = { someProperty: true }
  * ) => {
- * 	// ...
+ *   // ...
  * };
  * ```
  *
```

## `packages/svelte/src/ambient.d.ts`

```diff
diff --git a/packages/svelte/src/ambient.d.ts b/packages/svelte/src/ambient.d.ts
index 99ff5dd..401b641 100644
--- a/packages/svelte/src/ambient.d.ts
+++ b/packages/svelte/src/ambient.d.ts
@@ -212,11 +212,11 @@ declare namespace $derived {
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
@@ -383,9 +383,9 @@ declare namespace $effect {
  *
  * ```ts
  * let {
- * 	optionalProp = 42,
- * 	requiredProp,
- * 	bindableProp = $bindable()
+ *   optionalProp = 42,
+ *   requiredProp,
+ *   bindableProp = $bindable()
  * }: { optionalProp?: number; requiredProps: string; bindableProp: boolean } = $props();
  * ```
  *
@@ -481,7 +481,7 @@ declare namespace $bindable {
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
index 4215cc0..0bc2afe 100644
--- a/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js
+++ b/packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js
@@ -167,7 +167,7 @@ export function check_element(node, context) {
 					break;
 				}
 				for (const c_r of value.split(regex_whitespaces)) {
-					const current_role = /** @type {ARIARoleDefinitionKey} current_role */ (c_r);
+					const current_role = /** @type {ARIARoleDefinitionKey} Current_role */ (c_r);
 
 					if (current_role && is_abstract_role(current_role)) {
 						w.a11y_no_abstract_role(attribute, current_role);
```

## `packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js`

```diff
diff --git a/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js b/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
index ea561fb..e307e69 100644
--- a/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
+++ b/packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js
@@ -592,11 +592,11 @@ export function build_style_directives_object(
  * ```js
  * let value;
  * $.template_effect(() => {
- * 	if (value !== (value = 'new value')) {
- * 		element.property = value;
- * 		// or
- * 		$.set_attribute(element, property, value);
- * 	}
+ *   if (value !== (value = 'new value')) {
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
index 56e3984..5bc784a 100644
--- a/packages/svelte/src/compiler/phases/nodes.js
+++ b/packages/svelte/src/compiler/phases/nodes.js
@@ -39,7 +39,7 @@ export function is_element_node(node) {
  * Returns true for all component-like nodes
  *
  * @param {AST.SvelteNode} node
- * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
+ * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
  */
 export function is_component_node(node) {
 	return ['Component', 'SvelteComponent', 'SvelteSelf'].includes(node.type);
```

## `packages/svelte/src/compiler/preprocess/index.js`

```diff
diff --git a/packages/svelte/src/compiler/preprocess/index.js b/packages/svelte/src/compiler/preprocess/index.js
index 67389d0..eb24d81 100644
--- a/packages/svelte/src/compiler/preprocess/index.js
+++ b/packages/svelte/src/compiler/preprocess/index.js
@@ -53,7 +53,7 @@ class PreprocessResult {
 	 */
 	dependencies = [];
 
-	/** @type {string | null} last Part of the filename, as used for `sources` in sourcemaps */
+	/** @type {string | null} Last Part of the filename, as used for `sources` in sourcemaps */
 	file_basename = /** @type {any} */ (undefined);
 
 	/** @type {ReturnType<typeof getLocator>} */
```

## `packages/svelte/src/index-client.js`

```diff
diff --git a/packages/svelte/src/index-client.js b/packages/svelte/src/index-client.js
index fbecfc8..5e61c09 100644
--- a/packages/svelte/src/index-client.js
+++ b/packages/svelte/src/index-client.js
@@ -155,9 +155,9 @@ function create_custom_event(type, detail, { bubbles = false, cancelable = false
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
index 79d1923..3401688 100644
--- a/packages/svelte/src/index.d.ts
+++ b/packages/svelte/src/index.d.ts
@@ -242,8 +242,8 @@ export type ComponentEvents<Comp extends SvelteComponent> =
  * import MyComponent from './MyComponent.svelte';
  *
  * function withProps<TComponent extends Component<any>>(
- * 	component: TComponent,
- * 	props: ComponentProps<TComponent>
+ *   component: TComponent,
+ *   props: ComponentProps<TComponent>
  * ) {}
  *
  * // Errors if the second argument is not the correct props expected by the component in the first argument.
```

## `packages/svelte/src/internal/client/dom/elements/custom-element.js`

```diff
diff --git a/packages/svelte/src/internal/client/dom/elements/custom-element.js b/packages/svelte/src/internal/client/dom/elements/custom-element.js
index 3e9f181..2bd6990 100644
--- a/packages/svelte/src/internal/client/dom/elements/custom-element.js
+++ b/packages/svelte/src/internal/client/dom/elements/custom-element.js
@@ -270,7 +270,7 @@ function get_custom_elements_slots(element) {
 	/** @type {Record<string, true>} */
 	const result = {};
 	element.childNodes.forEach((node) => {
-		result[/** @type {Element} node */ (node).slot || 'default'] = true;
+		result[/** @type {Element} Node */ (node).slot || 'default'] = true;
 	});
 	return result;
 }
```

## `packages/svelte/src/internal/client/dom/hydration.js`

```diff
diff --git a/packages/svelte/src/internal/client/dom/hydration.js b/packages/svelte/src/internal/client/dom/hydration.js
index d9e2e34..73e5c03 100644
--- a/packages/svelte/src/internal/client/dom/hydration.js
+++ b/packages/svelte/src/internal/client/dom/hydration.js
@@ -22,7 +22,9 @@ export function set_hydrating(value) {
 }
 
 /**
- * The node that is currently being hydrated. This starts out as the first node inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
+ * The node that is currently being hydrated. This starts out as the first node inside the
+ * opening<!--[--> comment, and updates each time a component calls `$.child(...)` or
+ * `$.sibling(...)`.
  *
  * When entering a block (e.g. `{#if ...}`), `hydrate_node` is the block opening comment; by the
  * time we leave the block it is the closing comment, which serves as the block's anchor.
```

## `packages/svelte/src/internal/client/runtime.js`

```diff
diff --git a/packages/svelte/src/internal/client/runtime.js b/packages/svelte/src/internal/client/runtime.js
index 9fd7b28..8fd2a05 100644
--- a/packages/svelte/src/internal/client/runtime.js
+++ b/packages/svelte/src/internal/client/runtime.js
@@ -752,10 +752,10 @@ export function safe_get(signal) {
  *
  * ```ts
  * $effect(() => {
- * 	// this will run when `data` changes, but not when `time` changes
- * 	save(data, {
- * 		timestamp: untrack(() => time)
- * 	});
+ *   // this will run when `data` changes, but not when `time` changes
+ *   save(data, {
+ *     timestamp: untrack(() => time)
+ *   });
  * });
  * ```
  *
```

## `packages/svelte/src/reactivity/create-subscriber.js`

```diff
diff --git a/packages/svelte/src/reactivity/create-subscriber.js b/packages/svelte/src/reactivity/create-subscriber.js
index 823e379..dccc352 100644
--- a/packages/svelte/src/reactivity/create-subscriber.js
+++ b/packages/svelte/src/reactivity/create-subscriber.js
@@ -28,28 +28,28 @@ import { queue_micro_task } from '../internal/client/dom/task.js';
  * import { on } from 'svelte/events';
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
- * 			const off = on(this.#query, 'change', update);
+ *     this.#subscribe = createSubscriber((update) => {
+ *       // when the `change` event occurs, re-run any effects that read `this.current`
+ *       const off = on(this.#query, 'change', update);
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
index d117974..7905d22 100644
--- a/packages/svelte/src/store/index-client.js
+++ b/packages/svelte/src/store/index-client.js
@@ -43,8 +43,8 @@ export { derived, get, readable, readonly, writable } from './shared/index.js';
  * let count = $state(0);
  *
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v)
+ *   () => count,
+ *   (v) => (count = v)
  * );
  * ```
  *
```

## `packages/svelte/src/store/index-server.js`

```diff
diff --git a/packages/svelte/src/store/index-server.js b/packages/svelte/src/store/index-server.js
index eee4a92..665506e 100644
--- a/packages/svelte/src/store/index-server.js
+++ b/packages/svelte/src/store/index-server.js
@@ -31,8 +31,8 @@ export { derived, get, readable, readonly, writable } from './shared/index.js';
  * let count = $state(0);
  *
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v)
+ *   () => count,
+ *   (v) => (count = v)
  * );
  * ```
  *
```

