import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("vcs_dirs", () => {
  it("should ignore VCS directories by default", async () => {
    const cwd = join(fixturesDir, "vcs_dirs");
    const testCases = [["--check"]];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
