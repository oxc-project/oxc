import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("insertFinalNewline", () => {
  // .oxfmtrc.json:
  //   insertFinalNewline=false
  //
  // Expected: No trailing newline in formatted output
  // - test.ts: TypeScript file (oxc_formatter)
  // - test.css: CSS file (external formatter)
  // - test.toml: TOML file (toml formatter)
  it("oxfmtrc setting removes final newline", async () => {
    const cwd = join(fixturesDir, "oxfmtrc_only");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.ts", "test.css", "test.toml"]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] insert_final_newline=false
  //
  // Expected: No trailing newline in formatted output
  // - test.ts: TypeScript file (oxc_formatter)
  // - test.css: CSS file (external formatter)
  // - test.toml: TOML file (toml formatter)
  it("editorconfig setting removes final newline", async () => {
    const cwd = join(fixturesDir, "editorconfig_only");
    const snapshot = await runWriteModeAndSnapshot(cwd, ["test.ts", "test.css", "test.toml"]);
    expect(snapshot).toMatchSnapshot();
  });
});
