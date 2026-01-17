import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("external_formatter_with_config", () => {
  it("should pass config options to prettier", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["test.vue"]);
    expect(snapshot).toMatchSnapshot();
  });
});
