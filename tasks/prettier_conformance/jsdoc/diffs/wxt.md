# JSDoc Diffs: wxt

Date: 2026-03-15 (round 4)
Prettier version: 3.8.1
JSDoc tags: 1218
Files with diffs: 2

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
```

## `packages/wxt/src/modules.ts`

```diff
@@ -102,8 +102,8 @@ export function addPublicAssets(wxt: Wxt, dir: string): void {
  *   });
  *   });
  *
- * @param wxt The wxt instance provided by the module's setup function.
- * @param viteConfig A function that returns the vite config the module is
+ *   @param wxt The wxt instance provided by the module's setup function.
+ *   @param viteConfig A function that returns the vite config the module is
  *   adding. Same format as `vite` in `wxt.config.ts`.
```
