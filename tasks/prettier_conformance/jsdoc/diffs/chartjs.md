# JSDoc Diffs: Chart.js

Date: 2026-03-15 (round 4)
Prettier version: 3.8.1
JSDoc tags: 1276
Files with diffs: 4

## `src/core/core.controller.js`

```diff
@@ -203,7 +203,13 @@ class Chart {
-    /** @type {?{ attach?: function; detach?: function; resize?: function }} */
+    /**
+     * @type {{
+     *   attach?: function;
+     *   detach?: function;
+     *   resize?: function;
+     * } | null}
+     */
```

## `src/core/core.scale.js`

```diff
@@ -376,8 +376,8 @@ export default class Scale extends Element {
-   * @returns {{ top: number; left: number; bottom: number; right: number }}
-   *   The necessary padding
+   * @returns {{ top: number; left: number; bottom: number; right: number }} The
+   *   necessary padding

@@ -434,9 +434,9 @@ export default class Scale extends Element {
-   * @param {{ top: number; left: number; bottom: number; right: number }} margins
-   *   - The space between the edge of the other scales and edge of the chart This
-   *       space comes from two sources:
+   * @param {{ top: number; left: number; bottom: number; right: number }} margins -
+   *   The space between the edge of the other scales and edge of the chart This
+   *   space comes from two sources:
```

## `src/platform/platform.base.js`

```diff
@@ -7,8 +7,8 @@
-   * implementing the [W3C Canvas 2D Context API
-   * standard]{@link https://www.w3.org/TR/2dcontext/}.
+   * implementing the [W3C Canvas 2D Context API standard]{@link
+   * https://www.w3.org/TR/2dcontext/}.
```

## `src/types/index.d.ts`

```diff
@@ -2085,8 +2085,8 @@ export type AnimationsSpec<TType extends ChartType> = {
-         * 'number', 'color' and 'boolean'. Only really needed for 'color', because
-         * typeof does not get that right.
+         * 'number', 'color' and 'boolean'. Only really needed for 'color',
+         * because typeof does not get that right.

@@ -2557,8 +2557,8 @@ export declare class BasePlatform {
-   * implementing the [W3C Canvas 2D Context API
-   * standard]{@link https://www.w3.org/TR/2dcontext/}.
+   * implementing the [W3C Canvas 2D Context API standard]{@link
+   * https://www.w3.org/TR/2dcontext/}.

@@ -2582,7 +2582,8 @@ export declare class BasePlatform {
    * @param listener - Receives a notification (an object that implements the
-   *   {@link ChartEvent} interface) when an event of the specified type occurs.
+   *   {@link ChartEvent} interface) when an event of the specified type
+   *   occurs.
```
