# JSDoc Diffs: typedoc

Date: 2026-03-15
Prettier version: 3.8.1
JSDoc tags: 792
Files with diffs: 14

## `example/src/classes/CancellablePromise.ts`

```diff
diff --git a/example/src/classes/CancellablePromise.ts b/example/src/classes/CancellablePromise.ts
index 0fbb73b..c59cce0 100644
--- a/example/src/classes/CancellablePromise.ts
+++ b/example/src/classes/CancellablePromise.ts
@@ -32,8 +32,8 @@ function isPromiseWithCancel<T>(value: unknown): value is PromiseWithCancel<T> {
  * - A method with 10 overload signatures. Wow!
  *
  *   - Only the implementation signature has a doc comment. TypeDoc automatically
- *       copies the comment from the implementation signature to each of the
- *       visible signatures if they don't have one.
+ *     copies the comment from the implementation signature to each of the visible
+ *     signatures if they don't have one.
  *
  * A promise with a `cancel` method. If canceled, the `CancellablePromise` will
  * reject with a `Cancellation` object. Originally from
```

## `scripts/generate_options_schema.js`

```diff
diff --git a/scripts/generate_options_schema.js b/scripts/generate_options_schema.js
index bf0d5c4..3b6722c 100644
--- a/scripts/generate_options_schema.js
+++ b/scripts/generate_options_schema.js
@@ -109,8 +109,8 @@ addTypeDocOptions({
                 };
                 const defaults = /**
                  * @type {import("../dist/index.js").FlagsDeclarationOption<
-                 *         Record<string, boolean>
-                 *     >}
+                 *     Record<string, boolean>
+                 * >}
                  */ (option).defaults;
 
                 for (const key of Object.keys(defaults)) {
```

## `src/lib/converter/comments/textParser.ts`

```diff
diff --git a/src/lib/converter/comments/textParser.ts b/src/lib/converter/comments/textParser.ts
index 7a39ced..16c4d82 100644
--- a/src/lib/converter/comments/textParser.ts
+++ b/src/lib/converter/comments/textParser.ts
@@ -76,7 +76,8 @@ export class TextParserReentryState {
 
 /**
  * Look for relative links within a piece of text and add them to the
- * {@link FileRegistry} so that they can be correctly resolved during rendering.
+ * {@link FileRegistry} so that they can be correctly resolved during
+ * rendering.
  */
 export function textContent(
     parserData: Omit<TextParserData, "pos">,
```

## `src/lib/converter/converter.ts`

```diff
diff --git a/src/lib/converter/converter.ts b/src/lib/converter/converter.ts
index aa8824b..2ea0ce7 100644
--- a/src/lib/converter/converter.ts
+++ b/src/lib/converter/converter.ts
@@ -101,7 +101,8 @@ export interface ConverterEvents {
  * reflections.
  *
  * @group None
- * @summary Responsible for converting TypeScript symbols into {@link Reflection}s and {@link Type}s.
+ * @summary Responsible for converting TypeScript symbols into
+ *   {@link Reflection}s and {@link Type}s.
  */
 export class Converter extends AbstractComponent<Application, ConverterEvents> {
     /** @internal */
```

## `src/lib/models/ReflectionSymbolId.ts`

```diff
diff --git a/src/lib/models/ReflectionSymbolId.ts b/src/lib/models/ReflectionSymbolId.ts
index 83e26fd..e54e74a 100644
--- a/src/lib/models/ReflectionSymbolId.ts
+++ b/src/lib/models/ReflectionSymbolId.ts
@@ -57,8 +57,8 @@ export class ReflectionSymbolId {
      * project.
      *
      * @privateRemarks
-     *   This is used by typedoc-plugin-dt-links to determine the path to read to
-     *   get the source code of a definitely typed package.
+     *   This is used by typedoc-plugin-dt-links to determine the path to read
+     *   to get the source code of a definitely typed package.
      */
     fileName?: NormalizedPath;
```

## `src/lib/output/renderer.ts`

```diff
diff --git a/src/lib/output/renderer.ts b/src/lib/output/renderer.ts
index d61fb6e..1dd826e 100644
--- a/src/lib/output/renderer.ts
+++ b/src/lib/output/renderer.ts
@@ -122,19 +122,19 @@ export interface RendererEvents {
  * specify which theme should be used using the `--theme <name>` command line
  * argument.
  *
- * {@link Renderer} is a subclass of {@link EventDispatcher} and triggers a series
- * of events while a project is being processed. You can listen to these events
- * to control the flow or manipulate the output.
+ * {@link Renderer} is a subclass of {@link EventDispatcher} and triggers a
+ * series of events while a project is being processed. You can listen to these
+ * events to control the flow or manipulate the output.
  *
  * - {@link Renderer.EVENT_BEGIN}<br> Triggered before the renderer starts
  *   rendering a project. The listener receives an instance of
  *   {@link RendererEvent}.
  *
  *   - {@link Renderer.EVENT_BEGIN_PAGE}<br> Triggered before a document will be
- *       rendered. The listener receives an instance of {@link PageEvent}.
+ *     rendered. The listener receives an instance of {@link PageEvent}.
  *   - {@link Renderer.EVENT_END_PAGE}<br> Triggered after a document has been
- *       rendered, just before it is written to disc. The listener receives an
- *       instance of {@link PageEvent}.
+ *     rendered, just before it is written to disc. The listener receives an
+ *     instance of {@link PageEvent}.
  * - {@link Renderer.EVENT_END}<br> Triggered after the renderer has written all
  *   documents. The listener receives an instance of {@link RendererEvent}.
  * - {@link Renderer.EVENT_PREPARE_INDEX}<br> Triggered when the
```

## `src/lib/utils/html.ts`

```diff
diff --git a/src/lib/utils/html.ts b/src/lib/utils/html.ts
index b202692..abb94f1 100644
--- a/src/lib/utils/html.ts
+++ b/src/lib/utils/html.ts
@@ -146,8 +146,8 @@ export const enum ParserState {
 }
 
 /**
- * Parser for HTML attributes, each call to {@link step} will pause the parser at
- * key points used to extract relative links from markdown
+ * Parser for HTML attributes, each call to {@link step} will pause the parser
+ * at key points used to extract relative links from markdown
  *
  * The parser will pause at the points marked with `^`:
  *
```

## `src/test/converter/comment/comment.ts`

```diff
diff --git a/src/test/converter/comment/comment.ts b/src/test/converter/comment/comment.ts
index e8c342f..7f29d4a 100644
--- a/src/test/converter/comment/comment.ts
+++ b/src/test/converter/comment/comment.ts
@@ -29,7 +29,7 @@ import "./comment2";
  *     }
  *
  * @deprecated
- * @type {Data<object>} will Also be removed
+ * @type {Data<object>} Will Also be removed
  * @groupDescription Methods
  * Methods description!
  *
```

## `src/test/converter/function/function.ts`

```diff
diff --git a/src/test/converter/function/function.ts b/src/test/converter/function/function.ts
index 5f6b105..83e88fd 100644
--- a/src/test/converter/function/function.ts
+++ b/src/test/converter/function/function.ts
@@ -11,6 +11,7 @@ export function exportedFunction(): void {}
  *
  *       const value: BaseClass = new BaseClass("test");
  *       functionWithArguments("arg", 0, value);
+ *
  * @returns This is the return value of the function.
  */
 export function functionWithParameters(
```

## `src/test/converter/inheritance/inherit-doc.ts`

```diff
diff --git a/src/test/converter/inheritance/inherit-doc.ts b/src/test/converter/inheritance/inherit-doc.ts
index 5276309..77cc7ae 100644
--- a/src/test/converter/inheritance/inherit-doc.ts
+++ b/src/test/converter/inheritance/inherit-doc.ts
@@ -40,8 +40,8 @@ export interface InterfaceTarget<T> {
  *   Remarks will be inherited
  * @default
  *
- * This part of the commentary will not be inherited (this is an abuse of this tag)
- *
+ * This part of the commentary will not be inherited (this is an abuse of this
+ * tag)
  * @typeParam T - Type of arguments
  * @param arg1 - First argument
  * @param arg2 - Second argument
```

## `src/test/converter2/behavior/includeTag/duplicateRegion.ts`

```diff
diff --git a/src/test/converter2/behavior/includeTag/duplicateRegion.ts b/src/test/converter2/behavior/includeTag/duplicateRegion.ts
index 5d508ec..1db806e 100644
--- a/src/test/converter2/behavior/includeTag/duplicateRegion.ts
+++ b/src/test/converter2/behavior/includeTag/duplicateRegion.ts
@@ -7,7 +7,7 @@
 // #endregion dupEnd
 
 /**
- * {@includeCode duplicateRegion.ts#dupStart} {@includeCode
- * duplicateRegion.ts#dupEnd} {@includeCode foldAndRegion.java#dup}
+ * {@includeCode duplicateRegion.ts#dupStart}
+ * {@includeCode duplicateRegion.ts#dupEnd} {@includeCode foldAndRegion.java#dup}
  */
 export const includeWarnings = 123;
```

## `src/test/converter2/behavior/includeTag/invalidLineRanges.ts`

```diff
diff --git a/src/test/converter2/behavior/includeTag/invalidLineRanges.ts b/src/test/converter2/behavior/includeTag/invalidLineRanges.ts
index abf886e..b460dd8 100644
--- a/src/test/converter2/behavior/includeTag/invalidLineRanges.ts
+++ b/src/test/converter2/behavior/includeTag/invalidLineRanges.ts
@@ -1,5 +1,6 @@
 /**
- * {@includeCode invalidLineRanges.ts:100-200} {@includeCode
- * invalidLineRanges.ts:200-100} {@includeCode invalidLineRanges.ts:300}
+ * {@includeCode invalidLineRanges.ts:100-200}
+ * {@includeCode invalidLineRanges.ts:200-100}
+ * {@includeCode invalidLineRanges.ts:300}
  */
 export const includeWarnings = 456;
```

## `src/test/converter2/behavior/includeTag/missingRegion.ts`

```diff
diff --git a/src/test/converter2/behavior/includeTag/missingRegion.ts b/src/test/converter2/behavior/includeTag/missingRegion.ts
index d126281..30ea907 100644
--- a/src/test/converter2/behavior/includeTag/missingRegion.ts
+++ b/src/test/converter2/behavior/includeTag/missingRegion.ts
@@ -2,7 +2,8 @@
 // #endregion missingStart
 
 /**
- * {@includeCode missingRegion.ts#noRegion} {@includeCode
- * missingRegion.ts#missingStart} {@includeCode missingRegion.ts#missingEnd}
+ * {@includeCode missingRegion.ts#noRegion}
+ * {@includeCode missingRegion.ts#missingStart}
+ * {@includeCode missingRegion.ts#missingEnd}
  */
 export const includeWarnings = 456;
```

## `src/test/converter2/behavior/linkResolution.ts`

```diff
diff --git a/src/test/converter2/behavior/linkResolution.ts b/src/test/converter2/behavior/linkResolution.ts
index aad4bbd..86a23ff 100644
--- a/src/test/converter2/behavior/linkResolution.ts
+++ b/src/test/converter2/behavior/linkResolution.ts
@@ -90,7 +90,10 @@ export namespace Globals {
     export const A = 2;
 }
 
-/** {@link Navigation~Child.foo} {@link Navigation.Child#foo} {@link Child~foo} bad */
+/**
+ * {@link Navigation~Child.foo} {@link Navigation.Child#foo} {@link Child~foo}
+ * bad
+ */
 export namespace Navigation {
     export class Child {
         /** {@link foo} Child.foo, not Child#foo */
```
