import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("js-in-xxx", () => {
  it("should format with full-config", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["test.md", "test.mdx", "app.vue", "multi-script.vue", "no-imports.vue"],
      ["--config", "oxfmtrc-full.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
