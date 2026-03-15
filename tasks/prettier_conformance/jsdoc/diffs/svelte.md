# JSDoc Diffs: svelte

Date: 2026-03-15 (round 4)
Prettier version: 3.8.1
JSDoc tags: 4149
Files with diffs: 17

## `packages/svelte/src/action/public.d.ts`

Tabs→spaces in @example code block.

```diff
@@ -52,10 +52,10 @@ export interface ActionReturn<
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
```

## `packages/svelte/src/ambient.d.ts`

Tabs→spaces in @example code blocks.

```diff
@@ -220,11 +220,11 @@ declare namespace $derived {
  * let total = $derived.by(() => {
- * 	let result = 0;
- * 	for (const n of numbers) {
- * 		result += n;
- * 	}
- * 	return result;
+ *   let result = 0;
+ *   for (const n of numbers) {
+ *     result += n;
+ *   }
+ *   return result;

@@ -394,13 +394,13 @@ declare namespace $effect {
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

@@ -500,7 +500,7 @@ declare namespace $bindable {
  * $inspect(x).with(console.trace);
  * $inspect(x, y).with(() => {
- * 	debugger;
+ *   debugger;
```

## `packages/svelte/src/compiler/phases/2-analyze/index.js`

Tabs→spaces in inline @type cast with multiline type.

```diff
@@ -1316,9 +1316,10 @@ function calculate_blockers(instance, analysis) {
 		const init =
 			fn.type === "VariableDeclarator"
-				? /** @type {ESTree.FunctionExpression | ESTree.ArrowFunctionExpression} */ (
-						fn.init
-					)
+				? /**
+					 * @type {ESTree.FunctionExpression
+					 * 	| ESTree.ArrowFunctionExpression}
+					 */ (fn.init)
```

## `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js`

@type name capitalization: `current_role` → `Current_role`.

```diff
@@ -177,7 +177,7 @@ export function check_element(node, context) {
 				const current_role =
-					/** @type {ARIARoleDefinitionKey} current_role */ (c_r);
+					/** @type {ARIARoleDefinitionKey} Current_role */ (c_r);
```

## `packages/svelte/src/compiler/phases/3-transform/client/visitors/RegularElement.js`

Tabs→spaces in @example code block.

```diff
@@ -698,11 +698,11 @@ export function build_style_directives_object(
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
```

## `packages/svelte/src/compiler/phases/nodes.js`

Union type double-space normalization.

```diff
@@ -42,7 +42,7 @@ export function is_element_node(node) {
- * @returns {node is AST.Component |  AST.SvelteComponent | AST.SvelteSelf}
+ * @returns {node is AST.Component | AST.SvelteComponent | AST.SvelteSelf}
```

## `packages/svelte/src/compiler/preprocess/index.js`

@type name capitalization: `last` → `Last`.

```diff
@@ -57,7 +57,7 @@ class PreprocessResult {
-	 * @type {string | null} last Part of the filename, as used for `sources` in
+	 * @type {string | null} Last Part of the filename, as used for `sources` in
```

## `packages/svelte/src/index-client.js`

Tabs→spaces in @example code block.

```diff
@@ -164,9 +164,9 @@ function create_custom_event(
  * const dispatch = createEventDispatcher<{
- *  loaded: null; // does not take a detail argument
- *  change: string; // takes a detail argument of type string, which is required
- *  optional: number | null; // takes an optional detail argument of type number
+ *   loaded: null; // does not take a detail argument
+ *   change: string; // takes a detail argument of type string, which is required
+ *   optional: number | null; // takes an optional detail argument of type number
```

## `packages/svelte/src/index.d.ts`

Tabs→spaces in @example code block + {@link} wrapping.

```diff
@@ -246,8 +246,8 @@ export type ComponentEvents<Comp extends SvelteComponent> =
  * function withProps<TComponent extends Component<any>>(
- * 	component: TComponent,
- * 	props: ComponentProps<TComponent>,
+ *   component: TComponent,
+ *   props: ComponentProps<TComponent>,

@@ -295,8 +295,8 @@ export type ComponentType<Comp extends SvelteComponent = SvelteComponent>
-		 * The custom element version of the component. Only present if compiled with
-		 * the `customElement` compiler option
+		 * The custom element version of the component. Only present if compiled
+		 * with the `customElement` compiler option
```

## `packages/svelte/src/internal/client/dom/elements/custom-element.js`

@type name capitalization: `node` → `Node`.

```diff
@@ -295,7 +295,7 @@ function get_custom_elements_slots(element) {
-		result[/** @type {Element} node */ (node).slot || "default"] = true;
+		result[/** @type {Element} Node */ (node).slot || "default"] = true;
```

## `packages/svelte/src/internal/client/dom/hydration.js`

{@link} wrapping.

```diff
@@ -24,7 +24,8 @@ export function set_hydrating(value) {
  * The node that is currently being hydrated. This starts out as the first node
- * inside the opening<!--[--> comment, and updates each time a component calls `$.child(...)` or `$.sibling(...)`.
+ * inside the opening<!--[--> comment, and updates each time a component calls
+ * `$.child(...)` or `$.sibling(...)`.
```

## `packages/svelte/src/internal/client/runtime.js`

Tabs→spaces in @example code block.

```diff
@@ -798,10 +798,10 @@ export function safe_get(signal) {
  * $effect(() => {
- * 	// this will run when `data` changes, but not when `time` changes
- * 	save(data, {
- * 		timestamp: untrack(() => time),
- * 	});
+ *   // this will run when `data` changes, but not when `time` changes
+ *   save(data, {
+ *     timestamp: untrack(() => time),
+ *   });
```

## `packages/svelte/src/internal/server/renderer.js`

Multi-line @param type collapsed, merging two @param tags.

```diff
@@ -250,10 +250,9 @@ export class Renderer {
-	 * @param {{
-	 * 	failed?: (renderer: Renderer, error: unknown, reset: () => void) => void;
-	 * }} props
-	 * @param {(renderer: Renderer) => MaybePromise<void>} children_fn
+	 * @param {{ failed?: (renderer: Renderer, error: unknown, reset: () => void)
+	 * => void; }} props @param {(renderer: Renderer) => MaybePromise<void>}
+	 * children_fn
```

## `packages/svelte/src/reactivity/create-subscriber.js`

Tabs→spaces in @example code block.

```diff
@@ -32,28 +32,28 @@ import { queue_micro_task } from "../internal/client/dom/task.js";
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
+ *     this.#subscribe = createSubscriber((update) => {
  ...
```

## `packages/svelte/src/store/index-client.js`

Tabs→spaces in @example code block.

```diff
@@ -43,8 +43,8 @@ export { derived, get, readable, readonly, writable } from "./shared/index.js";
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v),
+ *   () => count,
+ *   (v) => (count = v),
```

## `packages/svelte/src/store/index-server.js`

Tabs→spaces in @example code block.

```diff
@@ -31,8 +31,8 @@ export { derived, get, readable, readonly, writable } from "./shared/index.js";
  * const store = toStore(
- * 	() => count,
- * 	(v) => (count = v),
+ *   () => count,
+ *   (v) => (count = v),
```

## `packages/svelte/src/utils.js`

{@link} wrapping.

```diff
@@ -188,9 +188,9 @@ export function is_boolean_attribute(name) {
- * @type {Record<string, string>} List of attribute names that should be
- *   aliased to their property names because they behave differently between
- *   setting them as an attribute and setting them as a property.
+ * @type {Record<string, string>} List of attribute names that should be aliased
+ *   to their property names because they behave differently between setting
+ *   them as an attribute and setting them as a property.
```
