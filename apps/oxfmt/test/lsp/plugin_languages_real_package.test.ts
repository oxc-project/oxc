import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { formatFixture, formatFixtureAfterConfigChange, formatFixtureContent } from "./utils";

const fixturesDir = join(
  import.meta.dirname,
  "..",
  "cli",
  "plugin_languages_real_package",
  "fixtures",
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

describe("LSP real prettier-plugin-svelte integration", () => {
  it("should format .svelte files through the real package from local node_modules", async () => {
    const snapshot = await formatFixture(fixturesDir, "App.svelte", "svelte");

    expect(snapshot).toContain(`--- BEFORE ---------
${input}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${expectedOutput}`.trimEnd());
  });

  it("should discover override-scoped real prettier-plugin-svelte config in LSP", async () => {
    const snapshot = await formatFixture(fixturesDir, "subdir/config/App.svelte", "svelte");

    expect(snapshot).toContain(`--- BEFORE ---------
${input}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${expectedOutput}`.trimEnd());
  });

  it.each([
    "untitled://Untitled-svelte",
    "vscode-userdata://svelte",
    "ccsettings://svelte",
  ])("should format in-memory Svelte documents for %s", async (uri) => {
    const snapshot = await formatFixtureContent(fixturesDir, "App.svelte", uri, "svelte");

    expect(snapshot).toContain(`--- URI -----------
${uri}`);
    expect(snapshot).toContain(`--- BEFORE ---------
${input}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${expectedOutput}`.trimEnd());
  });

  it("should restart formatter plugin support after LSP config changes", async () => {
    const snapshot = await formatFixtureAfterConfigChange(
      fixturesDir,
      "App.svelte",
      "svelte",
      {
        "fmt.configPath": "./empty.json",
      },
      {
        "fmt.configPath": null,
      },
    );

    expect(snapshot).toContain(`--- BEFORE ---------
${input}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER FIRST FORMAT ----------
${input}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER SECOND FORMAT ----------
${expectedOutput}`.trimEnd());
  });
});
