# JSDoc Diffs: Chart.js

Total JSDoc tags: 1276
Files with diffs: 21

## `src/core/core.adapters.ts`
```diff
diff --git a/src/core/core.adapters.ts b/src/core/core.adapters.ts
index 7a51b8e..c54c29d 100644
--- a/src/core/core.adapters.ts
+++ b/src/core/core.adapters.ts
@@ -105,11 +105,13 @@ class DateAdapterBase implements DateAdapter {
    * options type.
    *
    * @example
-   *   Chart._adapters._date.override<{ myAdapterOption: string }>({
-   *     init() {
-   *       console.log(this.options.myAdapterOption);
-   *     },
-   *   });
+   *   Chart._adapters._date.override <
+   *     { myAdapterOption: string } >
+   *     {
+   *       init() {
+   *         console.log(this.options.myAdapterOption);
+   *       },
+   *     };
    */
   static override<T extends AnyObject = AnyObject>(
     members: Partial<Omit<DateAdapter<T>, "options">>,
```

## `src/core/core.animator.js`
```diff
diff --git a/src/core/core.animator.js b/src/core/core.animator.js
index afa69fa..6208cec 100644
--- a/src/core/core.animator.js
+++ b/src/core/core.animator.js
@@ -2,7 +2,6 @@ import { requestAnimFrame } from "../helpers/helpers.extras.js";
 
 /**
  * @typedef {import("./core.animation.js").default} Animation
- *
  * @typedef {import("./core.controller.js").default} Chart
  */
 
```

## `src/core/core.controller.js`
```diff
diff --git a/src/core/core.controller.js b/src/core/core.controller.js
index 552deb3..9388da7 100644
--- a/src/core/core.controller.js
+++ b/src/core/core.controller.js
@@ -34,7 +34,6 @@ import { debounce } from "../helpers/helpers.extras.js";
 
 /**
  * @typedef {import("../types/index.js").ChartEvent} ChartEvent
- *
  * @typedef {import("../types/index.js").Point} Point
  */
 
@@ -203,7 +202,13 @@ class Chart {
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
@@ -1360,13 +1365,13 @@ class Chart {
 
   /**
    * @param {ChartEvent} e - The event
-   * @param {import("../types/index.js").ActiveElement[]} lastActive -
+   * @param {import("../types/index.js").ActiveElement[]} lastActive
+   *
    *   Previously active elements
    * @param {boolean} inChartArea - Is the event inside chartArea
    * @param {boolean} useFinalPosition - Should the evaluation be done with
    *   current or final (after animation) element positions
-   * @returns {import("../types/index.js").ActiveElement[]} - The active
-   *   elements
+   * @returns {import("../types/index.js").ActiveElement[]} - The active elements
    * @pravate
    */
   _getActiveElements(e, lastActive, inChartArea, useFinalPosition) {
```

## `src/core/core.datasetController.js`
```diff
diff --git a/src/core/core.datasetController.js b/src/core/core.datasetController.js
index 618824a..8392d1f 100644
--- a/src/core/core.datasetController.js
+++ b/src/core/core.datasetController.js
@@ -16,7 +16,6 @@ import { createContext, sign } from "../helpers/index.js";
 
 /**
  * @typedef {import("./core.controller.js").default} Chart
- *
  * @typedef {import("./core.scale.js").default} Scale
  */
 
```

## `src/core/core.interaction.js`
```diff
diff --git a/src/core/core.interaction.js b/src/core/core.interaction.js
index bb2d6cc..ad2255e 100644
--- a/src/core/core.interaction.js
+++ b/src/core/core.interaction.js
@@ -5,21 +5,17 @@ import { _isPointInArea, isNullOrUndef } from "../helpers/index.js";
 
 /**
  * @typedef {import("./core.controller.js").default} Chart
- *
  * @typedef {import("../types/index.js").ChartEvent} ChartEvent
- *
  * @typedef {{
  *   axis?: string;
  *   intersect?: boolean;
  *   includeInvisible?: boolean;
  * }} InteractionOptions
- *
  * @typedef {{
  *   datasetIndex: number;
  *   index: number;
  *   element: import("./core.element.js").default;
  * }} InteractionItem
- *
  * @typedef {import("../types/index.js").Point} Point
  */
 
```

## `src/core/core.plugins.js`
```diff
diff --git a/src/core/core.plugins.js b/src/core/core.plugins.js
index 545a0e5..26381fb 100644
--- a/src/core/core.plugins.js
+++ b/src/core/core.plugins.js
@@ -7,9 +7,7 @@ import {
 
 /**
  * @typedef {import("./core.controller.js").default} Chart
- *
  * @typedef {import("../types/index.js").ChartEvent} ChartEvent
- *
  * @typedef {import("../plugins/plugin.tooltip.js").default} Tooltip
  */
 
```

## `src/core/core.scale.autoskip.js`
```diff
diff --git a/src/core/core.scale.autoskip.js b/src/core/core.scale.autoskip.js
index b82926f..16ed4b6 100644
--- a/src/core/core.scale.autoskip.js
+++ b/src/core/core.scale.autoskip.js
@@ -3,7 +3,6 @@ import { _factorize } from "../helpers/helpers.math.js";
 
 /**
  * @typedef {import("./core.controller.js").default} Chart
- *
  * @typedef {{
  *   value: number | string;
  *   label?: string;
```

## `src/core/core.scale.js`
```diff
diff --git a/src/core/core.scale.js b/src/core/core.scale.js
index 9771d79..bd5310f 100644
--- a/src/core/core.scale.js
+++ b/src/core/core.scale.js
@@ -46,7 +46,6 @@ const getTicksLimit = (ticksLength, maxTicksLimit) =>
 
 /**
  * @typedef {import("../types/index.js").Chart} Chart
- *
  * @typedef {{
  *   value: number | string;
  *   label?: string;
@@ -435,12 +434,12 @@ export default class Scale extends Element {
    * @param {number} maxWidth - The max width in pixels
    * @param {number} maxHeight - The max height in pixels
    * @param {{ top: number; left: number; bottom: number; right: number }} margins
-   *   - The space between the edge of the other scales and edge of the chart This
-   *       space comes from two sources:
+   *   The space between the edge of the other scales and edge of the chart This
+   *   space comes from two sources:
    *
-   *       - Padding - space that's required to show the labels at the edges of the
-   *               scale
-   *       - Thickness of scales or legends in another orientation
+   *   - Padding - space that's required to show the labels at the edges of the
+   *     scale
+   *   - Thickness of scales or legends in another orientation
    */
   update(maxWidth, maxHeight, margins) {
     const { beginAtZero, grace, ticks: tickOpts } = this.options;
```

## `src/elements/element.arc.ts`
```diff
diff --git a/src/elements/element.arc.ts b/src/elements/element.arc.ts
index 54b622b..49ef62b 100644
--- a/src/elements/element.arc.ts
+++ b/src/elements/element.arc.ts
@@ -159,8 +159,7 @@ function rThetaToXY(r: number, theta: number, x: number, y: number) {
  *
  * Start End
  *
- * 1--->a--->2 Outer /\
- * 8 3 | | | | 7 4 \ / 6<---b<---5 Inner
+ * 1. -->a--->2 Outer / 8 3 | | | | 7 4 \ / 6<---b<---5 Inner
  */
 function pathArc(
   ctx: CanvasRenderingContext2D,
```

## `src/elements/element.line.js`
```diff
diff --git a/src/elements/element.line.js b/src/elements/element.line.js
index f0d587f..c31ad63 100644
--- a/src/elements/element.line.js
+++ b/src/elements/element.line.js
@@ -432,8 +432,7 @@ export default class LineElement extends Element {
    *   `start` index
    * @param {number} params.end - Limit segment to points ending at `start` +
    *   `count` index
-   * @returns {undefined | boolean} - True if the segment is a full loop (path
-   *   should be closed)
+   * @returns {undefined | boolean} - True if the segment is a full loop (path should be closed)
    */
   pathSegment(ctx, segment, params) {
     const segmentMethod = _getSegmentMethod(this);
@@ -446,8 +445,7 @@ export default class LineElement extends Element {
    * @param {CanvasRenderingContext2D | Path2D} ctx
    * @param {number} [start]
    * @param {number} [count]
-   * @returns {undefined | boolean} - True if line is a full loop (path should
-   *   be closed)
+   * @returns {undefined | boolean} - True if line is a full loop (path should be closed)
    */
   path(ctx, start, count) {
     const segments = this.segments;
```

## `src/helpers/helpers.curve.ts`
```diff
diff --git a/src/helpers/helpers.curve.ts b/src/helpers/helpers.curve.ts
index 745a804..77b6a34 100644
--- a/src/helpers/helpers.curve.ts
+++ b/src/helpers/helpers.curve.ts
@@ -124,8 +124,10 @@ function monotoneCompute(
 
 /**
  * This function calculates Bézier control points in a similar way than
+ *
  * |splineCurve|, but preserves monotonicity of the provided data and ensures no
- * local extremums are added between the dataset discrete points due to the
+ *
+ * Local extremums are added between the dataset discrete points due to the
  * interpolation. See :
  * https://en.wikipedia.org/wiki/Monotone_cubic_interpolation
  */
```

## `src/helpers/helpers.segment.js`
```diff
diff --git a/src/helpers/helpers.segment.js b/src/helpers/helpers.segment.js
index 2815a24..939bcca 100644
--- a/src/helpers/helpers.segment.js
+++ b/src/helpers/helpers.segment.js
@@ -9,9 +9,7 @@ import { isPatternOrGradient } from "./helpers.color.js";
 
 /**
  * @typedef {import("../elements/element.line.js").default} LineElement
- *
  * @typedef {import("../elements/element.point.js").default} PointElement
- *
  * @typedef {{ start: number; end: number; loop: boolean; style?: any }} Segment
  */
 
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

## `src/plugins/plugin.filler/filler.helper.js`
```diff
diff --git a/src/plugins/plugin.filler/filler.helper.js b/src/plugins/plugin.filler/filler.helper.js
index 59c92b7..6d99131 100644
--- a/src/plugins/plugin.filler/filler.helper.js
+++ b/src/plugins/plugin.filler/filler.helper.js
@@ -1,8 +1,6 @@
 /**
  * @typedef {import("../../core/core.controller.js").default} Chart
- *
  * @typedef {import("../../core/core.scale.js").default} Scale
- *
  * @typedef {import("../../elements/element.point.js").default} PointElement
  */
 
```

## `src/plugins/plugin.filler/filler.options.js`
```diff
diff --git a/src/plugins/plugin.filler/filler.options.js b/src/plugins/plugin.filler/filler.options.js
index bbd00d9..4ce68f3 100644
--- a/src/plugins/plugin.filler/filler.options.js
+++ b/src/plugins/plugin.filler/filler.options.js
@@ -6,11 +6,8 @@ import {
 
 /**
  * @typedef {import("../../core/core.scale.js").default} Scale
- *
  * @typedef {import("../../elements/element.line.js").default} LineElement
- *
  * @typedef {import("../../types/index.js").FillTarget} FillTarget
- *
  * @typedef {import("../../types/index.js").ComplexFillTarget} ComplexFillTarget
  */
 
```

## `src/plugins/plugin.filler/filler.target.js`
```diff
diff --git a/src/plugins/plugin.filler/filler.target.js b/src/plugins/plugin.filler/filler.target.js
index b4df09a..c831fe6 100644
--- a/src/plugins/plugin.filler/filler.target.js
+++ b/src/plugins/plugin.filler/filler.target.js
@@ -6,9 +6,7 @@ import { simpleArc } from "./simpleArc.js";
 
 /**
  * @typedef {import("../../core/core.controller.js").default} Chart
- *
  * @typedef {import("../../core/core.scale.js").default} Scale
- *
  * @typedef {import("../../elements/element.point.js").default} PointElement
  */
 
```

## `src/plugins/plugin.filler/filler.target.stack.js`
```diff
diff --git a/src/plugins/plugin.filler/filler.target.stack.js b/src/plugins/plugin.filler/filler.target.stack.js
index 0f1866b..b04ff1d 100644
--- a/src/plugins/plugin.filler/filler.target.stack.js
+++ b/src/plugins/plugin.filler/filler.target.stack.js
@@ -1,8 +1,6 @@
 /**
  * @typedef {import("../../core/core.controller.js").default} Chart
- *
  * @typedef {import("../../core/core.scale.js").default} Scale
- *
  * @typedef {import("../../elements/element.point.js").default} PointElement
  */
 
```

## `src/plugins/plugin.tooltip.js`
```diff
diff --git a/src/plugins/plugin.tooltip.js b/src/plugins/plugin.tooltip.js
index 95744d3..2fea839 100644
--- a/src/plugins/plugin.tooltip.js
+++ b/src/plugins/plugin.tooltip.js
@@ -24,11 +24,8 @@ import { createContext, drawPoint } from "../helpers/index.js";
 
 /**
  * @typedef {import("../platform/platform.base.js").Chart} Chart
- *
  * @typedef {import("../types/index.js").ChartEvent} ChartEvent
- *
  * @typedef {import("../types/index.js").ActiveElement} ActiveElement
- *
  * @typedef {import("../core/core.interaction.js").InteractionItem} InteractionItem
  */
 
```

## `src/scales/scale.linearbase.js`
```diff
diff --git a/src/scales/scale.linearbase.js b/src/scales/scale.linearbase.js
index d4dc11c..b713b36 100644
--- a/src/scales/scale.linearbase.js
+++ b/src/scales/scale.linearbase.js
@@ -20,7 +20,8 @@ import { formatNumber } from "../helpers/helpers.intl.js";
  *    setting is respected in this scenario
  * 2. If generationOptions.min, generationOptions.max, and generationOptions.count
  *    is defined spacing = (max - min) / count Ticks are generated as [min, min
- *    + spacing, ..., max]
+ *
+ *    - Spacing, ..., max]
  * 3. If generationOptions.count is defined spacing = (niceMax - niceMin) / count
  * 4. Compute optimal spacing of ticks using niceNum algorithm
  *
```

## `src/scales/scale.time.js`
```diff
diff --git a/src/scales/scale.time.js b/src/scales/scale.time.js
index 5001b18..6034775 100644
--- a/src/scales/scale.time.js
+++ b/src/scales/scale.time.js
@@ -16,9 +16,7 @@ import {
 
 /**
  * @typedef {import("../core/core.adapters.js").TimeUnit} Unit
- *
  * @typedef {{ common: boolean; size: number; steps?: number }} Interval
- *
  * @typedef {import("../core/core.adapters.js").DateAdapter} DateAdapter
  */
 
@@ -451,8 +449,9 @@ export default class TimeScale extends Scale {
   /**
    * Returns the start and end offsets from edges in the form of {start, end}
    * where each value is a relative width to the scale and ranges between 0 and
+   *
    * 1. They add extra margins on the both sides by scaling down the original
-   * scale. Offsets are added when the `offset` option is true.
+   *    scale. Offsets are added when the `offset` option is true.
    *
    * @param {number[]} timestamps
    * @protected
```

## `src/types/index.d.ts`
```diff
diff --git a/src/types/index.d.ts b/src/types/index.d.ts
index 328fabe..2c46da4 100644
--- a/src/types/index.d.ts
+++ b/src/types/index.d.ts
@@ -170,7 +170,7 @@ export interface BarControllerDatasetOptions
   /**
    * Point style for the legend
    *
-   * @default 'circle;
+   * @default "circle;
    */
   pointStyle: PointStyle;
 
@@ -2085,8 +2085,8 @@ export type AnimationsSpec<TType extends ChartType> = {
 
         /**
          * Type of property, determines the interpolator used. Possible values:
-         * 'number', 'color' and 'boolean'. Only really needed for 'color', because
-         * typeof does not get that right.
+         * 'number', 'color' and 'boolean'. Only really needed for 'color',
+         * because typeof does not get that right.
          */
         type: "color" | "number" | "boolean";
 
@@ -2401,7 +2401,7 @@ export interface PointOptions extends CommonElementOptions {
   /**
    * Point style
    *
-   * @default 'circle;
+   * @default "circle;
    */
   pointStyle: PointStyle;
   /**
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

