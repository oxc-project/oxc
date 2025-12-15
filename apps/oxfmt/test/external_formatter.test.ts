import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures", "external_formatter");

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
