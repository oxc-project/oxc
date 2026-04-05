import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("plugin_languages", () => {
  it("should format plugin-defined .svelte files from config plugins", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["App.svelte"]);

    expect(snapshot).toContain(`--- FILE -----------\nApp.svelte`);
    expect(snapshot).toContain(
      `--- BEFORE ---------\n<style>h1{color:red}</style>\n<h1>Hello {name}</h1>\n<script>export let name = "world";</script>`,
    );
    expect(snapshot).toContain(
      `--- AFTER ----------\n<script>\nexport let name = "world";\n</script>\n\n<h1>Hello {name}</h1>\n\n<style>\nh1{color:red}\n</style>`,
    );
  });
});
