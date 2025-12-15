import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("ignore_symlink", () => {
  it("should ignore symlinks by default", async () => {
    const cwd = join(fixturesDir, "symlink_dirs");
    const testCases = [["--check"]];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
