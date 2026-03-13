# JSDoc Diffs: typedoc

Prettier version: 3.8.1
Files with diffs: 25

## `example/src/classes/CancellablePromise.ts`
```diff
@@ -30,10 +30,9 @@ function isPromiseWithCancel<T>(value: unknown): value is PromiseWithCancel<T> {
  * - Complex method signatures
  * - Static methods
  * - A method with 10 overload signatures. Wow!
- *
  *   - Only the implementation signature has a doc comment. TypeDoc automatically
-     *       copies the comment from the implementation signature to each of the
-     *       visible signatures if they don't have one.
+     copies the comment from the implementation signature to each of the visible
+     signatures if they don't have one.
  *
  * A promise with a `cancel` method. If canceled, the `CancellablePromise` will
  * reject with a `Cancellation` object. Originally from
```

## `example/src/internals.ts`
```diff
@@ -1,9 +1,9 @@
 /**
  * @internal
  *
- * Use `@internal` to indicate that something is for internal use. If the
- * `--excludeInternal` option is passed, TypeDoc will not document the given
- * code.
+ *   Use `@internal` to indicate that something is for internal use. If the
+ *   `--excludeInternal` option is passed, TypeDoc will not document the given
+ *   code.
  */
 export function anInternalFunction(): void {
     // does nothing
```

## `src/lib/application.ts`
```diff
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
@@ -76,7 +76,8 @@

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
@@ -212,9 +212,9 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {

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
@@ -225,8 +225,8 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {

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
@@ -122,19 +122,18 @@ export interface RendererEvents {
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
- *
  *   - {@link Renderer.EVENT_BEGIN_PAGE}<br> Triggered before a document will be
-     *       rendered. The listener receives an instance of {@link PageEvent}.
+     rendered. The listener receives an instance of {@link PageEvent}.
  *   - {@link Renderer.EVENT_END_PAGE}<br> Triggered after a document has been
-     *       rendered, just before it is written to disc. The listener receives an
-     *       instance of {@link PageEvent}.
+     rendered, just before it is written to disc. The listener receives an
+     instance of {@link PageEvent}.
  * - {@link Renderer.EVENT_END}<br> Triggered after the renderer has written all
  *   documents. The listener receives an instance of {@link RendererEvent}.
  * - {@link Renderer.EVENT_PREPARE_INDEX}<br> Triggered when the
```

## `src/lib/output/router.ts`
```diff
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

## `src/lib/output/themes/default/assets/typedoc/utils/modal.ts`
```diff
@@ -1,19 +1,16 @@
 /**
  * @module
  *
- * Browsers allow scrolling of page with native dialog, which is a UX issue.
+ *   Browsers allow scrolling of page with native dialog, which is a UX issue.
+ *   `@starting-style` and `overlay` aren't well supported in FF, and only
+ *   available in latest versions of chromium, hence, a custom overlay
+ *   workaround is required. Workaround:
  *
- * `@starting-style` and `overlay` aren't well supported in FF, and only available in latest versions of chromium,
- * hence, a custom overlay workaround is required.
- *
- * Workaround:
- *
- * - Append a custom overlay element (a div) to `document.body`,
- *   this does **NOT** handle nested modals,
- *   as the overlay div cannot be in the top layer, which wouldn't overshadow the parent modal.
- *
- * - Add exit animation on dialog and overlay, without actually closing them
- * - Listen for `animationend` event, and close the modal immediately
+ *   - Append a custom overlay element (a div) to `document.body`, this does
+ *     **NOT** handle nested modals, as the overlay div cannot be in the top
+ *     layer, which wouldn't overshadow the parent modal.
+ *   - Add exit animation on dialog and overlay, without actually closing them
+ *   - Listen for `animationend` event, and close the modal immediately
  *
  * @see
  * - The "[right](https://frontendmasters.com/blog/animating-dialog/)" way to animate modals
```

## `src/lib/serialization/components.ts`
```diff
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
@@ -16,8 +16,10 @@ import "./comment2";
  *
  * An example with decorators that should not parse to tag
  *
- *     @myDecorator
- *     @FactoryDecorator("a", "b", "c")
+ * @deprecated
+ * @type {Data<object>} will Also be removed
+ * @myDecorator
+ * @FactoryDecorator("a", "b", "c")
  *     export class CommentedClass {
  *         myProp: string = "myProp";
  *
@@ -28,8 +30,6 @@ import "./comment2";
  *         myMethod() {}
  *     }
  *
- * @deprecated
- * @type {Data<object>} will Also be removed
  * @groupDescription Methods
  * Methods description!
  *
```

## `src/test/converter/function/function.ts`
```diff
@@ -9,8 +9,9 @@ export function exportedFunction(): void {}
  *   in the next line.
  * @param paramA This is a **parameter** pointing to an interface.
  *
- *       const value: BaseClass = new BaseClass("test");
- *       functionWithArguments("arg", 0, value);
+ *   const value: BaseClass = new BaseClass("test");
+ *   functionWithArguments("arg", 0, value);
+ *
  * @returns This is the return value of the function.
  */
 export function functionWithParameters(
```

## `src/test/converter/inheritance/inherit-doc.ts`
```diff
@@ -40,7 +40,8 @@ export interface InterfaceTarget<T> {
  *   Remarks will be inherited
  * @default
  *
- * This part of the commentary will not be inherited (this is an abuse of this tag)
+ *   This part of the commentary will not be inherited (this is an abuse of this
+ *   tag)
  *
  * @typeParam T - Type of arguments
  * @param arg1 - First argument
```

## `src/test/converter2/behavior/includeTag/duplicateRegion.ts`
```diff
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

## `src/test/converter2/issues/gh1896.js`
```diff
@@ -10,9 +10,7 @@
 /**
  * Before tag
  *
- * @typedef {{ (one: number, two: number): number }} Type2
- *
- *   Some type 2.
+ * @typedef {{ (one: number, two: number): number }} Type2 Some type 2.
  */

 export const answer = 42;
```

## `src/test/converter2/issues/gh2384.js`
```diff
@@ -1,6 +1,6 @@
 /**
  * @typedef {Int8Array | Uint8Array} TypedArray A union type representing all
- *   possible TypedArrays.
+ * possible TypedArrays.
  */

 /**
```

## `src/test/converter2/issues/gh671.js`
```diff
@@ -1,7 +1,8 @@
 /**
  * @param {string} x The string to parse as a number
- * @param {boolean} [int=true] Whether to parse as an integer or float. Default
- *   is `true`
+ * @param {boolean} [int=true] Whether to parse as an integer or float
+ *
+ *   Default is `true`
  */
 export function toNumber(x, int = true) {
     return int ? parseInt(x) : parseFloat(x);
```
