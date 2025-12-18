import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures", "editorconfig");

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
  //   [nested/**/*.json] indent_size=8, max_line_length=40
  //
  // Expected:
  // - test.js: tabWidth=2 (from [*], not matched by [nested/**/*.js])
  // - test.ts: tabWidth=4 (from [*.ts])
  // - nested/deep/test.js: tabWidth=8 (from [nested/**/*.js], deep path glob)
  // - nested/deep/test.json: tabWidth=8, printWidth=40 (from [nested/**/*.json], deep path glob for external formatter)
  // TODO: Should fix `editor_config_parser` to accept `cwd` for `resolve()`
  // oxlint-disable-next-line vitest/no-disabled-tests
  it.skip("per-file overrides", async () => {
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
});
