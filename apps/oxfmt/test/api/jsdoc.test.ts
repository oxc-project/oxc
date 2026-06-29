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

  it("should format fenced scss, less and graphql through the Rust standalone path", async () => {
    // Since plan Step 5-1, these languages format via `oxc_formatter_css` /
    // `oxc_formatter_graphql` (string-in/string-out) instead of Prettier.
    // scss/less use their own variant (the old Prettier path parsed all of
    // css/scss/less as scss).
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

  it("should keep fenced code verbatim when the Rust formatter cannot parse it", async () => {
    // Parse errors inside comments are NOT diagnostics and do NOT fall back
    // to Prettier: the block stays as-is.
    // - css: genuinely broken
    // - graphql: draft-spec syntax (fragment spread arguments, graphql-js 17)
    //   that the apollo-parser fork does not cover yet
    const source = `
/**
 * \`\`\`css
 * .a { color: }}} broken {{{
 * \`\`\`
 *
 * \`\`\`graphql
 * fragment F on T { ...G(x: 1) }
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
