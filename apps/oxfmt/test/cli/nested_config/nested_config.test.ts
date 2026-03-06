import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runCli, runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("nested_config", () => {
  // Structure:
  //   fixtures/
  //     .oxfmtrc.json          # { "semi": true }
  //     src/root.js            # `const a = 1` (no semi → needs formatting by root config)
  //     pkg-a/
  //       .oxfmtrc.json        # { "semi": false }
  //       src/file.js          # `const a = 1;` (has semi → needs formatting by nested config)
  //     pkg-b/
  //       .oxfmtrc.json        # { "semi": false, "ignorePatterns": ["src/ignored.js"] }
  //       src/file.js          # `const a = 1;` (has semi → needs formatting by nested config)
  //       src/ignored.js       # Should be ignored by nested ignorePatterns

  it("should apply nearest config to each file", async () => {
    // All files need formatting: root.js by root config, pkg-*/src/file.js by nested configs.
    // pkg-b/src/ignored.js is skipped by nested ignorePatterns.
    const result = await runCli(fixturesDir, ["--check", "!*.json"]);
    // Exit code 1 = format issues found
    expect(result.exitCode).toBe(1);
    // 3 files should be flagged (root.js, pkg-a/src/file.js, pkg-b/src/file.js)
    // pkg-b/src/ignored.js should NOT appear (nested ignorePatterns)
    expect(result.stdout).toContain("src/root.js");
    expect(result.stdout).toContain("pkg-a/src/file.js");
    expect(result.stdout).toContain("pkg-b/src/file.js");
    expect(result.stdout).not.toContain("ignored.js");
    expect(result.stdout).toContain("3 files");
  });

  it("--config skips nested config discovery", async () => {
    // With explicit --config, only root config (semi: true) applies everywhere.
    // pkg-a/src/file.js and pkg-b/src/file.js have semi → already formatted for root config.
    // Only src/root.js (no semi) is flagged.
    // pkg-b/src/ignored.js is NOT ignored (nested ignorePatterns not loaded).
    const result = await runCli(fixturesDir, [
      "--check",
      "!*.json",
      "--config",
      ".oxfmtrc.json",
    ]);
    expect(result.exitCode).toBe(1);
    expect(result.stdout).toContain("src/root.js");
    expect(result.stdout).not.toContain("pkg-a/src/file.js");
    expect(result.stdout).not.toContain("pkg-b/src/file.js");
    // ignored.js is badly formatted for any config, so it should appear
    expect(result.stdout).toContain("ignored.js");
    expect(result.stdout).toContain("2 files");
  });

  it("write mode applies correct config per file", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["src/root.js", "pkg-a/src/file.js", "pkg-b/src/file.js"],
      ["!*.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
