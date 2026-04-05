import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { format } from "../../dist/index.js";

const fixturesDir = join(import.meta.dirname, "..", "cli", "plugin_languages_package", "fixtures");

describe("package-defined formatter languages", () => {
  it("should format .svelte through a package plugin resolved from cwd", async () => {
    const input = `<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>
`;
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("App.svelte", input, {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles",
      } as any);

      expect(result.code).toBe(
        `<script>
export let name = "world";
</script>

<h1>Hello {name}</h1>

<style>
h1{color:red}
</style>
`,
      );
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });
});
