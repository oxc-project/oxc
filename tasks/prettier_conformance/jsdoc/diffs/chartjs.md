# JSDoc Diffs: Chart.js

Date: 2026-03-14
Prettier version: 3.8.1
JSDoc tags: 1276
Files with diffs: 4

## `src/core/core.controller.js`

```diff
diff --git a/src/core/core.controller.js b/src/core/core.controller.js
index 552deb3..355423d 100644
--- a/src/core/core.controller.js
+++ b/src/core/core.controller.js
@@ -203,7 +203,13 @@ class Chart {
     this._active = [];
     this._lastEvent = undefined;
     this._listeners = {};
-    /** @type {?{ attach?: function; detach?: function; resize?: function }} */
+    /**
+     * @type {{
+     *   attach?: function;
+     *   detach?: function;
+     *   resize?: function;
+     * } | null}
+     */
     this._responsiveListeners = undefined;
     this._sortedMetasets = [];
     this.scales = {};
```

## `src/core/core.scale.js`

```diff
diff --git a/src/core/core.scale.js b/src/core/core.scale.js
index 9771d79..12cd38c 100644
--- a/src/core/core.scale.js
+++ b/src/core/core.scale.js
@@ -376,8 +376,8 @@ export default class Scale extends Element {
    * Get the padding needed for the scale
    *
    * @private
-   * @returns {{ top: number; left: number; bottom: number; right: number }}
-   *   The necessary padding
+   * @returns {{ top: number; left: number; bottom: number; right: number }} The
+   *   necessary padding
    */
   getPadding() {
     return {
@@ -434,9 +434,9 @@ export default class Scale extends Element {
   /**
    * @param {number} maxWidth - The max width in pixels
    * @param {number} maxHeight - The max height in pixels
-   * @param {{ top: number; left: number; bottom: number; right: number }} margins
-   *   - The space between the edge of the other scales and edge of the chart This
-   *       space comes from two sources:
+   * @param {{ top: number; left: number; bottom: number; right: number }} margins -
+   *   The space between the edge of the other scales and edge of the chart This
+   *   space comes from two sources:
    *
    *       - Padding - space that's required to show the labels at the edges of the
    *               scale
```

## `src/platform/platform.base.js`

```diff
diff --git a/src/platform/platform.base.js b/src/platform/platform.base.js
index a45371e..02502af 100644
--- a/src/platform/platform.base.js
+++ b/src/platform/platform.base.js
@@ -7,8 +7,8 @@
 export default class BasePlatform {
   /**
    * Called at chart construction time, returns a context2d instance
-   * implementing the [W3C Canvas 2D Context API
-   * standard]{@link https://www.w3.org/TR/2dcontext/}.
+   * implementing the [W3C Canvas 2D Context API standard]{@link
+   * https://www.w3.org/TR/2dcontext/}.
    *
    * @param {HTMLCanvasElement} canvas - The canvas from which to acquire
    *   context (platform specific)
```

## `src/types/index.d.ts`

```diff
diff --git a/src/types/index.d.ts b/src/types/index.d.ts
index 328fabe..fa55578 100644
--- a/src/types/index.d.ts
+++ b/src/types/index.d.ts
@@ -2085,8 +2085,8 @@ export type AnimationsSpec<TType extends ChartType> = {

         /**
          * Type of property, determines the interpolator used. Possible values:
-         * 'number', 'color' and 'boolean'. Only really needed for 'color', because
-         * typeof does not get that right.
+         * 'number', 'color' and 'boolean'. Only really needed for 'color',
+         * because typeof does not get that right.
          */
         type: "color" | "number" | "boolean";

@@ -2557,8 +2557,8 @@ export type ElementChartOptions<TType extends ChartType = ChartType> = {
 export declare class BasePlatform {
   /**
    * Called at chart construction time, returns a context2d instance
-   * implementing the [W3C Canvas 2D Context API
-   * standard]{@link https://www.w3.org/TR/2dcontext/}.
+   * implementing the [W3C Canvas 2D Context API standard]{@link
+   * https://www.w3.org/TR/2dcontext/}.
    *
    * @param {HTMLCanvasElement} canvas - The canvas from which to acquire
    *   context (platform specific)
@@ -2582,7 +2582,8 @@ export declare class BasePlatform {
    * @param {Chart} chart - Chart from which to listen for event
    * @param {string} type - The ({@link ChartEvent}) type to listen for
    * @param listener - Receives a notification (an object that implements the
-   *   {@link ChartEvent} interface) when an event of the specified type occurs.
+   *   {@link ChartEvent} interface) when an event of the specified type
+   *   occurs.
    */
   addEventListener(
     chart: Chart,
```
