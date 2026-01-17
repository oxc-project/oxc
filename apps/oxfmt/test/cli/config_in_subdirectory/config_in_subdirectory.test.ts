import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("config_in_subdirectory", () => {
  it("should not panic when config with `ignorePatterns` is in subdirectory", async () => {
    // Simulates running from project root:
    // ```
    // oxfmt -c ./subdir/.oxfmtrc.json ./subdir
    // ```
    // Where the config has `ignorePatterns`, causing matcher root to be `./subdir/`
    const snapshot = await runAndSnapshot(fixturesDir, [
      ["--check", "-c", "./subdir/.oxfmtrc.json", "./subdir"],
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should not panic when target path is absolute and outside matcher root", async () => {
    // fixtures/
    //   subdir/           <- matcher root (has `.oxfmtrc.json` with `ignorePatterns`)
    //   outside/file.js   <- target path (outside of matcher root)
    const snapshot = await runAndSnapshot(fixturesDir, [
      // Target path is outside of matcher root - should not panic
      ["--check", "-c", "./subdir/.oxfmtrc.json", join(fixturesDir, "outside", "file.js")],
    ]);
    expect(snapshot).toMatchSnapshot();
  });
});
