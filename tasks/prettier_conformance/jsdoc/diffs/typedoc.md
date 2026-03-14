# JSDoc Diffs: typedoc

Date: 2026-03-14
Prettier version: 3.8.1
JSDoc tags: 792
Files with diffs: 23

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

## `example/src/index.ts`

```diff
diff --git a/example/src/index.ts b/example/src/index.ts
index 24aa604..f9b2c10 100644
--- a/example/src/index.ts
+++ b/example/src/index.ts
@@ -3,7 +3,6 @@
  * @categoryDescription Component
  * React Components -- This description is added with the `@categoryDescription` tag
  * on the entry point in src/index.ts
- *
  * @document documents/external-markdown.md
  * @document documents/markdown.md
  * @document documents/syntax-highlighting.md
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

## `src/index.ts`

```diff
diff --git a/src/index.ts b/src/index.ts
index 6e25ce8..66ad112 100644
--- a/src/index.ts
+++ b/src/index.ts
@@ -18,14 +18,11 @@ export { resetReflectionID } from "./lib/models/Reflection.js";
  * root import.
  *
  * @primaryExport
- *
  * @categoryDescription Types
  * Describes a TypeScript type.
- *
  * @categoryDescription Reflections
  * Describes a documentation entry. The root entry is a {@link ProjectReflection}
  * and contains {@link DeclarationReflection} instances.
- *
  * @summary
  * TypeDoc converts source code into these object types.
  */
```

## `src/lib/application.ts`

```diff
diff --git a/src/lib/application.ts b/src/lib/application.ts
index 8485ba8..5385e5c 100644
--- a/src/lib/application.ts
+++ b/src/lib/application.ts
@@ -101,12 +101,13 @@ export interface ApplicationEvents {
 /**
  * The default TypeDoc main application class.
  *
- * This class holds the two main components of TypeDoc, the {@link Converter} and
- * the {@link Renderer}. When running TypeDoc, first the {@link Converter} is
- * invoked which generates a {@link ProjectReflection} from the passed in source
- * files. The {@link ProjectReflection} is a hierarchical model representation of
- * the TypeScript project. Afterwards the model is passed to the {@link Renderer}
- * which uses an instance of {@link Theme} to generate the final documentation.
+ * This class holds the two main components of TypeDoc, the {@link Converter}
+ * and the {@link Renderer}. When running TypeDoc, first the {@link Converter}
+ * is invoked which generates a {@link ProjectReflection} from the passed in
+ * source files. The {@link ProjectReflection} is a hierarchical model
+ * representation of the TypeScript project. Afterwards the model is passed to
+ * the {@link Renderer} which uses an instance of {@link Theme} to generate the
+ * final documentation.
  *
  * Both the {@link Converter} and the {@link Renderer} emit a series of events
  * while processing the project. Subscribe to these Events to control the
@@ -114,8 +115,8 @@ export interface ApplicationEvents {
  *
  * @remarks
  *   Access to an Application instance can be retrieved with
- *   {@link Application.bootstrap} or {@link Application.bootstrapWithPlugins}. It
- *   can not be constructed manually.
+ *   {@link Application.bootstrap} or {@link Application.bootstrapWithPlugins}.
+ *   It can not be constructed manually.
  * @group None
  * @summary Root level class which contains most useful behavior.
  */
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
index aa8824b..c65af07 100644
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
@@ -212,9 +213,9 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {
 
     /**
      * Triggered when the converter has created a signature reflection. The
-     * listener will be given {@link Context}, {@link Models.SignatureReflection}
-     * | {@link Models.ProjectReflection} the declaration,
-     * `ts.SignatureDeclaration | ts.IndexSignatureDeclaration |
+     * listener will be given {@link Context},
+     * {@link Models.SignatureReflection} | {@link Models.ProjectReflection} the
+     * declaration, `ts.SignatureDeclaration | ts.IndexSignatureDeclaration |
      * ts.JSDocSignature | undefined`, and `ts.Signature | undefined`. The
      * signature will be undefined if the created signature is an index
      * signature.
@@ -225,8 +226,8 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {
 
     /**
      * Triggered when the converter has created a parameter reflection. The
-     * listener will be given {@link Context}, {@link Models.ParameterReflection}
-     * and a `ts.Node?`
+     * listener will be given {@link Context},
+     * {@link Models.ParameterReflection} and a `ts.Node?`
      *
      * @event
      */
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

## `src/lib/output/router.ts`

```diff
diff --git a/src/lib/output/router.ts b/src/lib/output/router.ts
index 1138c1c..27c4f2a 100644
--- a/src/lib/output/router.ts
+++ b/src/lib/output/router.ts
@@ -123,8 +123,8 @@ export interface Router {
 /**
  * Base router class intended to make it easier to implement a router.
  *
- * Child classes need only {@link getIdealBaseName}, this class will take care of
- * the recursing through child reflections.
+ * Child classes need only {@link getIdealBaseName}, this class will take care
+ * of the recursing through child reflections.
  *
  * @group Routers
  */
```

## `src/lib/output/themes/default/DefaultThemeRenderContext.ts`

```diff
diff --git a/src/lib/output/themes/default/DefaultThemeRenderContext.ts b/src/lib/output/themes/default/DefaultThemeRenderContext.ts
index 04697d8..8b0fea1 100644
--- a/src/lib/output/themes/default/DefaultThemeRenderContext.ts
+++ b/src/lib/output/themes/default/DefaultThemeRenderContext.ts
@@ -160,9 +160,9 @@ export class DefaultThemeRenderContext {
     typeDetails = bind(typeDetails, this);
 
     /**
-     * Should call the {@link typeDetails} helper if rendering additional details
-     * about the type will provide the user with more information about the
-     * type.
+     * Should call the {@link typeDetails} helper if rendering additional
+     * details about the type will provide the user with more information about
+     * the type.
      */
     typeDetailsIfUseful = bind(typeDetailsIfUseful, this);
 
```

## `src/lib/serialization/components.ts`

```diff
diff --git a/src/lib/serialization/components.ts b/src/lib/serialization/components.ts
index 21da59e..85fe8d8 100644
--- a/src/lib/serialization/components.ts
+++ b/src/lib/serialization/components.ts
@@ -4,9 +4,9 @@ import type { ModelToObject } from "./schema.js";
 /**
  * Represents Serializer plugin component.
  *
- * Like Converter plugins each {@link Serializer} plugin defines a predicate that
- * instructs if an object can be serialized by it. This is done dynamically at
- * runtime via a `supports` method.
+ * Like Converter plugins each {@link Serializer} plugin defines a predicate
+ * that instructs if an object can be serialized by it. This is done dynamically
+ * at runtime via a `supports` method.
  */
 export interface SerializerComponent<T extends object> {
     /**
```

## `src/lib/serialization/serializer.ts`

```diff
diff --git a/src/lib/serialization/serializer.ts b/src/lib/serialization/serializer.ts
index 3cbbb48..8b6f1f8 100644
--- a/src/lib/serialization/serializer.ts
+++ b/src/lib/serialization/serializer.ts
@@ -30,7 +30,8 @@ export class Serializer extends EventDispatcher<SerializerEvents> {
     static readonly EVENT_BEGIN = "begin";
 
     /**
-     * Triggered when the {@link Serializer} has finished transforming a project.
+     * Triggered when the {@link Serializer} has finished transforming a
+     * project.
      *
      * @event
      */
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

## `src/lib/utils/options/declaration.ts`

```diff
diff --git a/src/lib/utils/options/declaration.ts b/src/lib/utils/options/declaration.ts
index d7205e3..ab76638 100644
--- a/src/lib/utils/options/declaration.ts
+++ b/src/lib/utils/options/declaration.ts
@@ -154,9 +154,9 @@ export type TypeDocOptions = {
 /**
  * Describes all TypeDoc specific options as returned by
  * {@link Options.getValue}, this is slightly more restrictive than the
- * {@link TypeDocOptions} since it does not allow both keys and values for mapped
- * option types, and does not allow partials of flag values. It also does not
- * mark keys as optional.
+ * {@link TypeDocOptions} since it does not allow both keys and values for
+ * mapped option types, and does not allow partials of flag values. It also does
+ * not mark keys as optional.
  *
  * @interface
  */
```

## `src/test/converter/comment/comment.ts`

```diff
diff --git a/src/test/converter/comment/comment.ts b/src/test/converter/comment/comment.ts
index e8c342f..0fa23c9 100644
--- a/src/test/converter/comment/comment.ts
+++ b/src/test/converter/comment/comment.ts
@@ -29,10 +29,9 @@ import "./comment2";
  *     }
  *
  * @deprecated
- * @type {Data<object>} will Also be removed
+ * @type {Data<object>} Will Also be removed
  * @groupDescription Methods
  * Methods description!
- *
  * @gh3020 {type annotation}
  */
 export class CommentedClass {
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
index 5d508ec..b0a96f2 100644
--- a/src/test/converter2/behavior/includeTag/duplicateRegion.ts
+++ b/src/test/converter2/behavior/includeTag/duplicateRegion.ts
@@ -7,7 +7,8 @@
 // #endregion dupEnd
 
 /**
- * {@includeCode duplicateRegion.ts#dupStart} {@includeCode
- * duplicateRegion.ts#dupEnd} {@includeCode foldAndRegion.java#dup}
+ * {@includeCode duplicateRegion.ts#dupStart}
+ * {@includeCode duplicateRegion.ts#dupEnd}
+ * {@includeCode foldAndRegion.java#dup}
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

## `src/test/converter2/behavior/linkResolutionErrors.ts`

```diff
diff --git a/src/test/converter2/behavior/linkResolutionErrors.ts b/src/test/converter2/behavior/linkResolutionErrors.ts
index 90227ea..c870a29 100644
--- a/src/test/converter2/behavior/linkResolutionErrors.ts
+++ b/src/test/converter2/behavior/linkResolutionErrors.ts
@@ -1,7 +1,7 @@
 /**
  * {@link Map.size} TS resolves link, not included in docs #2700 #2967
  * {@link DoesNotExist} Symbol does not exist #2681
- * {@link @typedoc/foo.DoesNotExist} Symbol does not exist, looks like an attempt
- * to link to a package directly #2360
+ * {@link @typedoc/foo.DoesNotExist} Symbol does not exist, looks like an
+ * attempt to link to a package directly #2360
  */
 export const abc = new Map<string, number>();
```

