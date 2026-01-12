import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("ignore_symlink", () => {
  it("should ignore symlinks by default", async () => {
    const testCases = [["--check"]];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
