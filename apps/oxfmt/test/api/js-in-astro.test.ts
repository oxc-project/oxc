import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format js-in-astro with prettier-plugin-oxfmt", () => {
  it("should format .astro frontmatter w/ sort-imports", async () => {
    const input = `---
import z from "z";
  import a from "a";
    import m from "m";

const   x  =    1;
---
<div>{x}</div>
`;
    const result = await format("Layout.astro", input, {
      experimentalSortImports: {},
    });
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .astro <script> blocks", async () => {
    const input = `---
const title = "Hello";
---
<html>
  <body>
    <h1>{title}</h1>
    <script>
import z from "z";
  import a from "a";
    const   y  =   2;
    </script>
  </body>
</html>
`;
    const result = await format("Page.astro", input, {
      experimentalSortImports: {},
    });
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .astro w/ Tailwind class sorting", async () => {
    const input = `---
const title = "Hello";
---
<div class="flex p-4">text</div>
<div class="p-4 flex">text</div>
`;
    const result = await format("Component.astro", input, {
      experimentalTailwindcss: {},
    });
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .astro frontmatter-only (no template)", async () => {
    const input = `---
import z from "z";
import a from "a";
export const prerender   =   true;
---
`;
    const result = await format("Config.astro", input, {
      experimentalSortImports: {},
    });
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .astro without frontmatter", async () => {
    const input = `<div class="p-4 flex">
  <p>Hello</p>
</div>
`;
    const result = await format("NoFrontmatter.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .astro with multiple <script> blocks", async () => {
    const input = `---
const title = "Hello";
---
<html>
  <body>
    <h1>{title}</h1>
    <script>
      const   a  =   1;
    </script>
    <script>
      const   b  =   2;
    </script>
  </body>
</html>
`;
    const result = await format("MultiScript.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle .astro with client:load directives", async () => {
    const input = `---
import Counter from "../components/Counter";
---
<Counter   client:load  />
`;
    const result = await format("ClientDirective.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle .astro with TypeScript in frontmatter", async () => {
    const input = `---
interface Props {
  title:   string;
  count:number;
}

const { title,    count } = Astro.props;
---
<h1>{title} ({count})</h1>
`;
    const result = await format("TypedComponent.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle .astro with class:list directive", async () => {
    const input = `---
const isActive = true;
---
<div class:list={["base",   {active:isActive}]}>text</div>
`;
    const result = await format("ClassList.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle .astro with set:html directive", async () => {
    const input = `---
const   rawHtml   =   "<em>Hello</em>";
---
<div set:html={rawHtml} />
`;
    const result = await format("SetHtml.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should pass astroAllowShorthand option", async () => {
    const input = `---
const title = "Hello";
---
<Component {title} />
`;
    const result = await format("Shorthand.astro", input, {
      astroAllowShorthand: true,
    });
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should pass astroSkipFrontmatter option", async () => {
    const input = `---
import z from "z";
  import a from "a";
    const   x  =    1;
---
<div>{x}</div>
`;
    const withSkip = await format("Skip.astro", input, {
      astroSkipFrontmatter: true,
    });
    const withoutSkip = await format("NoSkip.astro", input, {
      astroSkipFrontmatter: false,
    });
    // With skip: frontmatter should be preserved as-is
    expect(withSkip.code).toContain("import z from \"z\";\n  import a from \"a\"");
    // Without skip: frontmatter should be formatted
    expect(withoutSkip.code).not.toContain("  import a");
    expect(withSkip.errors).toStrictEqual([]);
    expect(withoutSkip.errors).toStrictEqual([]);
  });

  it("should handle empty .astro file", async () => {
    const result = await format("Empty.astro", "", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle empty frontmatter", async () => {
    const input = `---
---
<div>hello</div>
`;
    const result = await format("EmptyFrontmatter.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should pass formatting options through to Prettier", async () => {
    const input = `---
const x = "hello";
---
<div>{x}</div>
`;
    const result = await format("Options.astro", input, {
      printWidth: 40,
      singleQuote: true,
    });
    expect(result.code).toContain("'hello'");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle <script is:inline>", async () => {
    const input = `---
const title = "Hello";
---
<script is:inline>
  const   x  =   1;
</script>
`;
    const result = await format("Inline.astro", input, {});
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });
});
