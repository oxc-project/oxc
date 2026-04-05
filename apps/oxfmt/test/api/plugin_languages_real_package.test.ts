import { describe, expect, it } from "vitest";
import { createRequire } from "node:module";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { format } from "../../dist/index.js";

const fixturesDir = join(import.meta.dirname, "..", "cli", "plugin_languages_real_package", "fixtures");
const realPluginPath = join(
  fixturesDir,
  "node_modules",
  "prettier-plugin-svelte",
  "src",
  "index.js",
);
const input = `<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>
`;
const expectedOutput = `<script>
  export let name = "world";
</script>

<h1>Hello {name}</h1>

<style>
  h1 {
    color: red;
  }
</style>
`;

describe("real prettier-plugin-svelte integration", () => {
  it("should resolve svelte/compiler from the fixture-local node_modules", () => {
    const requireFromPlugin = createRequire(realPluginPath);
    const resolvedCompiler = requireFromPlugin.resolve("svelte/compiler");

    expect(resolvedCompiler).toBe(join(fixturesDir, "node_modules", "svelte", "compiler.js"));
  });

  it("should format .svelte through the real package resolved from cwd", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("App.svelte", input, {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any);

      expect(result.code).toBe(expectedOutput);
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });

  it("should format .svelte through a direct real plugin object passed to the API", async () => {
    const { default: sveltePlugin } = await import(pathToFileURL(realPluginPath).href);

    const result = await format("App.svelte", input, {
      plugins: [sveltePlugin],
      svelteSortOrder: "scripts-markup-styles-options",
      tabWidth: 2,
    } as any);

    expect(result.code).toBe(expectedOutput);
    expect(result.errors).toStrictEqual([]);
  });

  it("preserves a missing final newline when it is the only real-plugin difference", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("AlreadyFormattedNoFinalNewline.svelte", expectedOutput.trimEnd(), {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any);

      expect(result.code).toBe(expectedOutput.trimEnd());
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });

  it("preserves a missing final newline for an otherwise empty real-plugin file", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("EmptyNoFinalNewline.svelte", "", {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any);

      expect(result.code).toBe("");
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });
});
