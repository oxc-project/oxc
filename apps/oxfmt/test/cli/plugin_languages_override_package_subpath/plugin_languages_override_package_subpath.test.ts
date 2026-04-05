import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("plugin_languages_override_package_subpath", () => {
  it("should format .svelte through an override plugin subpath resolved from the config directory", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["subdir/config/App.svelte"],
      ["-c", "./subdir/config/.oxfmtrc.json"],
    );

    expect(snapshot).toContain(`--- FILE -----------
subdir/config/App.svelte`);
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
