import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("vcs_dirs", () => {
  it("should ignore VCS directories by default", async () => {
    const testCases = [["--check"]];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should ignore VCS directories when expanding globs", async () => {
    const testCases = [["--check", "**/*.js"]];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
