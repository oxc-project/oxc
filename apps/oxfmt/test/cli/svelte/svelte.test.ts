import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("svelte", () => {
  it("should format svelte files", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["test.svelte"]);
    expect(snapshot).toMatchSnapshot();
  });
});
