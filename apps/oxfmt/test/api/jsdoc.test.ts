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

    const result = await format("a.ts", source, { jsdoc: true });
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
});
