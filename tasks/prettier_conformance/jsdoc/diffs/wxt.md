# JSDoc Diffs: wxt (round 6, 2026-03-17)

1 file with diffs, 1,183 JSDoc tags

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
