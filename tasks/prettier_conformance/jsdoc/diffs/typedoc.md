# JSDoc Diffs: typedoc

Total JSDoc tags: 791
Files with diffs: 37

## Diff categories

### 1. Block tag description placement (most common)
oxfmt puts the description on the same line as the tag name, prettier-plugin-jsdoc puts it on the next line.
Affects: `@privateRemarks`, `@categoryDescription`, `@groupDescription`, `@summary`, `@see`, `@remarks`, `@typedef`, `@fires`

### 2. `@remarks` indent changed from 2-space to 4-space
oxfmt uses 4-space indent for `@remarks` body, prettier uses 2-space.
Also adds trailing semicolons to single-word remarks bodies.

### 3. Code block in indented context destroyed
In `comment.ts`, a 4-space-indented code block (markdown code block via indentation) was treated as tag description text and collapsed.

### 4. Capitalization bug
`@hidden` -> `@hidden` And (capitalized 'And')
`id:` -> `Id:` (capitalized inside type annotation)

### 5. Blank line removal between tags
oxfmt removes blank lines between consecutive tags in some cases where prettier preserves them.

### 6. Pipe `|` in description treated as table separator
In `converter.ts`, a `|` character in prose caused blank lines to be inserted around it.

### 7. `@template` tag duplicates default description
In `gh2384.js` and `gh671.js`, text after line-wrapped `@template`/`@param` default values gets duplicated.

## Full diff

```diff
diff --git a/example/src/classes/CancellablePromise.ts b/example/src/classes/CancellablePromise.ts
index 0fbb73b..6c3d8e5 100644
--- a/example/src/classes/CancellablePromise.ts
+++ b/example/src/classes/CancellablePromise.ts
@@ -32,19 +32,19 @@ function isPromiseWithCancel<T>(value: unknown): value is PromiseWithCancel<T> {
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
  * [real-cancellable-promise](https://github.com/srmagura/real-cancellable-promise).
  *
  * @typeParam T What the `CancellablePromise` resolves to
- * @groupDescription Methods
- * Descriptions can be added for groups with `@groupDescription`, which will show up in
- * the index where groups are listed. This works for both manually created groups which
- * are created with `@group`, and implicit groups like the `Methods` group that this
- * description is attached to.
+ * @groupDescription Methods Descriptions can be added for groups with
+ *   `@groupDescription`, which will show up in the index where groups are
+ *   listed. This works for both manually created groups which are created with
+ *   `@group`, and implicit groups like the `Methods` group that this
+ *   description is attached to.
  */
 export class CancellablePromise<T> {
     /**
diff --git a/example/src/index.ts b/example/src/index.ts
index 24aa604..663a875 100644
--- a/example/src/index.ts
+++ b/example/src/index.ts
@@ -1,9 +1,7 @@
 /**
  * @packageDocumentation
- * @categoryDescription Component
- * React Components -- This description is added with the `@categoryDescription` tag
- * on the entry point in src/index.ts
- *
+ * @categoryDescription Component React Components -- This description is added
+ *   with the `@categoryDescription` tag on the entry point in src/index.ts
  * @document documents/external-markdown.md
  * @document documents/markdown.md
  * @document documents/syntax-highlighting.md
diff --git a/example/src/internals.ts b/example/src/internals.ts
index d1dec1f..7e70f61 100644
--- a/example/src/internals.ts
+++ b/example/src/internals.ts
@@ -12,7 +12,7 @@ export function anInternalFunction(): void {
 /**
  * @ignore
  *
- *   `@hidden` and `@ignore` keep the subsequent code from being documented.
+ * `@hidden` And `@ignore` keep the subsequent code from being documented.
  */
 export function willNotBeDocumented(target: any, value: number): number {
     return 0;
diff --git a/src/index.ts b/src/index.ts
index 6e25ce8..3b3f6d7 100644
--- a/src/index.ts
+++ b/src/index.ts
@@ -1,11 +1,9 @@
 /**
  * @module TypeDoc API
- *
  * In addition to the members documented here, TypeDoc exports a `typedoc/debug`
  * entry point which exports some functions which may be useful during plugin
  * development or debugging. Exports from that entry point are **not stable**
  * and may change or be removed at any time.
- *
  * TypeDoc also exports a `typedoc/browser` entry point which exports a subset
  * of the members described here which makes it suitable for usage in browser
  * bundles which want to use TypeDoc's JSON output in the browser.
@@ -18,24 +16,20 @@ export { resetReflectionID } from "./lib/models/Reflection.js";
  * root import.
  *
  * @primaryExport
- *
  * @categoryDescription Types
  * Describes a TypeScript type.
- *
- * @categoryDescription Reflections
- * Describes a documentation entry. The root entry is a {@link ProjectReflection}
- * and contains {@link DeclarationReflection} instances.
- *
- * @summary
- * TypeDoc converts source code into these object types.
+ * @categoryDescription Reflections Describes a documentation entry. The root
+ *   entry is a {@link ProjectReflection} and contains
+ *   {@link DeclarationReflection} instances.
+ * @summary TypeDoc converts source code into these object types.
  */
 export * as Models from "./lib/models/index.js";
 /**
  * All symbols documented under the Configuration namespace are also available
  * in the root import.
  *
- * @summary
- * Controls how TypeDoc reads option files and what options are available.
+ * @summary Controls how TypeDoc reads option files and what options are
+ *   available.
  */
 export {
     type CommentParserConfig,
diff --git a/src/lib/application.ts b/src/lib/application.ts
index 8485ba8..a0220f9 100644
--- a/src/lib/application.ts
+++ b/src/lib/application.ts
@@ -101,21 +101,22 @@ export interface ApplicationEvents {
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
  * application flow or alter the output.
  *
  * @remarks
- *   Access to an Application instance can be retrieved with
- *   {@link Application.bootstrap} or {@link Application.bootstrapWithPlugins}. It
- *   can not be constructed manually.
+ *     Access to an Application instance can be retrieved with
+ *     {@link Application.bootstrap} or {@link Application.bootstrapWithPlugins}. It
+ *     can not be constructed manually.
  * @group None
  * @summary Root level class which contains most useful behavior.
  */
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
diff --git a/src/lib/converter/context.ts b/src/lib/converter/context.ts
index d8ec81d..2b9d6cb 100644
--- a/src/lib/converter/context.ts
+++ b/src/lib/converter/context.ts
@@ -257,9 +257,8 @@ export class Context {
     /**
      * Create a {@link ReferenceType} which points to the provided symbol.
      *
-     * @privateRemarks
-     *   This is available on Context so that it can be monkey-patched by
-     *   typedoc-plugin-missing-exports
+     * @privateRemarks This is available on Context so that it can be
+     *   monkey-patched by typedoc-plugin-missing-exports
      */
     createSymbolReference(
         symbol: ts.Symbol,
@@ -287,10 +286,9 @@ export class Context {
      * Create a stable {@link ReflectionSymbolId} for the provided symbol,
      * optionally targeting a specific declaration.
      *
-     * @privateRemarks
-     *   This is available on Context so that it can be monkey-patched by
-     *   typedoc-plugin-missing-exports It might also turn out to be generally
-     *   useful for other plugin users.
+     * @privateRemarks This is available on Context so that it can be
+     *   monkey-patched by typedoc-plugin-missing-exports It might also turn out
+     *   to be generally useful for other plugin users.
      */
     createSymbolId(symbol: ts.Symbol, declaration?: ts.Declaration) {
         return createSymbolIdImpl(symbol, declaration);
diff --git a/src/lib/converter/converter.ts b/src/lib/converter/converter.ts
index aa8824b..3d8cdce 100644
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
@@ -212,8 +213,11 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {
 
     /**
      * Triggered when the converter has created a signature reflection. The
-     * listener will be given {@link Context}, {@link Models.SignatureReflection}
+     * listener will be given {@link Context},
+     * {@link Models.SignatureReflection}
+     *
      * | {@link Models.ProjectReflection} the declaration,
+     *
      * `ts.SignatureDeclaration | ts.IndexSignatureDeclaration |
      * ts.JSDocSignature | undefined`, and `ts.Signature | undefined`. The
      * signature will be undefined if the created signature is an index
@@ -225,8 +229,8 @@ export class Converter extends AbstractComponent<Application, ConverterEvents> {
 
     /**
      * Triggered when the converter has created a parameter reflection. The
-     * listener will be given {@link Context}, {@link Models.ParameterReflection}
-     * and a `ts.Node?`
+     * listener will be given {@link Context},
+     * {@link Models.ParameterReflection} and a `ts.Node?`
      *
      * @event
      */
diff --git a/src/lib/converter/plugins/GroupPlugin.ts b/src/lib/converter/plugins/GroupPlugin.ts
index 86316a4..9187d05 100644
--- a/src/lib/converter/plugins/GroupPlugin.ts
+++ b/src/lib/converter/plugins/GroupPlugin.ts
@@ -152,9 +152,8 @@ export class GroupPlugin extends ConverterComponent {
     /**
      * Extracts the groups for a given reflection.
      *
-     * @privateRemarks
-     *   If you change this, also update extractCategories in CategoryPlugin
-     *   accordingly.
+     * @privateRemarks If you change this, also update extractCategories in
+     *   CategoryPlugin accordingly.
      */
     getGroups(reflection: DeclarationReflection | DocumentReflection) {
         return GroupPlugin.getGroups(reflection, this.groupReferencesByType);
diff --git a/src/lib/models/ReflectionSymbolId.ts b/src/lib/models/ReflectionSymbolId.ts
index 83e26fd..333c686 100644
--- a/src/lib/models/ReflectionSymbolId.ts
+++ b/src/lib/models/ReflectionSymbolId.ts
@@ -56,9 +56,8 @@ export class ReflectionSymbolId {
      * is set so that it is available to plugins when initially converting a
      * project.
      *
-     * @privateRemarks
-     *   This is used by typedoc-plugin-dt-links to determine the path to read to
-     *   get the source code of a definitely typed package.
+     * @privateRemarks This is used by typedoc-plugin-dt-links to determine the
+     *   path to read to get the source code of a definitely typed package.
      */
     fileName?: NormalizedPath;
 
diff --git a/src/lib/models/types.ts b/src/lib/models/types.ts
index 8f3a572..8dca611 100644
--- a/src/lib/models/types.ts
+++ b/src/lib/models/types.ts
@@ -765,11 +765,10 @@ export class QueryType extends Type {
     }
 
     /**
-     * @privateRemarks
-     *   An argument could be made that this ought to return true for
-     *   indexedObject since precedence is different than on the value side...
-     *   if someone really cares they can easily use a custom theme to change
-     *   this.
+     * @privateRemarks An argument could be made that this ought to return true
+     *   for indexedObject since precedence is different than on the value
+     *   side... if someone really cares they can easily use a custom theme to
+     *   change this.
      */
     override needsParenthesis(): boolean {
         return false;
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
 
diff --git a/src/lib/output/themes/default/assets/typedoc/utils/modal.ts b/src/lib/output/themes/default/assets/typedoc/utils/modal.ts
index 037b879..5df78d4 100644
--- a/src/lib/output/themes/default/assets/typedoc/utils/modal.ts
+++ b/src/lib/output/themes/default/assets/typedoc/utils/modal.ts
@@ -2,21 +2,17 @@
  * @module
  *
  * Browsers allow scrolling of page with native dialog, which is a UX issue.
+ * `@starting-style` and `overlay` aren't well supported in FF, and only
+ * available in latest versions of chromium, hence, a custom overlay workaround
+ * is required. Workaround:
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
+ * - Append a custom overlay element (a div) to `document.body`, this does **NOT**
+ *   handle nested modals, as the overlay div cannot be in the top layer, which
+ *   wouldn't overshadow the parent modal.
  * - Add exit animation on dialog and overlay, without actually closing them
  * - Listen for `animationend` event, and close the modal immediately
  *
- * @see
- * - The "[right](https://frontendmasters.com/blog/animating-dialog/)" way to animate modals
+ * @see - The "[right](https://frontendmasters.com/blog/animating-dialog/)" way to animate modals
  * - [Workaround](https://github.com/whatwg/html/issues/7732#issuecomment-2437820350) to prevent background scrolling
  */
 
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
diff --git a/src/lib/utils-common/events.ts b/src/lib/utils-common/events.ts
index 3a79f93..127b569 100644
--- a/src/lib/utils-common/events.ts
+++ b/src/lib/utils-common/events.ts
@@ -3,8 +3,8 @@ import { insertPrioritySorted } from "./array.js";
 /**
  * Intentionally very simple event emitter.
  *
- * @privateRemarks
- *   This is essentially a stripped down copy of EventHooks in hooks.ts.
+ * @privateRemarks This is essentially a stripped down copy of EventHooks in
+ *   hooks.ts.
  */
 export class EventDispatcher<T extends Record<keyof T, unknown[]>> {
     // Function is *usually* not a good type to use, but here it lets us specify stricter
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
diff --git a/src/test/converter/comment/comment.ts b/src/test/converter/comment/comment.ts
index e8c342f..40dabca 100644
--- a/src/test/converter/comment/comment.ts
+++ b/src/test/converter/comment/comment.ts
@@ -16,23 +16,14 @@ import "./comment2";
  *
  * An example with decorators that should not parse to tag
  *
- *     @myDecorator
- *     @FactoryDecorator("a", "b", "c")
- *     export class CommentedClass {
- *         myProp: string = "myProp";
- *
- *         @PropDecorator() decoratedProp: string;
- *
- *         constructor(@ParamDecorator public param: string) {}
- *
- *         myMethod() {}
- *     }
- *
  * @deprecated
  * @type {Data<object>} will Also be removed
+ * @myDecorator
+ * @FactoryDecorator("a", "b", "c") export class CommentedClass { myProp: string
+ *   = "myProp"; @PropDecorator() decoratedProp: string;
+ *   constructor(@ParamDecorator public param: string) {} myMethod() {} }
  * @groupDescription Methods
  * Methods description!
- *
  * @gh3020 {type annotation}
  */
 export class CommentedClass {
diff --git a/src/test/converter/function/function.ts b/src/test/converter/function/function.ts
index 5f6b105..db052d7 100644
--- a/src/test/converter/function/function.ts
+++ b/src/test/converter/function/function.ts
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
diff --git a/src/test/converter/inheritance/inherit-doc.ts b/src/test/converter/inheritance/inherit-doc.ts
index 5276309..03427ec 100644
--- a/src/test/converter/inheritance/inherit-doc.ts
+++ b/src/test/converter/inheritance/inherit-doc.ts
@@ -37,10 +37,11 @@ export interface InterfaceTarget<T> {
  * This part of the commentary will be inherited by other entities
  *
  * @remarks
- *   Remarks will be inherited
+ *     Remarks will be inherited
  * @default
  *
- * This part of the commentary will not be inherited (this is an abuse of this tag)
+ * This part of the commentary will not be inherited (this is an abuse of this
+ * tag)
  *
  * @typeParam T - Type of arguments
  * @param arg1 - First argument
@@ -57,7 +58,6 @@ export function functionSource<T>(arg1: T, arg2: T): string {
 /**
  * @example
  *     This function inherited commentary from the `functionSource` function
- *
  * @typeParam T - This will be inherited
  * @param arg1 - This will be inherited
  * @param arg2 - This will be inherited
diff --git a/src/test/converter/js/index.js b/src/test/converter/js/index.js
index ec5bd88..3edd805 100644
--- a/src/test/converter/js/index.js
+++ b/src/test/converter/js/index.js
@@ -15,13 +15,11 @@
 
 /**
  * @typedef {string | number} UnionType Docs for alias
- *
  * @typedef {{ x: string } & { y: number }} IntersectionType Docs for alias
  */
 
 /**
  * @callback NoReturnTag Even though in the same comment block
- *
  * @callback HasReturnTag
  * @returns {string}
  */
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
diff --git a/src/test/converter2/behavior/inheritDocBasic.ts b/src/test/converter2/behavior/inheritDocBasic.ts
index 5643e40..4fcbe85 100644
--- a/src/test/converter2/behavior/inheritDocBasic.ts
+++ b/src/test/converter2/behavior/inheritDocBasic.ts
@@ -2,7 +2,7 @@
  * Summary
  *
  * @remarks
- *   Remarks
+ *     Remarks;
  * @typeParam T - Type parameter
  */
 export interface InterfaceSource<T> {
diff --git a/src/test/converter2/behavior/inheritDocWarnings.ts b/src/test/converter2/behavior/inheritDocWarnings.ts
index 99634d7..f298723 100644
--- a/src/test/converter2/behavior/inheritDocWarnings.ts
+++ b/src/test/converter2/behavior/inheritDocWarnings.ts
@@ -2,7 +2,7 @@
  * Source
  *
  * @remarks
- *   Remarks
+ *     Remarks;
  */
 export const source = 123;
 
@@ -17,7 +17,7 @@ export const target1 = 123;
 
 /**
  * @remarks
- *   Target remarks
+ *     Target remarks
  * @inheritDoc source
  */
 export const target2 = 123;
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
diff --git a/src/test/converter2/issues/gh1896.js b/src/test/converter2/issues/gh1896.js
index 7c3d79d..9a366b0 100644
--- a/src/test/converter2/issues/gh1896.js
+++ b/src/test/converter2/issues/gh1896.js
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
diff --git a/src/test/converter2/issues/gh2384.js b/src/test/converter2/issues/gh2384.js
index 37589fd..535123c 100644
--- a/src/test/converter2/issues/gh2384.js
+++ b/src/test/converter2/issues/gh2384.js
@@ -1,11 +1,14 @@
 /**
  * @typedef {Int8Array | Uint8Array} TypedArray A union type representing all
- *   possible TypedArrays.
+ * possible TypedArrays.
  */
 
 /**
  * @template {TypedArray} [T=Typed<TypedArray>] Desc. Default is
  *   `Typed<TypedArray>`
+ *
+ *   Default is `Typed<TypedArray>`
+ *
  * @typedef {T} Typed A generic type representing a TypedArray.
  */
 
diff --git a/src/test/converter2/issues/gh3012.ts b/src/test/converter2/issues/gh3012.ts
index da94037..049e3b5 100644
--- a/src/test/converter2/issues/gh3012.ts
+++ b/src/test/converter2/issues/gh3012.ts
@@ -1,6 +1,6 @@
 /**
  * @remarks
- *   DictRemarks
+ *     DictRemarks;
  */
 export const DictionarySchema = {};
 
@@ -8,6 +8,6 @@ export const DictionarySchema = {};
  * {@inheritDoc DictionarySchema}
  *
  * @remarks
- *   Alias of {@link DictionarySchema}
+ *     Alias of {@link DictionarySchema}
  */
 export const NullProtoObjectSchema = DictionarySchema;
diff --git a/src/test/converter2/issues/gh3020.ts b/src/test/converter2/issues/gh3020.ts
index 024b91f..b0efc79 100644
--- a/src/test/converter2/issues/gh3020.ts
+++ b/src/test/converter2/issues/gh3020.ts
@@ -1,7 +1,7 @@
 /**
  * Component demo.
  *
- * @fires {CustomEvent<{ id: string; source: Element }>} item-click - Event when
+ * @fires {CustomEvent<{ Id: string; source: Element }>} item-click - Event when
  *   item is clicked.
  */
 export class ButtonControlElement extends Object {}
diff --git a/src/test/converter2/issues/gh671.js b/src/test/converter2/issues/gh671.js
index 69b42d0..6a03682 100644
--- a/src/test/converter2/issues/gh671.js
+++ b/src/test/converter2/issues/gh671.js
@@ -2,6 +2,8 @@
  * @param {string} x The string to parse as a number
  * @param {boolean} [int=true] Whether to parse as an integer or float. Default
  *   is `true`
+ *
+ *   Default is `true`
  */
 export function toNumber(x, int = true) {
     return int ? parseInt(x) : parseFloat(x);
diff --git a/src/test/converter2/renderer/index.ts b/src/test/converter2/renderer/index.ts
index 69845b4..a3a76fa 100644
--- a/src/test/converter2/renderer/index.ts
+++ b/src/test/converter2/renderer/index.ts
@@ -77,7 +77,7 @@ export class ModifiersClass {
  * Enum comment {@link Value1}
  *
  * @remarks
- *   Block tag
+ *     Block tag
  */
 export enum Enumeration {
     /** Value1 comment */
```
