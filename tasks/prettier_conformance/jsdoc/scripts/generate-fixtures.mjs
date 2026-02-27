/**
 * Generate jsdoc conformance test fixtures.
 *
 * Extracts test cases from prettier-plugin-jsdoc's test suite,
 * runs them through prettier + the plugin, and writes input/output pairs
 * as fixture files for the Rust conformance runner.
 *
 * Usage:
 *   cd tasks/prettier_conformance/jsdoc/scripts
 *   npm install
 *   node generate-fixtures.mjs
 *
 * Environment variables:
 *   PLUGIN_DIR - path to the cloned prettier-plugin-jsdoc repo
 *                (default: /tmp/prettier-plugin-jsdoc)
 */

import * as prettier from "prettier";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURES_DIR = resolve(__dirname, "..", "fixtures");
const PLUGIN_DIR = process.env.PLUGIN_DIR || "/tmp/prettier-plugin-jsdoc";

// Default options matching the plugin's test setup
const DEFAULT_OPTIONS = {
  parser: "babel",
  plugins: ["prettier-plugin-jsdoc"],
  jsdocSpaces: 1,
};

/**
 * Format code with prettier + jsdoc plugin.
 */
async function format(code, options = {}) {
  return prettier.format(code, {
    ...DEFAULT_OPTIONS,
    ...options,
  });
}

/**
 * Map prettier parser to file extension for SourceType detection in Rust.
 */
function parserToExt(parser) {
  switch (parser) {
    case "typescript":
      return "ts";
    case "babel-ts":
      return "ts";
    case "babel-flow":
      return "js";
    default:
      return "js";
  }
}

/**
 * Write a fixture pair (input + expected output).
 */
function writeFixture(category, name, input, output, options = {}) {
  const dir = resolve(FIXTURES_DIR, category);
  mkdirSync(dir, { recursive: true });

  const ext = parserToExt(options.parser || "babel");
  const inputPath = resolve(dir, `${name}.${ext}`);
  const outputPath = resolve(dir, `${name}.output.${ext}`);

  writeFileSync(inputPath, input);
  writeFileSync(outputPath, output);
  console.log(`  wrote ${category}/${name}.${ext}`);
}

// ============================================================================
// Test case definitions
// ============================================================================

/**
 * Each test case: { name, input, options? }
 * We only include tests using default-like options (jsdocSpaces: 1, no
 * vertical alignment, no custom spaces, etc.)
 */

// ---------------------------------------------------------------------------
// main.test.ts
// ---------------------------------------------------------------------------
const mainTests = [
  {
    name: "001-js-code-formatted",
    input: `
const variable1 = 1             // No semicolon
const stringVar = "text"        // Wrong quotes
  const indented = 2            // Wrong indentation

// Longer then 80 characters
const someLongList = ['private', 'memberof', 'description', 'example', 'param', 'returns', 'link']`,
  },
  {
    name: "002-regular-jsdoc",
    input: `
/**
* function example description that was wrapped by hand
* so it have more then one line and don't end with a dot
* REPEATED TWO TIMES BECAUSE IT WAS EASIER to copy
* function example description that was wrapped by hand
* so it have more then one line.
* @return {Boolean} Description for @returns with s
* @param {String|Number} text - some text description that is very long and needs to be wrapped
* @param {String} [defaultValue="defaultTest"] TODO
* @arg {Number|Null} [optionalNumber]
* @private
*@memberof test
@async
* @examples
*   var one = 5
*   var two = 10
*
*   if(one > 2) { two += one }
* @undefiendTag \${" "}
* @undefiendTag {number} name des
*/
const testFunction = (text, defaultValue, optionalNumber) => true
`,
  },
  {
    name: "003-jsdoc-default-values",
    input: `
/**
* @param {String} [arg1="defaultTest"] foo
* @param {number} [arg2=123] the width of the rectangle
* @param {number} [arg3= 123 ]
* @param {number} [arg4= Foo.bar.baz ]
* @param {number|string} [arg5=123] Something. Default is \`"wrong"\`
*/
`,
  },
  {
    name: "004-single-line-conversion",
    // Three separate assertions in original test; we combine the inputs
    input: `/** single line description*/

/**
 * single line description
 * @example
 */

/**
 * single line description
 * @return {Boolean} Always true
 * @example
 */
`,
  },
  {
    name: "005-undefined-null-void-type",
    input: `/**
 * @return {undefined}
 */

/**
 * @return {null}
 */

/**
 * @returns { void } \${" "}
 */
`,
  },
  {
    name: "006-inner-types",
    input: `/**
 * @param {Array.<String>} test test param
 */

/**
 * @param {String[]} test Test param
 */

/**
 * @param {(String|Object)[]} test Test param
 */

/**
 * @returns {Promise<Number|String|undefined>} test promise
 */

/**
 * @returns {Object<Number|String|undefined>} test object
 */
`,
  },
  {
    name: "007-params-ordering-10-tags",
    input: `/**
 * description
 * @param {Number} test1 Test param
 * @param {Number} test2 Test param
 * @param {Number|String} test3 Test param
 * @param {?undefined} test4 Test param
 * @param {!undefined} test5 Test param
 * @param {*} test6 Test param
 * @param {"*"} test6 Test param
 * @param {?Number} test7 Test param
 * @param {...Number} test8 Test param
 * @param {!Number} test9 Test param
 * @param {String} test10 Test param
 * @param {Array} test11 Test param
 * @returns {Promise<Object<string, number|undefined>>} test return
 */
`,
  },
  {
    name: "008-complex-inner-types",
    input: `/**
 * @param {Array<(String|Number)>} test test param
 * @param {Array<Object.<String, Number>>} test test param
 * @param {...Number} test Test param
 * @todo  todo is no param
 * @param {?Number} test Test param
 * @param {?undefined} test Test param
 * @param {!Number} test Test param
 * @param {Number} test Test param
 * @param {Number|String} test Test param
 * @param {undefined} test Test param
 * @param {*} test Test param
 */

/**
 * @returns {Promise<Object<string, number|undefined>>} test return
 */
`,
  },
  {
    name: "009-big-single-word",
    input: `/**
    * Simple Single Word
    * https://github.com/babel/babel/pull/7934/files#diff-a739835084910b0ee3ea649df5a4d223R67
   */`,
  },
  {
    name: "010-hyphen-description",
    input: `
/**
 * Assign the project to an employee.
 * @param {Object} employee - The employee who is responsible for the project.
 * @param {string} employee.name - The name of the employee.
 * @param {string} employee.department - The employee's department.
 */
`,
  },
  {
    name: "011-bad-defined-name",
    input: `
  /** @type{import('@jest/types/build/Config').InitialOptions} */
  /** @type{{foo:string}} */

  /** @typedef{import('@jest/types/build/Config').InitialOptions} name a description  */
`,
  },
  {
    name: "012-long-description",
    input: `
  /** Configures custom logging for the {@link @microsoft/signalr.HubConnection}.
   *
   * https://example.com
   * @param {LogLevel | string | ILogger} logging A {@link @microsoft/signalr.LogLevel}, a string representing a LogLevel, or an object implementing the {@link @microsoft/signalr.ILogger} interface.
   *    See {@link https://docs.microsoft.com/aspnet/core/signalr/configuration#configure-logging|the documentation for client logging configuration} for more details.
   * @returns The {@link @microsoft/signalr.HubConnectionBuilder} instance, for chaining.
   */
`,
  },
  {
    name: "013-since-tag",
    input: `
  /**
   * @since 3.16.0
   */
`,
  },
  {
    name: "014-incorrect-comment",
    input: `
  /***
   * Some comment
   */
  export class Dummy {}

  /**
   *
   */
  export class Dummy {}
`,
  },
  {
    name: "015-empty-comment",
    input: `
  // Line Comment
  //
`,
  },
  {
    name: "016-empty-jsdoc-default",
    input: `
/**
 */
function test() {}

/** */
const value = 1;

/**
 *
 */
class MyClass {}
`,
  },
  {
    name: "017-optional-parameters",
    input: `
  /**
   * @param {number=} arg1
   * @param {number} [arg2]
   * @param {number} [arg3=4]
   */
`,
  },
  {
    name: "018-non-jsdoc-comment",
    input: `
  // @type   { something  }
  /* @type   { something  }  */
  /* /** @type   { something  }  */
`,
  },
  {
    name: "019-rest-parameters",
    input: `
  /**
   * @param {... *} arg1
   * @param {... number} arg2
   * @param {... (string|number)} arg3
   * @param {... string|number} arg4 This is equivalent to arg3
   *
   */
  function a(){}
`,
  },
  {
    name: "020-param-order",
    input: `
  /**
* @param {  string   }    param0 description
* @param {  number   }    param2 description
* @param {  object   }    param1 description
   */
function fun(param0, param1, param2){}

export const SubDomain = {
/**
 * @param {} subDomainAddress2
 * @param {any} subDomainAddress
* @returns {import('axios').AxiosResponse<import('../types').SubDomain>}
*/
async subDomain(subDomainAddress2,subDomainAddress) {
},
};


/**
 * @param {  string   }    param0 description
 * @param {  number   }    param2 description
 * @param {  object   }    param1 description
    */
 const fun=(param0, param1, param2)=>{
   console.log('')
 }
`,
  },
  {
    name: "021-jsdoc-tags",
    input: `/**
    * @namespace
    * @borrows trstr as trim
    */

  /**
    * Whether the type should be non null, \`required: true\` = \`nullable: false\`
    *
    * @default (depends on whether nullability is configured in type or schema)
    * @see declarativeWrappingPlugin
    */

/**
 * @default 'i am a value' i am the description
*/
`,
  },
  {
    name: "022-example-tag",
    input: `
/**
 * ABCCCC
 *
 * @example <caption>DDDD</caption>
 *
 *   const MyHook = () => {
 *     const config useConfig(state => state.config)
 *     return <span></span>
 *   }
 *
 *   const useMyHook = () => {
 *     const config useConfig(state => state.config)
 *     return config
 *   }
 *
 * @example <caption>AAAA</caption>
 *   const config = useConfig.getState().config
 */
`,
  },
  {
    name: "023-optional-params",
    input: `
/**
 * @param {string=} p2 - An optional param (Google Closure syntax)
 * @param {string} [p3] - Another optional param (JSDoc syntax).
 * @returns {string=}
*/
`,
  },
  {
    name: "024-non-escapable-character",
    input: `
  /**
   * \\\\\\\\
   *
   *
   * \\\\\\\\-
   */

  /**
   * \\\\\\\\
   */
`,
  },
  {
    name: "025-file-tag",
    input: `
  /** @file A file description */
`,
  },
  {
    name: "026-block-quote",
    input: `
  /** > A block quote */

  /**
   *  > \`\`\`js
   *  > > A block quote
   *  > \`\`\`
   *  >
   *  > turns into
   *  >
   *  > \`\`\`js
   *  > A block quote
   *  > \`\`\`
   *
   *  sdssasdassd
   *
   */
`,
  },
  {
    name: "027-satisfies",
    input: `
  /**
   * Bounce give a renderContent and show that around children when isVisible is
   * true
   *
   * @satisfies {React.FC<BounceProps>}
   * @example
   *   <Bounce
   *     isVisible={isVisible}
   *     dismiss={() => setVisible(false)}
   *     renderContent={() => {
   *       return <InsideOfPopeUp />;
   *     }}>
   *     <Button />
   *   </Bounce>;
   *
   * @type {React.FC<BounceProps>}
   */
`,
  },
  {
    name: "028-bracket-spacing-true",
    input: `
/**
 * Function with type annotations
 * @param {string} name - The name parameter
 * @param {number} age - The age parameter
 * @returns {object} The result object
 */
function example(name, age) {}
`,
    options: { jsdocBracketSpacing: true },
  },
  {
    name: "029-bracket-spacing-false",
    input: `
/**
 * Function with type annotations
 * @param {string} name - The name parameter
 * @param {number} age - The age parameter
 * @returns {object} The result object
 */
function example(name, age) {}
`,
    options: { jsdocBracketSpacing: false },
  },
];

// ---------------------------------------------------------------------------
// descriptions.test.ts  (uses parser: "babel-ts")
// ---------------------------------------------------------------------------
const descriptionsTests = [
  {
    name: "001-paragraph",
    input: `
/**
 * Does the following things:
 *
 *    1. Thing 1
 *
 *    2. Thing 2
 *
 *    3. Thing 3
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "002-paragraph-compact",
    input: `
  /**
   * Does the following things:
   *
   *    1. Thing 1
   *    2. Thing 2
   *    3. Thing 3
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "003-paragraph-class",
    input: `
  class test {
    /**
     * Lorem ipsum dolor sit amet, consectetur adipiscing elit,
     *  sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
     *
     *  Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
     *
     * lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
     *    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
     */
    a(){}
  }
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "004-example-and-types",
    input: `
  /**
   * Transforms data
   *
   * @override
   */


  /**
   * Bounce give a renderContent and show that around children when isVisible is
   * true
   *
   * @example
   *   <Bounce
   *     isVisible={isVisible}
   *     dismiss={() => setVisible(false)}
   *     renderContent={() => {
   *       return <InsideOfPopeUp />;
   *     }}>
   *     <Button />
   *   </Bounce>;
   *
   * @type {React.FC<BounceProps>}
   */

   /**
    * @param {string} a
    *
    * \`\`\`js
    * var a = 0;
    * \`\`\`
    */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "005-dash-list",
    input: `
  /**
   * We will allow the scroll view to give up its lock iff it acquired the lock
   * during an - animation. This is a very useful default that happens to satisfy
   * many common user experiences.
   *
   * - Stop a scroll on the left edge, then turn that into an outer view's
   *   backswipe.
   * - Stop a scroll mid-bounce at the top, continue pulling to have the outer
   *   view dismiss.
   * - However, without catching the scroll view mid-bounce (while it is
   *   motionless), if you drag far enough for the scroll view to become
   *   responder (and therefore drag the scroll view a bit), any backswipe
   *   navigation of a swipe gesture higher in the view hierarchy, should be
   *   rejected.
   */
  function scrollResponderHandleTerminationRequest() {
    return !this.state.observedScrollSinceBecomingResponder;
  }



  /**
   * - stop a scroll on the left edge, then turn that into an outer view's
   *   backswipe.
   * - Stop a scroll mid-bounce at the top, continue pulling to have the outer
   *   view dismiss.
   */
  function scrollResponderHandleTerminationRequest() {
    return !this.state.observedScrollSinceBecomingResponder;
  }

  /**- stop a scroll on the left edge, then turn that into an outer view's
   *   backswipe.
   * - Stop a scroll mid-bounce at the top, continue pulling to have the outer
   *   view dismiss.
   */
  function scrollResponderHandleTerminationRequest() {
    return !this.state.observedScrollSinceBecomingResponder;
  }
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "006-hit-rect",
    input: `
  /**
   * Measures the \`HitRect\` node on activation. The Bounding rectangle is with
   * respect to viewport - not page, so adding the \`pageXOffset/pageYOffset\`
   * should result in points that are in the same coordinate system as an
   * event's \`globalX/globalY\` data values.
   *
   * - Consider caching this for the lifetime of the component, or possibly being able to share this
   *   cache between any \`ScrollMap\` view.
   *
   * @private
   *
   * @sideeffects
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "007-test-case-parsing",
    input: `
  /**
   * Handles parsing of a test case file.
   *
   *
   * A test case file consists of at least two parts, separated by a line of dashes.
   * This separation line must start at the beginning of the line and consist of at least three dashes.
   *
   * The test case file can either consist of two parts:
   *
   *     const a=''
   *     const b={c:[]}
   *
   *
   * or of three parts:
   *
   *     {source code}
   *     ----
   *     {expected token stream}
   *     ----
   *     {text comment explaining the test case}
   *
   * If the file contains more than three parts, the remaining parts are just ignored.
   * If the file however does not contain at least two parts (so no expected token stream),
   * the test case will later be marked as failed.
   *
   *
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "008-press-responder",
    input: `
/**
 * ========================== PressResponder Tutorial ==========================
 *
 * The \`PressResponder\` class helps you create press interactions by analyzing the
 * geometry of elements and observing when another responder (e.g. ScrollView)
 * has stolen the touch lock. It offers hooks for your component to provide
 * interaction feedback to the user:
 *
 * - When a press has activated (e.g. highlight an element)
 * - When a press has deactivated (e.g. un-highlight an element)
 * - When a press sould trigger an action, meaning it activated and deactivated while within the geometry of the element without the lock being stolen.
 *
 * A high quality interaction isn't as simple as you might think. There should
 * be a slight delay before activation. Moving your finger beyond an element's
 * bounds should trigger deactivation, but moving the same finger back within an
 * element's bounds should trigger reactivation.
 *
 * 1- In order to use \`PressResponder\`, do the following:
 *\`\`\`js
 *     const pressResponder = new PressResponder(config);
 *\`\`\`
 *   2.   Choose the rendered component who should collect the press events. On that
 *   element, spread \`pressability.getEventHandlers()\` into its props.
 *\`\`\`js
 *    return (
 *      <View {...this.state.pressResponder.getEventHandlers()} />
 *    );
 *\`\`\`
 * 3. Reset \`PressResponder\` when your component unmounts.
 *\`\`\`js
 *    componentWillUnmount() {
 *      this.state.pressResponder.reset();
 *    }
 *\`\`\`
 * ==================== Implementation Details ====================
 *
 * \`PressResponder\` only assumes that there exists a \`HitRect\` node. The \`PressRect\`
 * is an abstract box that is extended beyond the \`HitRect\`.
 *
 * # Geometry
 *  When the press is released outside the \`HitRect\`,
 *  the responder is NOT eligible for a "press".
 *
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "009-focus-heuristics",
    input: `
  /**
   * 1-    a keydown event occurred immediately before a focus event
   * 2- a focus event happened on an element which requires keyboard interaction (e.g., a text field);
   * 2- a focus event happened on an element which requires keyboard interaction (e.g., a text field);
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "010-numbered-list-long",
    input: `
/**
 * Etiam sit amet orci eget eros faucibus tincidunt. Duis leo. Sed fringilla mauris sit amet nibh. Donec sodales sagittis magna. Sed consequat, leo eget bibendum sodales, augue velit cursus nunc, quis gravida magna mi a libero. Fusce vulputate eleifend sapien. Vestibulum purus quam, scelerisque ut, mollis sed, nonummy id, metus.
 *
 * 1. Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim.
 * 2. Donec pede justo, fringilla vel, aliquet nec, vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, justo. Nullam dictum felis eu pede mollis pretium. Integer tincidunt. Cras dapibus. Vivamus elementum semper nisi. Aenean vulputate eleifend tellus.
 *
 *    Aenean leo ligula, porttitor eu, consequat vitae, eleifend ac, enim. Aliquam lorem ante, dapibus in, viverra quis, feugiat a, tellus. Phasellus viverra nulla ut metus varius laoreet. Quisque rutrum. Aenean imperdiet. Etiam ultricies nisi vel augue. Curabitur ullamcorper ultricies nisi. Nam eget dui.
 *
 * @public
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "011-nested-list",
    input: `
/**
 * 1.  Foo
 *     1.  Entry 1
 *     2.  Entry 2
 *         - Foo
 *         - bar
 *     3.  Entry 3
 * 2.  Bar
 *     1.  Entry 1
 *     2.  Entry 2
 *     3.  Entry 3
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "012-newline-backslash",
    input: `
/**
 * A short description,\\
 * A long description.
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "013-list-in-tags",
    input: `
/**
 * @param {any} var An example list:
 *
 *   - Item 1
 *   - Item 2
 *
 * @returns {Promise} A return value.
 */

 /**
  * @param {any} var An example list:
  *
  *   - Item 1
  *   - Item 2
  *
  */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "014-touchable-tutorial",
    input: `
/**
 * \`Touchable\`: Taps done right.
 *
 * You hook your \`ResponderEventPlugin\` events into \`Touchable\`. \`Touchable\`
 * will measure time/geometry and tells you when to give feedback to the user.
 *
 * ====================== Touchable Tutorial ===============================
 * The \`Touchable\` mixin helps you handle the "press" interaction. It analyzes
 *  the geometry of elements, and observes when another responder (scroll view
 * etc) has stolen the touch lock. It notifies your component when it should
 * give feedback to the user. (bouncing/highlighting/unhighlighting).
 *
 * - When a touch was activated (typically you highlight)
 * - When a touch was deactivated (typically you unhighlight)
 * - When a touch was "pressed" - a touch ended while still within the geometry
 *   of the element, and no other element (like scroller) has "stolen" touch
 *   lock ("responder") (Typically you bounce the element).
 *
 * A good tap interaction isn't as simple as you might think. There should be a
 * slight delay before showing a highlight when starting a touch. If a
 * subsequent touch move exceeds the boundary of the element, it should
 * unhighlight, but if that same touch is brought back within the boundary, it
 * should rehighlight again. A touch can move in and out of that boundary
 * several times, each time toggling highlighting, but a "press" is only
 * triggered if that touch ends while within the element's boundary and no
 * scroller (or anything else) has stolen the lock on touches.
 *
 * To create a new type of component that handles interaction using the
 * \`Touchable\` mixin, do the following:
 *
 * - Initialize the \`Touchable\` state.
 *\`\`\`js
 *   getInitialState: function(   ) {
 *     return merge(this.touchableGetInitialState(), yourComponentState);
 *   }
 *\`\`\`
 * - Choose the rendered component who's touches should start the interactive
 *   sequence. On that rendered node, forward all \`Touchable\` responder
 *   handlers. You can choose any rendered node you like. Choose a node whose
 *   hit target you'd like to instigate the interaction sequence:
 *\`\`\`js
 *   // In render function:
 *   return (
 *     <View
 *
 *       onStartShouldSetResponder={this.touchableHandleStartShouldSetResponder}
 *       onResponderTerminationRequest={this.touchableHandleResponderTerminationRequest}
 *       onResponderGrant={this.touchableHandleResponderGrant}
 *       onResponderMove={this.touchableHandleResponderMove}
 *       onResponderRelease={this.touchableHandleResponderRelease}
 *       onResponderTerminate={this.touchableHandleResponderTerminate}>
 *       <View>
 *         Even though the hit detection/interactions are triggered by the
 *         wrapping (typically larger) node, we usually end up implementing
 *         custom logic that highlights this inner one.
 *       </View>
 *     </View>
 *   );
 *\`\`\`
 * - You may set up your own handlers for each of these events, so long as you
 *   also invoke the \`touchable*\` handlers inside of your custom handler.
 *
 * - Implement the handlers on your component class in order to provide
 *   feedback to the user. See documentation for each of these class methods
 *   that you should implement.
 *\`\`\`js
 *   touchableHandlePress: function() {
 *      this.performBounceAnimation();  // or whatever you want to do.
 *   },
 *   touchableHandleActivePressIn: function() {
 *     this.beginHighlighting(...);  // Whatever you like to convey activation
 *   },
 *   touchableHandleActivePressOut: function() {
 *     this.endHighlighting(...);  // Whatever you like to convey deactivation
 *   },
 *\`\`\`
 * - There are more advanced methods you can implement (see documentation below):
 * \`\`\`js
 *   touchableGetHighlightDelayMS: function() {
 *     return 20;
 *   }
 *   // In practice, *always* use a predeclared constant (conserve memory).
 *   touchableGetPressRectOffset: function() {
 *     return {top: 20, left: 20, right: 20, bottom: 100};
 *   }
 * \`\`\`
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "015-code-type-for-style",
    input: `
    /**
     * Utility type for getting the values for specific style keys.
     * # test:
     * The following is bad because position is more restrictive than 'string':
     * \`\`\`
     * type Props = {position: string};
     * \`\`\`
     *
     * You should use the following instead:
     *
     * \`\`\`
     * type Props = {position: TypeForStyleKey<'position'>};
     * \`\`\`
     *
     * This will correctly give you the type 'absolute' | 'relative'
     */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "016-print-width-80",
    input: `/**
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  *
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  * A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A
  */`,
    options: { parser: "babel-ts", jsdocPrintWidth: 80 },
  },
  {
    name: "017-markdown-format",
    input: `/**
 * Header
 * ======
 *
 * _Look,_ code blocks are formatted *too!*
 *
 * \`\`\` js
 * function identity(x) { return x }
 * \`\`\`
 *
 * Pilot|Airport|Hours
 * --|:--:|--:
 * John Doe|SKG|1338
 * Jane Roe|JFK|314
 *
 * - - - - - - - - - - - - - - -
 *
 * + List
 *  + with a [link] (/to/somewhere)
 * + and [another one]
 *
 *
 *   [another one]:  http://example.com 'Example title'
 *
 * Lorem ipsum dolor sit amet, consectetur adipiscing elit.
 * Curabitur consectetur maximus risus, sed maximus tellus tincidunt et.
 *
 * @param {string} a __very__ important!
 * @param {string} b _less_ important...
 * @param {string} a __very__ important!
 * @param {string} b *less* important...
 */`,
    options: { parser: "babel-ts" },
  },
  {
    name: "018-description-underscores",
    input: `/**
 * @param {string} a __very__ important!
 * @param {string} b _less_ important...
 */`,
    options: { parser: "babel-ts" },
  },
  {
    name: "019-hash-in-text",
    input: `/**
* JS: \`console.log("foo # bar");\`
*
* Some # text
*
* More text
*/`,
    options: { parser: "babel-ts" },
  },
  {
    name: "020-empty-lines",
    input: `/**
* Foo
*
*
*
*
*
* Bar
*
*
*
*
* @param a Baz
*/`,
    options: { parser: "babel-ts" },
  },
  {
    name: "021-star-list",
    input: `/**
 * Simplifies the token stream to ease the matching with the expected token stream.
 *
 * * Strings are kept as-is
 * * In arrays each value is transformed individually
 * * Values that are empty (empty arrays or strings only containing whitespace)
 *
 * @param {TokenStream} tokenStream
 * @returns {SimplifiedTokenStream}
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "022-bold-warning",
    input: `/**
    * Some comment text.
    *
    * **Warning:** I am a warning.
    */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "023-hash-in-code-block",
    input: `/**
* \`\`\`py
* # This program adds two numbers
*
* num1 = 1.5
* num2 = 6.3
*
* # Add two numbers
* sum = num1 + num2
*
* # Display the sum
* print('The sum of {0} and {1} is {2}'.format(num1, num2, sum))
* \`\`\`
*/
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "024-markdown-table",
    input: `
/**
 * description
 * | A| B |C |
 * | - | - | - |
 * |C | V | B |
 * |1|2|3|
 *
 * description
 *
 *
 * | A| B |C |
 * |C | V | B |
 * |1|2|3|
 * end
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "025-markdown-table-only",
    input: `
/**
 * | A| B |C |
 * | - | - | - |
 * |C | V | B |
 * |1|2|3|
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "026-table-in-param",
    input: `
/**
 * @param {string} a description
 * | A| B |C |
 * | - | - | - |
 * |C | V | B |
 * |1|2|3|
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "027-table-in-fenced-code",
    input: `
/**
 * description
 * \`\`\`
 * fenced code
 * | A| B |C |
 * | - | - | - |
 * |C | V | B |
 * |1|2|3|
 * \`\`\`
 *
 * \`\`\`
 * Second fenced table-like
 * 10
 * |--3
 * \`--4
 * \`\`\`
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "028-table-in-indented-code",
    input: `
/**
 * description
 *
 * indented code
 *
 *     | A| B |C |
 *     | - | - | - |
 *     |C | V | B |
 *     |1|2|3|
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "029-jsdoc-link",
    input: `
/**
 * Calculate the
 * {@link https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 * @param second
 * @param first
 */
 export function difference(first, second) {}


 /**
 * Calculate the
 * {@link https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * {@link https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 */

  /**
 * Calculate the
 * {@link https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between
 * {@link https://en.wikipedia.org/wiki/Complement}
 * between two sets.
 */`,
    options: { parser: "babel-ts" },
  },
  {
    name: "030-jsdoc-link-synonyms",
    input: `
/**
 * Calculate the
 * {@linkcode https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 * @param second
 * @param first
 */
 export function difference(first, second) {}


 /**
 * Calculate the
 * {@linkcode https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * {@linkcode https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 */

  /**
 * Calculate the
 * {@linkcode https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between
 * {@linkcode https://en.wikipedia.org/wiki/Complement}
 * between two sets.
 */

  /**
 * Calculate the
 * {@linkplain https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 * @param second
 * @param first
 */
 export function difference(first, second) {}


 /**
 * Calculate the
 * {@linkplain https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * {@linkplain https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between two sets.
 */

  /**
 * Calculate the
 * {@linkplain https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement difference}
 * between
 * {@linkplain https://en.wikipedia.org/wiki/Complement}
 * between two sets.
 */`,
    options: { parser: "babel-ts" },
  },
  {
    name: "031-markdown-link",
    input: `
/**
 @param {string} [dir] [Next.js](https://nextjs.org) project directory path.
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "032-jsx-tsx-css",
    input: `
 /**
  * \`\`\`js
  * let   a
  * \`\`\`
  *
  * \`\`\`jsx
  * let   a
  * \`\`\`
  *
  * \`\`\`css
  * .body {color:red;
  * }
  * \`\`\`
  *
  * \`\`\`html
  * <div class="body"  >   </   div>
  * \`\`\`
  */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "033-not-capitalizing-false",
    input: `/**

  * simplifies the token stream to ease the matching with the expected token stream.

  * Simplifies the token stream to ease the matching with the expected token stream.
  *
  * * Strings are kept as-is
  * * in arrays each value is transformed individually
  * * Values that are empty (empty arrays or strings only containing whitespace)
  *
  * @param {TokenStream} tokenStream Description
  * @returns {SimplifiedTokenStream} description
  */
`,
    options: { parser: "babel-ts", jsdocCapitalizeDescription: false },
  },
  {
    name: "034-not-capitalizing-true",
    input: `/**

  * simplifies the token stream to ease the matching with the expected token stream.

  * Simplifies the token stream to ease the matching with the expected token stream.
  *
  * * Strings are kept as-is
  * * in arrays each value is transformed individually
  * * Values that are empty (empty arrays or strings only containing whitespace)
  *
  * @param {TokenStream} tokenStream Description
  * @returns {SimplifiedTokenStream} description
  */
`,
    options: { parser: "babel-ts", jsdocCapitalizeDescription: true },
  },
  {
    name: "035-code-in-description",
    input: `
  /**
   * Inspired from react-native View
   *
   * \`\`\`js
   * import { View } from "react-native";
   *
   *
   *
   * function MyComponent() {
   *  return (
   *   <View style={{ alignItems: 'center' }}>
   *    <View variant="a" href="/" onPress={()=>{
   * history.push('/')
   * }} style={{ width:300,height:50 }} >
   *    <Text>Hello World</Text>
   *   </View>
   *  </View>
   * );
   * }
   * \`\`\`
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "036-indented-code",
    input: `
/**
 * description
 *
 *     an indented code block
 *     of a few lines.
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "037-fenced-code",
    input: `
/**
 * description
 *
 * \`\`\`
 * A fenced code block
 * spanning a few lines.
 * \`\`\`
 */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "038-link",
    input: `
  /**
   * Name of something.
   *
   * See [documentation](1) for more details.
   *
   * # 1
   * https://www.documentation.com
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "039-reference-style-link",
    input: `
  /**
   * Name of something.
   *
   * See [documentation][1] for more details.
   *
   * [1]: https://www.documentation.com
   */
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "040-reference-link-with-param",
    input: `
  /**
   * This is a [link][1] test.
   *
   * [1]: https://www.google.com/
   *
   * @param value - Any value you want to test is a number.
   */
   function test(value) {
    return true
   }
`,
    options: { parser: "babel-ts" },
  },
  {
    name: "041-table-replaced",
    input: `
    /**
     * @param options Options object for setting file system flags (default: \`"r"\`):
     *
     * | flags   | description                                                                                                    |
     * | ------- | -------------------------------------------------------------------------------------------------------------- |
     * | \`"a"\`   | Open file for appending. The file is created if it does not exist.                                             |
     * | \`"ax"\`  | Like \`"a"\` but fails if the path exists.                                                                       |
     * | \`"a+"\`  | Open file for reading and appending. The file is created if it does not exist.                                 |
     * | \`"ax+"\` | Like \`"a+"\` but fails if the path exists.                                                                      |
     * | \`"as"\`  | Open file for appending in synchronous mode. The file is created if it does not exist.                         |
     * | \`"as+"\` | Open file for reading and appending in synchronous mode. The file is created if it does not exist.             |
     * | \`"r"\`   | Open file for reading. An exception occurs if the file does not exist.                                         |
     * | \`"r+"\`  | Open file for reading and writing. An exception occurs if the file does not exist.                             |
     * | \`"rs+"\` | Open file for reading and writing in synchronous mode. Instructs the OS to bypass the local file system cache. |
     * | \`"w"\`   | Open file for writing. The file is created (if it does not exist) or truncated (if it exists).                 |
     * | \`"wx"\`  | Like \`"w"\` but fails if the path exists.                                                                       |
     * | \`"w+"\`  | Open file for reading and writing. The file is created (if it does not exist) or truncated (if it exists).     |
     * | \`"wx+"\` | Like \`"w+"\` but fails if the path exists.                                                                      |
     */
`,
    options: { parser: "babel-ts" },
  },
];

// ---------------------------------------------------------------------------
// default.test.ts
// ---------------------------------------------------------------------------
const defaultTests = [
  {
    name: "001-default-string-desc",
    input: `
  /**
   * The summary
   *
   * @default "type" description
   */
`,
  },
  {
    name: "002-convert-double-to-single",
    input: `
  /**
   * The summary
   *
   * @default "value"
   */
`,
    options: { singleQuote: true },
  },
  {
    name: "003-convert-single-to-double",
    input: `
  /**
   * The summary
   *
   * @default 'value'
   */
`,
    options: { singleQuote: false },
  },
  {
    name: "004-cant-convert-with-apostrophe",
    input: `
  /**
   * The summary
   *
   * @default "This isn't bad"
   */
`,
    options: { singleQuote: true },
  },
  {
    name: "005-default-empty-array",
    input: `
  /**
   * The summary
   *
   * @default []
   */
`,
  },
  {
    name: "006-default-empty-object",
    input: `
  /**
   * The summary
   *
   * @default {}
   */
`,
  },
  {
    name: "007-empty-default-tag",
    input: `
  /**
   * The summary
   *
   * @default
   */
`,
  },
  {
    name: "008-default-filled-array",
    input: `
    /**
     * The summary
     *
     * @default [1,'two',{three:true},['four']]
     */
`,
  },
  {
    name: "009-defaultValue-filled-array",
    input: `
    /**
     * The summary
     *
     * @defaultValue [1,'two',{three:true},['four']]
     */
`,
  },
  {
    name: "010-default-filled-object",
    input: `
    /**
     * The summary
     *
     * @default {object:'value',nestingTest:{obj:'nested'}}
     */
`,
  },
  {
    name: "011-defaultValue-filled-object",
    input: `
    /**
     * The summary
     *
     * @defaultValue {object:'value',nestingTest:{obj:'nested'}}
     */
`,
  },
  {
    name: "012-double-default-one",
    input: `
  /**
   * The summary
   *
   * @default "something"
   * @default {}
   */
`,
  },
  {
    name: "013-double-default-two",
    input: `
  /**
   * The summary
   *
   * @default {}
   * @default "something"
   */
`,
  },
  {
    name: "014-single-line-codegen",
    input: `
    /** @default codegen */
`,
  },
  {
    name: "015-multi-line-codegen",
    input: `
    /**
     * @default codegen
     */
`,
  },
  {
    name: "016-code-in-default",
    input: `/**
     * The path to the config file or directory contains the config file.
     *
     * @default process.cwd()
     */
`,
  },
];

// ---------------------------------------------------------------------------
// exampleTag.test.ts
// ---------------------------------------------------------------------------
const exampleTagTests = [
  {
    name: "001-example-js-code",
    input: `
/**
* @examples
*   var one = 5
*   var two = 10

     const resolveDescription = formatDescription(tag, description, tagString, a);
*
*   if(one > 2) { two += one

 }

* @undefiendTag
* @undefiendTag {number} name des
*/
const testFunction = (text, defaultValue, optionalNumber) => true
`,
  },
  {
    name: "002-empty-example",
    input: `/**
 * single line description
 * @example
 */

/**
 * single line description
 * @return {Boolean} Always true
 * @example
 */
`,
  },
  {
    name: "003-example-start-xml",
    input: `
  /**
   * @example <caption>TradingViewChart</caption>;
   *
   * export default Something
   */
`,
  },
  {
    name: "004-example-start-xml-with-fn",
    input: `
  /**
   * @example <caption>TradingViewChart</caption>
   *
   * function Something(){
   *   return <caption>TradingViewChart</caption>
   * }
   * export default Something
   */
`,
  },
  {
    name: "005-example-json",
    input: `
  /**
   * @example {
   *   '0%': '#afc163',
   *   '25%': '#66FF00',
   *   '50%': '#00CC00',     // ====>  linear-gradient(to right, #afc163 0%, #66FF00 25%,
   *   '75%': '#009900',     //         #00CC00 50%, #009900 75%, #ffffff 100%)
   *   '100%': '#ffffff'
   * }
   * @description
   * Then this man came to realize the truth:
   * Besides six pence, there is the moon.
   * Besides bread and butter, there is the bug.
   * And...
   * Besides women, there is the code.
   */
`,
  },
  {
    name: "006-example-idempotent",
    input: `
  /**
   * @example <caption>with selector</caption>
   *   const $ = ccashio.test(\`
   *     <div id=test>
   *       <p>Hello</p>
   *       <b><p>World</p></b>
   *     </div>
   *   \`);
  */
`,
  },
];

// ---------------------------------------------------------------------------
// typeScript.test.ts  (uses parser: "typescript")
// ---------------------------------------------------------------------------
const typescriptTests = [
  {
    name: "001-typedef-object",
    input: `
  /**
 @typedef {
    {
        "userId": string,
        "title": string,
        "profileImageLink": string,
        "identityStatus": "None",
        "isBusinessUser": boolean,
        "isResellerUser": boolean,
        "isSubUser": boolean,
        "shareCode": number,
        "referredBy": string,
        "businessName": string,
        "businessUserId": string,
        "nationalCode": string,
        "state": string,
        "city": string,
        "address": string,
        "phoneNumber": string
      }
     } User
     */
  export let User

  /**
     @typedef {
      {
        "domainId": 0,
        persianName: string,
        "englishName": string, // comment
        "resellerUserId": string,
        "isActive": true,
        "logoFileUniqueId": string,
        "logoFileName": string,
        "logoFileUrl": string,
        "domainPersianName": string,
        "domainEnglishName": string,
        "resellerUserDisplayName": string,
        "about": string
      }
     } SubDomain
     */

    /**
     @typedef {
      () => a.b
     } SubDomain
     */
`,
    options: { parser: "typescript" },
  },
  {
    name: "002-hoisted-object",
    input: `
  /**
 @typedef {
    {
        "userId": {
        title: string,
        "profileImageLink": *,
        "identityStatus": "None",
        "isBusinessUser": "isResellerUser"|"isBoolean"|  "isSubUser" |    "isNot",
        "shareCode": number,
        "referredBy": any,
        },
        id:number
      }
     } User
     */

`,
    options: { parser: "typescript" },
  },
  {
    name: "003-max-width",
    input: `
class test {
  /**
   * Replaces text in a string, using a regular expression or search string.
   * @param {string | RegExp} searchValue A string to search for.
   * @param {string | (substring: string, ...args: any[]) => string} replaceValue A string containing the text to replace for every successful match of searchValue in this string.
   * @param {string | (substring: string, ...args: any[]) => string} A_big_string_for_test A string containing the text to replace for every successful match of searchValue in this string.
   * @param {string | (substring: string, ...args: any[]) => string} replaceValue A_big_string_for_test string containing the text to replace for every successful match of searchValue in this string.
   * @param {string | (substring: string, ...args: any[]) => string} A_big_string_for_test A_big_string_for_test string containing the text to replace for every successful match of searchValue in this string.
   * @returns {StarkStringType & NativeString}
   */
  replace(searchValue, replaceValue) {
    class test{
      /**
     * Replaces text in a string, using a regular expression or search string.
     *
     * @param {string | RegExp} searchValue A string to search for.
     * @param {string | (substring: string, ...args: any[]) => string} replaceValue
     *     A string containing the text to replace for every successful match of
     *     searchValue in this string.
     * @param {string | (substring: string, ...args: any[]) => string} A_big_string_for_test
     *     A string containing the text to replace for every successful match of searchValue
     *     in this string.
     * @param {string | (substring: string, ...args: any[]) => string} replaceValue
     *     A_big_string_for_test string containing the text to replace for every
     *     successful match of searchValue in this string.
     * @param {string | (substring: string, ...args: any[]) => string} A_big_string_for_test
     *     A_big_string_for_test string containing the text to replace for every successful
     *     match of searchValue in this string.
     * @returns {StarkStringType & NativeString}
     */
        testFunction(){

        }
      }

    this._value = this._value.replace(searchValue, replaceValue);
    return this;
  }
}
`,
    options: { parser: "typescript" },
  },
  {
    name: "004-interface-deprecated",
    input: `
export interface FetchCallbackResponseArray<T, V> {
  resource: Resource<T>;
      /**
       * @deprecated Resolve clear with condition in your fetch api this function will be remove
       */
  refetch: (...arg: V[]) => void;
  /**
   * @deprecated Resolve clear with condition in your fetch api this function will be remove
   */
  clear: () => void;
}
`,
    options: { parser: "typescript" },
  },
  {
    name: "005-typedef-import",
    input: `
/**
 * @typedef {import("Foo")} Foo
 */
`,
    options: { parser: "typescript" },
  },
  {
    name: "006-union-types",
    input: `
/**
 * @typedef {{ foo: string } | { bar: string; manyMoreLongArguments: object } | { baz: string }} Foo
 */
`,
    options: { parser: "typescript" },
  },
  {
    name: "007-long-union-types",
    input: `
  /**
   * Gets a configuration object assembled from environment variables and .env configuration files.
   *
   * @memberof Config
   * @function getEnvConfig
   * @returns {Config.SomeConfiguration | Config.SomeOtherConfiguration | Config.AnotherConfiguration | Config.YetAnotherConfiguration } The environment configuration
   */
  export default () => configurator.config;
`,
    options: { parser: "typescript" },
  },
  {
    name: "008-type-imports",
    input: `
/**
 * @import BM, { B as B1,
 * B2   , B4 } from 'moduleb'
 * @typedef {Object} Foo
 * @import BMain, {B3  } from "moduleb"
 * @import {A} from 'modulea'
 */
/**
 * @import BDefault, {        B5 } from   './moduleb'
 * @import C    from    "modulec"
 */
`,
    options: { parser: "typescript" },
  },
];

// ---------------------------------------------------------------------------
// tagGroup.test.ts (uses jsdocSeparateTagGroups: true)
// ---------------------------------------------------------------------------
const tagGroupTests = [
  {
    name: "001-tag-group",
    input: `
  /**
   * Aliquip ex proident tempor eiusmod aliquip amet. Labore commodo nulla tempor
   * consequat exercitation incididunt non. Duis laboris reprehenderit proident
   * proident.
   * @see {@link http://acme.com}
   * @example
   *   const foo = 0;
   *   const a = "";
   *   const b = "";
   *
   * @param id A test id.
   * @throws Minim sit ad commodo ut dolore magna magna minim consequat. Ex
   *   consequat esse incididunt qui voluptate id voluptate quis ex et. Ullamco
   *   cillum nisi amet fugiat.
   * @return Minim sit a.
   */

`,
    options: { jsdocSeparateTagGroups: true },
  },
  {
    name: "002-inconsistent-formatting",
    input: `
    /**
     * Aliquip ex proident tempor eiusmod aliquip amet. Labore commodo nulla tempor
     * consequat exercitation incididunt non. Duis laboris reprehenderit proident
     * proident.
     *
     * @example
     *   const foo = 0;
     *
     *
     * @param id A test id.
     *
     * @throws Minim sit ad commodo ut dolore magna magna minim consequat. Ex
     *   consequat esse incididunt qui voluptate id voluptate quis ex et. Ullamco
     *   cillum nisi amet fugiat.
     * @see {@link http://acme.com}
     */
`,
    options: { jsdocSeparateTagGroups: true },
  },
];

// ---------------------------------------------------------------------------
// singleTag.test.ts
// ---------------------------------------------------------------------------
const singleTagTests = [
  {
    name: "001-single-tag",
    input: `
  /**
* @param {  string   }    param0 description
   */
function fun(param0){}

  export const SubDomain = {
    /**
     * @returns {import('axios').AxiosResponse<import('../types').SubDomain>}
     */
    async subDomain(subDomainAddress) {
    },
  };

`,
  },
];

// ---------------------------------------------------------------------------
// template.test.ts (uses parser: "babel-flow")
// ---------------------------------------------------------------------------
const templateTests = [
  {
    name: "001-template-callback",
    input: `
/**
 * @template T
 * @callback CallbackName
 * @param {GetStyles<T>} getStyles
 * @returns {UseStyle<T>}
 */
`,
    options: { parser: "babel-flow", jsdocSeparateReturnsFromParam: true },
  },
  {
    name: "002-extends",
    input: `
 /**
  * The bread crumbs indicate the navigate path and trigger the active page.
  * @class
  * @typedef {object} props
  * @prop        {any} navigation
  * @extends {PureComponent<props>}
  */
 export class BreadCrumbs extends PureComponent {}
`,
    options: { parser: "babel-flow", jsdocSeparateReturnsFromParam: true },
  },
  {
    name: "003-typeparam-callback",
    input: `
/**
 * @typeParam T
 * @callback CallbackName
 * @param {GetStyles<T>} getStyles
 * @returns {UseStyle<T>}
 */
`,
    options: { parser: "babel-flow", jsdocSeparateReturnsFromParam: true },
  },
];

// ---------------------------------------------------------------------------
// dottedNames.test.ts
// ---------------------------------------------------------------------------
const dottedNamesTests = [
  {
    name: "001-dotted-names",
    input: `
  /**
   * @param {object} data
   * @param {string} data.userName
   * @param {string} data.password
   *
   * @typedef {object} LoginResponse
   * @property {string} token
   * @property {number} expires
   * @property {boolean} mustChangePassword
   *
   * @returns {import('axios').AxiosResponse<LoginResponse>}
   */
    function a(){}
`,
  },
];

// ---------------------------------------------------------------------------
// remarks.test.ts  (tsdoc: false only)
// ---------------------------------------------------------------------------
const remarksTests = [
  {
    name: "001-remarks-no-tsdoc",
    input: `
  /**
   * Just a simple test
   *
   * @remarks
   *   - Remark 1
   *   - Remark 2
   *   - Remark 3
   */
`,
    options: { tsdoc: false },
  },
];

// ---------------------------------------------------------------------------
// react.test.ts
// ---------------------------------------------------------------------------
const reactTests = [
  {
    name: "001-react-xaxis",
    input: `
  import React, { memo } from "react";
  import { Text, View, StyleSheet } from "react-native";
  import * as d3Scale from "d3-scale";
  import * as array from "d3-array";
  import Svg, { G, Text as SVGText } from "react-native-svg";
  import { useLayout, useInlineStyle } from "./hooks";

  /**
   * @typedef {object} XAxisProps
   * @property {number} [spacingOuter]
   * Spacing between the labels. Only applicable if
   * \`scale=d3Scale.scaleBand\` and should then be equal to \`spacingOuter\` prop on the
   * actual BarChart
   *
   * Default is \`0.05\`
   * @property {number} [spacingInner] Spacing between the labels. Only applicable if
   * \`scale=d3Scale.scaleBand\` and should then be equal to \`spacingInner\` prop on the
   * actual BarChart
   *
   * Default is \`0.05\`
   * @property {d3Scale.scaleLinear} [scale] Should be the same as passed into the charts \`xScale\`
   * Default is \`d3Scale.scaleLinear\`
   *
   * @property {()=>any} [xAccessor] Default is \`({index}) => index\`
   * @property {number} [max]
   * @property {number} [min]
   */

  /**
   * @type {React.FC<XAxisProps & import("react-native-svg").TextProps>}
   */
  const XAxis = memo(
    ({
      contentInset: { left = 0, right = 0 } = {},
      style,
      data,
      numberOfTicks,
      children,
      min,
      max,
      spacingInner = 0.05,
      spacingOuter = 0.05,
      xAccessor = ({ index }) => index,
      scale = d3Scale.scaleLinear,
      formatLabel = value => value,
      ...svg
    })=>{})
`,
  },
];

// ---------------------------------------------------------------------------
// modern.test.ts (uses parser: "typescript")
// ---------------------------------------------------------------------------
const modernTests = [
  {
    name: "001-modern-array-types",
    input: `
  /**
   * @typedef {import("react-native-reanimated").default.Adaptable<number>} Adaptable
   * @param {Adaptable} animNode
   * @param {object} InterpolationConfig
   * @param {ReadonlyArray<Adaptable>} InterpolationConfig.inputRange Like [0,1]
   * @param {Array<string>} InterpolationConfig.outputRange Like ["#0000ff","#ff0000"]
   * @param {import("react-native-reanimated").default.Extrapolate} [InterpolationConfig.extrapolate]
   * @param {Array<Foo<Bar>>} arg1
   * @param {Array<(item: Foo<Bar>) => Bar<number>> | Array<number> | Array<string>} arg2
   * @param {Array.<(item: Foo.<Bar>) => Bar.<number>> | Array.<number> | Array.<'Foo.<>'>} arg3
   * @param {"Array.<(item: Foo.<Bar>) => Bar.<number>> | Array.<number> | Array.<'Foo.<>'>"} arg4
   * @param {Array<Array<Array<number>>>} arg5
   * @param {{ foo: Array<number>; bar: Array<string> }} arg6
   *
   */
  function a(){}
`,
    options: { parser: "typescript" },
  },
];

// ---------------------------------------------------------------------------
// objectProperty.test.ts (uses parser: "babel-flow")
// ---------------------------------------------------------------------------
const objectPropertyTests = [
  {
    name: "001-object-property",
    input: `

/**\t
 * Copyright (c) 2015-present, Facebook, Inc.\t
 * All rights reserved.\t
 *
 *    This source code is licensed under the license found in the LICENSE file in\t
 * the root directory of this source tree.\t
 */\t

/**\t
 * Copyright (c) 2015-present, Facebook, Inc.\t
 * All rights reserved.\t
 *
 * This source code is licensed under the license found in the LICENSE file in\t
 * the root directory of this source tree.\t
 */\t

const obj = {
  /**
* @param {object} [filters]
   * @param {string} [filters.searchInput]
   * @param {boolean} [filters.isActive]
   * @param {boolean} [filters.isPerson]
   * @param {import('../types').IdentityStatus} [filters.identityStatuses]
   * @param {string} [filters.lastActivityFrom] YYYY-MM-DD
   * @param {string} [filters.lastActivityTo]
   * @param {string} [filters.registeredFrom]
   * @param {string} [filters.registeredTo]
   * @param {number} [filters.skip]
   * @param {number} [filters.take]
   * @param {string} [filters.orderBy]
   * @param {boolean} [filters.orderDesc]
   * @returns {import('axios').AxiosResponse<
    import('../types').ResellerUserIntroduced[]
  >}
  */
  a(filters) {},
};
`,
    options: { parser: "babel-flow" },
  },
];

// ---------------------------------------------------------------------------
// File-based tests
// ---------------------------------------------------------------------------
async function generateFileTests() {
  const filesDir = resolve(PLUGIN_DIR, "tests", "files");

  // Only include files with default-ish options (skip tsdoc, custom order, prism)
  const fileTests = [
    { name: "typeScript.js", options: {} },
    { name: "typeScript.ts", options: {} },
    { name: "types.ts", options: {} },
    { name: "order.jsx", options: {} },
    { name: "create-ignorer.js", options: {} },
  ];

  for (const { name, options } of fileTests) {
    const filePath = resolve(filesDir, name);
    let input;
    try {
      input = readFileSync(filePath, "utf-8");
    } catch {
      console.warn(`  SKIP file ${name}: not found at ${filePath}`);
      continue;
    }

    const fileExt = name.split(".").pop();
    const parser =
      fileExt === "ts" ? "typescript" : fileExt === "tsx" ? "typescript" : fileExt === "jsx" ? "babel" : "babel";

    const formatOpts = {
      ...options,
      parser,
      filepath: filePath,
      trailingComma: "all",
    };

    const output = await format(input, formatOpts);

    // Use original extension for the fixture file name
    const baseName = name.replace(/\.[^.]+$/, "");
    const safeName = baseName.replace(/\./g, "-");
    const fixtureDir = resolve(FIXTURES_DIR, "files");
    mkdirSync(fixtureDir, { recursive: true });
    writeFileSync(resolve(fixtureDir, `${safeName}.${fileExt}`), input);
    writeFileSync(resolve(fixtureDir, `${safeName}.output.${fileExt}`), output);
    console.log(`  wrote files/${safeName}.${fileExt}`);
  }
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  console.log("Generating jsdoc conformance fixtures...\n");

  const allCategories = [
    ["main", mainTests],
    ["descriptions", descriptionsTests],
    ["default", defaultTests],
    ["example-tag", exampleTagTests],
    ["typescript", typescriptTests],
    ["tag-group", tagGroupTests],
    ["single-tag", singleTagTests],
    ["template", templateTests],
    ["misc/dotted-names", dottedNamesTests],
    ["misc/remarks", remarksTests],
    ["misc/react", reactTests],
    ["misc/modern", modernTests],
    ["misc/object-property", objectPropertyTests],
  ];

  let totalCount = 0;
  let errorCount = 0;

  for (const [category, tests] of allCategories) {
    console.log(`\n[${category}] (${tests.length} tests)`);
    for (const test of tests) {
      try {
        const opts = test.options || {};
        const output = await format(test.input, opts);
        writeFixture(category, test.name, test.input, output, opts);
        totalCount++;
      } catch (err) {
        console.error(`  ERROR ${category}/${test.name}: ${err.message}`);
        errorCount++;
      }
    }
  }

  // File-based tests
  console.log("\n[files]");
  try {
    await generateFileTests();
    totalCount += 5; // approximate
  } catch (err) {
    console.error(`  ERROR files: ${err.message}`);
    errorCount++;
  }

  console.log(`\nDone! Generated ${totalCount} fixtures, ${errorCount} errors.`);
}

main().catch(console.error);
