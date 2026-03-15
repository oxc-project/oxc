# JSDoc Diffs: typedoc

Date: 2026-03-15 (round 5)
Prettier version: 3.8.1
JSDoc tags: 792
Files with diffs: 13

## `example/src/classes/CancellablePromise.ts`

```diff
@@ -32,8 +32,8 @@ function isPromiseWithCancel<T>(value: unknown): value is PromiseWithCancel<T> {
  *   - Only the implementation signature has a doc comment. TypeDoc automatically
- *       copies the comment from the implementation signature to each of the
- *       visible signatures if they don't have one.
+ *     copies the comment from the implementation signature to each of the visible
+ *     signatures if they don't have one.
```

## `scripts/generate_options_schema.js`

```diff
@@ -109,8 +109,8 @@ addTypeDocOptions({
                 const defaults = /**
                  * @type {import("../dist/index.js").FlagsDeclarationOption<
-                 *         Record<string, boolean>
-                 *     >}
+                 *     Record<string, boolean>
+                 * >}
                  */ (option).defaults;
```

## `src/lib/converter/comments/textParser.ts`

```diff
@@ -76,7 +76,8 @@ export class TextParserReentryState {
  * Look for relative links within a piece of text and add them to the
- * {@link FileRegistry} so that they can be correctly resolved during rendering.
+ * {@link FileRegistry} so that they can be correctly resolved during
+ * rendering.
```

## `src/lib/models/ReflectionSymbolId.ts`

```diff
@@ -57,8 +57,8 @@ export class ReflectionSymbolId {
  * @privateRemarks
-     *   This is used by typedoc-plugin-dt-links to determine the path to read to
-     *   get the source code of a definitely typed package.
+     *   This is used by typedoc-plugin-dt-links to determine the path to read
+     *   to get the source code of a definitely typed package.
```

## `src/lib/output/renderer.ts`

```diff
@@ -122,19 +122,19 @@ export interface RendererEvents {
- * {@link Renderer} is a subclass of {@link EventDispatcher} and triggers a series
- * of events while a project is being processed. You can listen to these events
- * to control the flow or manipulate the output.
+ * {@link Renderer} is a subclass of {@link EventDispatcher} and triggers a
+ * series of events while a project is being processed. You can listen to these
+ * events to control the flow or manipulate the output.
 *
 *   - {@link Renderer.EVENT_BEGIN_PAGE}<br> Triggered before a document will be
- *       rendered. The listener receives an instance of {@link PageEvent}.
+ *     rendered. The listener receives an instance of {@link PageEvent}.
 *   - {@link Renderer.EVENT_END_PAGE}<br> Triggered after a document has been
- *       rendered, just before it is written to disc. The listener receives an
- *       instance of {@link PageEvent}.
+ *     rendered, just before it is written to disc. The listener receives an
+ *     instance of {@link PageEvent}.
```

## `src/lib/utils/html.ts`

```diff
@@ -146,8 +146,8 @@ export const enum ParserState {
- * Parser for HTML attributes, each call to {@link step} will pause the parser at
- * key points used to extract relative links from markdown
+ * Parser for HTML attributes, each call to {@link step} will pause the parser
+ * at key points used to extract relative links from markdown
```

## `src/test/converter/comment/comment.ts`

```diff
@@ -29,7 +29,7 @@ import "./comment2";
 * @deprecated
- * @type {Data<object>} will Also be removed
+ * @type {Data<object>} Will Also be removed
```

## `src/test/converter/function/function.ts`

```diff
@@ -11,6 +11,7 @@ export function exportedFunction(): void {}
 *       functionWithArguments("arg", 0, value);
+ *
 * @returns This is the return value of the function.
```

## `src/test/converter/inheritance/inherit-doc.ts`

```diff
@@ -40,8 +40,8 @@ export interface InterfaceTarget<T> {
 * @default
 *
- * This part of the commentary will not be inherited (this is an abuse of this tag)
- *
+ * This part of the commentary will not be inherited (this is an abuse of this
+ * tag)
 * @typeParam T - Type of arguments
```

## `src/test/converter2/behavior/includeTag/duplicateRegion.ts`

```diff
@@ -7,7 +7,7 @@
- * {@includeCode duplicateRegion.ts#dupStart} {@includeCode
- * duplicateRegion.ts#dupEnd} {@includeCode foldAndRegion.java#dup}
+ * {@includeCode duplicateRegion.ts#dupStart}
+ * {@includeCode duplicateRegion.ts#dupEnd} {@includeCode foldAndRegion.java#dup}
```

## `src/test/converter2/behavior/includeTag/invalidLineRanges.ts`

```diff
@@ -1,5 +1,6 @@
- * {@includeCode invalidLineRanges.ts:100-200} {@includeCode
- * invalidLineRanges.ts:200-100} {@includeCode invalidLineRanges.ts:300}
+ * {@includeCode invalidLineRanges.ts:100-200}
+ * {@includeCode invalidLineRanges.ts:200-100}
+ * {@includeCode invalidLineRanges.ts:300}
```

## `src/test/converter2/behavior/includeTag/missingRegion.ts`

```diff
@@ -2,7 +2,8 @@
- * {@includeCode missingRegion.ts#noRegion} {@includeCode
- * missingRegion.ts#missingStart} {@includeCode missingRegion.ts#missingEnd}
+ * {@includeCode missingRegion.ts#noRegion}
+ * {@includeCode missingRegion.ts#missingStart}
+ * {@includeCode missingRegion.ts#missingEnd}
```

## `src/test/converter2/behavior/linkResolution.ts`

```diff
@@ -90,7 +90,10 @@ export namespace Globals {
-/** {@link Navigation~Child.foo} {@link Navigation.Child#foo} {@link Child~foo} bad */
+/**
+ * {@link Navigation~Child.foo} {@link Navigation.Child#foo} {@link Child~foo}
+ * bad
+ */
```
