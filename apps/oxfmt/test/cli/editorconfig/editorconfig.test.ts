import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot, runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("editorconfig", () => {
  // .editorconfig:
  //   [*] indent_style=tab, indent_size=4, max_line_length=40
  //
  // Expected: useTabs=true, tabWidth=4, printWidth=40
  // - Code should be broken into multiple lines (printWidth=40)
  // - Indentation should use tabs (not spaces)
  it("basic settings from .editorconfig", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] indent_style=tab, indent_size=8
  // .oxfmtrc.json:
  //   useTabs=false, tabWidth=2
  //
  // Expected: useTabs=false, tabWidth=2 (oxfmtrc wins)
  // - Indentation should use 2 spaces, NOT tabs
  it(".oxfmtrc overrides .editorconfig", async () => {
    const cwd = join(fixturesDir, "with_oxfmtrc");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] indent_style=space, indent_size=2
  //   [*.ts] indent_size=4
  //   [nested/**/*.js] indent_size=8
  //   [nested/deep/*.json] indent_style=tab, max_line_length=40
  //
  // Expected:
  // - test.js: tabWidth=2 (from [*], not matched by [nested/**/*.js])
  // - test.ts: tabWidth=4 (from [*.ts])
  // - nested/deep/test.js: tabWidth=8 (from [nested/**/*.js], deep path glob)
  // - nested/deep/test.json: useTab=true, printWidth=40 (from [nested/deep/*.json], deep path glob for external formatter)
  it("per-file overrides", async () => {
    const cwd = join(fixturesDir, "per_file_override");
    const snapshot = await runWriteModeAndSnapshot(cwd, [
      "test.js",
      "test.ts",
      "nested/deep/test.js",
      "nested/deep/test.json",
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*.js] indent_style=tab, indent_size=4
  //   (no [*] section)
  //
  // Expected:
  // - test.js: useTabs=true, tabWidth=4 (from [*.js])
  // - test.ts: default settings (no matching section)
  it("no root [*] section", async () => {
    const cwd = join(fixturesDir, "no_root_section");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.js", "test.ts"]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] indent_style=tab
  // .oxfmtrc.json:
  //   tabWidth=8
  //
  // Expected: useTabs=true (from editorconfig), tabWidth=8 (from oxfmtrc)
  // - Both settings should be merged, not one overwriting the other entirely
  it("partial override - settings from both files merged", async () => {
    const cwd = join(fixturesDir, "partial_override");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig: (empty file)
  //
  // Expected: default settings (useTabs=false, tabWidth=2)
  // - Empty editorconfig should not cause errors
  it("empty .editorconfig", async () => {
    const cwd = join(fixturesDir, "empty");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  // Structure:
  //   nested_cwd/
  //     .editorconfig   # [*] indent_size=2, [sub/*.ts] indent_size=8
  //     sub/            # <- cwd (nested directory)
  //       test.ts       # Pre-formatted with 8-space indentation
  //
  // When running from `sub/` directory:
  // - .editorconfig is found in parent directory
  // - [sub/*.ts] pattern should match `sub/test.ts` relative to .editorconfig location
  // - NOT relative to cwd (sub/) - which would be `sub/sub/*.ts`
  //
  // Expected: --check should pass (exit 0) because file is already formatted with indent_size=8
  // If pattern resolution was wrong, it would use indent_size=2 and fail
  it("nested cwd - patterns resolved relative to .editorconfig location", async () => {
    const nestedCwd = join(fixturesDir, "nested_cwd", "sub");
    const snapshot = await runAndSnapshot(nestedCwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });
});
