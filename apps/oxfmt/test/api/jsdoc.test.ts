import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("JSDoc", () => {
  it("should format fenced css and html through the api path", async () => {
    // Keep this coverage in apps/oxfmt instead of the standalone JSDoc conformance runner:
    // CSS/HTML fenced blocks only format when embedded-language callbacks are wired through
    // the oxfmt app API. The Rust-only JSDoc fixture runner intentionally skips that case.
    const source = `
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
`.trim();

    const result = await format("a.ts", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(
      `
/**
 * \`\`\`js
 * let a;
 * \`\`\`
 *
 * \`\`\`jsx
 * let a;
 * \`\`\`
 *
 * \`\`\`css
 * .body {
 *   color: red;
 * }
 * \`\`\`
 *
 * \`\`\`html
 * <div class="body"></div>
 * \`\`\`
 */
`.trimStart(),
    );
  });

  it("should format fenced scss, less and graphql through the native dispatch registry", async () => {
    const source = `
/**
 * \`\`\`scss
 * .a { .b { color :blue } }
 * \`\`\`
 *
 * \`\`\`less
 * .mixin() { color: red; }
 * \`\`\`
 *
 * \`\`\`graphql
 * query Q  { user { id   name } }
 * \`\`\`
 */
`.trim();

    const result = await format("a.ts", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(
      `
/**
 * \`\`\`scss
 * .a {
 *   .b {
 *     color: blue;
 *   }
 * }
 * \`\`\`
 *
 * \`\`\`less
 * .mixin() {
 *   color: red;
 * }
 * \`\`\`
 *
 * \`\`\`graphql
 * query Q {
 *   user {
 *     id
 *     name
 *   }
 * }
 * \`\`\`
 */
`.trimStart(),
    );
  });

  it("should format fenced json, json5 and yaml via the native registry", async () => {
    const source = `
/**
 * \`\`\`json
 * {"a":1,   "b":[2,3]}
 * \`\`\`
 *
 * \`\`\`json5
 * {a:1,}
 * \`\`\`
 *
 * \`\`\`yaml
 * key:   value
 * list:
 *   -   1
 * \`\`\`
 */
`.trim();

    const result = await format("a.js", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(
      `
/**
 * \`\`\`json
 * { "a": 1, "b": [2, 3] }
 * \`\`\`
 *
 * \`\`\`json5
 * { a: 1 }
 * \`\`\`
 *
 * \`\`\`yaml
 * key: value
 * list:
 *   - 1
 * \`\`\`
 */
`.trimStart(),
    );
  });

  it("should keep fenced code verbatim for languages outside the registry", async () => {
    const source = `
/**
 * \`\`\`ruby
 * var =   1
 * \`\`\`
 *
 * \`\`\`python
 * x   =   1
 * \`\`\`
 */
`.trim();

    const result = await format("a.ts", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(`${source}\n`);
  });

  it("should keep fenced code verbatim when the Rust formatter cannot parse it", async () => {
    const source = `
/**
 * \`\`\`css
 * .a { color: }}} broken {{{
 * \`\`\`
 *
 * \`\`\`graphql
 * fragment Broken on T {
 * \`\`\`
 */
`.trim();

    const result = await format("a.ts", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(`${source}\n`);
  });

  it("should format jsdoc in vue the same as in plain js", async () => {
    const jsdocComment = `
/**
 * @param {string}   name   -  The   name
 * @returns  {boolean}   Whether   it   is   valid
 */`;

    const jsSource = `${jsdocComment.trim()}
function validate(name) {}
`.trim();

    const vueSource = `<script>
${jsdocComment}
function validate(name) {}
</script>
`;

    const jsResult = await format("a.js", jsSource, { jsdoc: {} });
    const vueResult = await format("a.vue", vueSource, { jsdoc: {} });

    expect(jsResult.errors).toStrictEqual([]);
    expect(vueResult.errors).toStrictEqual([]);
    // The JSDoc inside <script> should be formatted the same as in plain JS
    expect(vueResult.code).toContain(jsResult.code.trim());
  });

  it("should format css-in-jsdoc-in-js-in-vue", async () => {
    const source = `<script>
/**
 * \`\`\`css
 * .body {color:red;
 * }
 * \`\`\`
 */
function foo() {}
</script>
`;

    const result = await format("a.vue", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toContain(
      [" * ```css", " * .body {", " *   color: red;", " * }", " * ```"].join("\n"),
    );
  });
});
