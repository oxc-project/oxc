# JSDoc Diffs: wxt

Total JSDoc tags: 1183
Files with diffs: 6

## `packages/storage/src/index.ts`
```diff
@@ -691,7 +691,7 @@ export interface WxtStorage {
    * Get an item from storage, or return `null` if it doesn't exist.
    *
    * @example
-   *   await storage.getItem<number>("local:installDate");
+   *   (await storage.getItem) < number > "local:installDate";
    */

@@ -742,7 +742,7 @@ export interface WxtStorage {
-   *   await storage.setItem<number>("local:installDate", Date.now());
+   *   (await storage.setItem) < number > ("local:installDate", Date.now());
```

## `packages/unocss/src/index.ts`
```diff
@@ -52,8 +52,10 @@
    * @example
-   *   {undefined} ('popup',
-   *   'options');
+   *   {
+   *     undefined;
+   *   }
+   *   ("popup", "options");
```

## `packages/wxt/src/core/utils/building/detect-dev-changes.ts`
```diff
@@ -15,7 +15,7 @@
-   *       automatically by the dev server
+   *     automatically by the dev server

@@ -24,10 +24,10 @@
-   *       are pre-rendered
+   *     are pre-rendered
    *   - Chrome is OK reloading the page when the HTML file is changed without
-   *       reloading the whole extension. Not sure about firefox, this might need
-   *       to change to an extension reload
+   *     reloading the whole extension. Not sure about firefox, this might need to
+   *     change to an extension reload
```

## `packages/wxt/src/modules.ts`
```diff
@@ -102,8 +102,8 @@
-   * @param wxt The wxt instance provided by the module's setup function.
-   * @param viteConfig A function that returns the vite config the module is
+   *   @param wxt The wxt instance provided by the module's setup function.
+   *   @param viteConfig A function that returns the vite config the module is
    *   adding. Same format as `vite` in `wxt.config.ts`.
```

## `packages/wxt/src/types.ts`
```diff
@@ -113,8 +113,7 @@
-   * @default
-   * "chrome"
+   * @default "chrome"

@@ -134,8 +134,7 @@
-   * @default
-   * consola
+   * @default consola

@@ -342,7 +342,7 @@
    * @example
-   *   { "testing": "src/utils/testing.ts" }
+   *   { testing: "src/utils/testing.ts" }

@@ -405,7 +405,7 @@
-   * [https://vitejs.dev/config/shared-options.html](https://vitejs.dev/config/shared-options.html).
+   * https://vitejs.dev/config/shared-options.html.

@@ -1064,19 +1064,18 @@
-   * @default
-   * // Enable dev mode and allow content script sourcemaps
+   * @default // Enable dev mode and allow content script sourcemaps
    * {
-   *   devtools: {
-   *     ...
+   * devtools: {
+   * ...
(indentation of nested object lost)

@@ -1531,8 +1531,7 @@
-   * See
-   * [https://wxt.dev/guide/key-concepts/auto-imports.html#eslint](https://wxt.dev/guide/key-concepts/auto-imports.html#eslint)
+   * See https://wxt.dev/guide/key-concepts/auto-imports.html#eslint
```

## `packages/wxt/src/utils/content-script-context.ts`
```diff
@@ -238,8 +238,8 @@
-   * @internal
-   * Abort the abort controller and execute all `onInvalidated` listeners.
+   * @internal Abort the abort controller and execute all `onInvalidated`
+   *   listeners.
```
