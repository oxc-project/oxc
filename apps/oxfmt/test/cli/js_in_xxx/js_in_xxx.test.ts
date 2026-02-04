import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("js-in-xxx", () => {
  it("should format .vue with sort-imports and sort-tailwindcss", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["app.vue"],
      ["--config", "vue-full.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("should format multiple <script> tags with semi: false", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["multi-script.vue"],
      ["--config", "no-semi.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
