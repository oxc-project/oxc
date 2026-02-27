import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format js-in-svelte with prettier-plugin-oxfmt", () => {
  it("should format .svelte w/ sort-imports", async () => {
    const input = `
<script>
import z from "z";
  import a from "a";
    import m from "m";
</script>

<div>hello</div>
`;
    const result = await format("App.svelte", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toMatchInlineSnapshot(`
      "<script>
        import a from "a";
        import m from "m";
        import z from "z";
      </script>

      <div>hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .svelte w/ sort-tailwindcss", async () => {
    const input = `
<script>
import clsx from "clsx";
import z from "z";
import a from "a";

const cls = clsx("p-4 flex");
</script>

<div class="p-4 flex">hello</div>
`;
    const result = await format("App.svelte", input, {
      experimentalSortImports: {},
      sortTailwindcss: { functions: ["clsx"] },
    });

    expect(result.code).toMatchInlineSnapshot(`
      "<script>
        import a from "a";
        import clsx from "clsx";
        import z from "z";

        const cls = clsx("flex p-4");
      </script>

      <div class="flex p-4">hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });
});
