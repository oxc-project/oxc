import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { format } from "../../dist/index.js";

const sveltePluginPath = join(
  import.meta.dirname,
  "..",
  "cli",
  "plugin_languages",
  "fixtures",
  "plugins",
  "prettier-plugin-svelte.mjs",
);

describe("plugin-defined formatter languages", () => {
  it("should format .svelte through a configured external plugin", async () => {
    const input = `<style>h1{color:red}</style>\n<h1>Hello {name}</h1>\n<script>export let name = \"world\";</script>\n`;

    const result = await format("App.svelte", input, {
      plugins: [sveltePluginPath],
      svelteSortOrder: "scripts-markup-styles",
    } as any);

    expect(result.code).toBe(
      `<script>\nexport let name = \"world\";\n</script>\n\n<h1>Hello {name}</h1>\n\n<style>\nh1{color:red}\n</style>\n`,
    );
    expect(result.errors).toStrictEqual([]);
  });
});
