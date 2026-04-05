import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("plugin_languages_package", () => {
  it("should format .svelte through a package plugin from local node_modules", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["App.svelte"]);

    expect(snapshot).toContain(`--- FILE -----------
App.svelte`);
    expect(snapshot).toContain(
      `--- BEFORE ---------
<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>`,
    );
    expect(snapshot).toContain(
      `--- AFTER ----------
<script>
export let name = "world";
</script>

<h1>Hello {name}</h1>

<style>
h1{color:red}
</style>`,
    );
  });
});
