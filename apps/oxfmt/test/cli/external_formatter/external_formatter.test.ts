import { describe, expect, it } from "vite-plus/test";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("external_formatter", () => {
  it("should format json by default", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, [
      "foo.json",
      "package.json",
      "tsconfig.dummy.json",
    ]);
    expect(snapshot).toMatchSnapshot();
  });
});
