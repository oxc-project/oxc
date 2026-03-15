# JSDoc Diffs: wxt

Date: 2026-03-15 (post-fix retest #2)
Prettier version: 3.8.1
JSDoc tags: 1218
Files with diffs: 4 (excluding .oxfmtrc.json)

## `packages/unocss/src/index.ts`

```diff
@@ -52,8 +52,10 @@ export interface UnoCSSOptions<Theme extends object = object> {
    * it from specific builds.
    *
    * @example
-   *   {undefined} ('popup',
-   *   'options');
+   *   {
+   *     undefined;
+   *   }
+   *   ("popup", "options");
    *
    * @default [ ]
    */
```

## `packages/wxt/src/core/utils/building/__tests__/group-entrypoints.test.ts`

```diff
@@ -185,10 +185,7 @@ describe("groupEntrypoints", () => {
     expect(actual).toEqual([[background]]);
   });

-  it.todo(
-    "should group ESM compatible sandbox scripts with sandbox pages",
-    () => {
-      // Main world content scripts
-    },
-  );
+  it.todo("should group ESM compatible sandbox scripts with sandbox pages", () => {
+    // Main world content scripts
+  });
 });
```

## `packages/wxt/src/core/utils/building/detect-dev-changes.ts`

```diff
@@ -15,7 +15,7 @@ import { wxt } from "../../wxt";
  * - Do nothing
  *
  *   - CSS or JS file associated with an HTML page is changed - this is handled
- *       automatically by the dev server
+ *     automatically by the dev server
  *   - Change isn't used by any of the entrypoints
  * - Reload Content script
  *
@@ -24,10 +24,10 @@ import { wxt } from "../../wxt";
  * - Reload HTML file
  *
  *   - HTML file itself is saved - HMR doesn't handle this because the HTML pages
- *       are pre-rendered
+ *     are pre-rendered
  *   - Chrome is OK reloading the page when the HTML file is changed without
- *       reloading the whole extension. Not sure about firefox, this might need
- *       to change to an extension reload
+ *     reloading the whole extension. Not sure about firefox, this might need to
+ *     change to an extension reload
  * - Reload extension
  *
  *   - Background script is changed
```

## `packages/wxt/src/types.ts`

```diff
@@ -70,7 +70,8 @@ export interface InlineConfig {
    * @example
    *   {{browser}} -mv{{manifestVersion}}
    *
-   * @default <span v-pre>`"{{browser}}-mv{{manifestVersion}}{{modeSuffix}}"`</span>
+   * @default <span
+   *   v-pre>`"{{browser}}-mv{{manifestVersion}}{{modeSuffix}}"`</span>
    */
   outDirTemplate?: string;
   /**
@@ -342,7 +343,7 @@ export interface InlineConfig {
    * root directory or an absolute path.
    *
    * @example
-   *   { "testing": "src/utils/testing.ts" }
+   *   { testing: "src/utils/testing.ts" }
    */
   alias?: Record<string, string>;
   /** Experimental settings - use with caution. */
```
