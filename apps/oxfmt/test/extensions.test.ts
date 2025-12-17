import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("extensions", () => {
  it("should check files with supported extensions", async () => {
    const cwd = join(fixturesDir, "extensions");
    const testCases = [["--check"]];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
