# JSDoc Diffs: wxt

Date: 2026-03-15 (round 5)
Prettier version: 3.8.1
JSDoc tags: 1183
Files with diffs: 1

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

### Diff Analysis

**Category: Indentation normalization in nested markdown lists**

oxfmt reduces over-indentation (6 spaces to 4 spaces) in nested list item continuation lines within JSDoc block comments. Also re-wraps one long line that was previously split.

Previously seen diff in `packages/wxt/src/modules.ts` (round 4) is now gone -- that @param indentation issue has been fixed.
