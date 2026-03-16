import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

const VP_ENV = { VITE_PLUS_VERSION: "1" };

describe("vite_config", () => {
  it("basic: reads fmt field from vite.config.ts", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("ignored: vite.config.ts is not used without VITE_PLUS_VERSION", async () => {
    // Same fixture as "basic" but without VITE_PLUS_VERSION env
    // vite.config.ts has semi: false, but should be ignored → defaults (semi: true) → check passes
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("error: explicit --config vite.config.ts without fmt field", async () => {
    const cwd = join(fixturesDir, "no_fmt_field");
    const snapshot = await runAndSnapshot(
      cwd,
      [["--check", "--config", "vite.config.ts", "test.ts"]],
      VP_ENV,
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("skip: auto-discovered vite.config.ts without fmt field uses defaults", async () => {
    const cwd = join(fixturesDir, "no_fmt_field");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("error: explicit --config vite.config.ts that fails to load", async () => {
    const cwd = join(fixturesDir, "error_load_failure", "child");
    const snapshot = await runAndSnapshot(
      cwd,
      [["--check", "--config", "vite.config.ts", "test.ts"]],
      VP_ENV,
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("error: auto-discovered vite.config.ts that fails to load", async () => {
    const cwd = join(fixturesDir, "error_load_failure", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("ignored: not auto-discovered vite.config.ts that fails to load", async () => {
    // Without VITE_PLUS_VERSION, broken vite.config.ts is completely ignored
    // Parent .oxfmtrc.json (semi: false) is found instead → `const a = 1` (no semi) matches → check passes
    const cwd = join(fixturesDir, "error_load_failure", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("vp: parent oxfmt config is not used when vite.config.ts without fmt is skipped", async () => {
    // child/ has vite.config.ts without .fmt → skipped
    // parent has .oxfmtrc.json with semi: false, but it's ignored in VP mode
    // So defaults (semi: true) apply → `const a = 1;` check passes
    const cwd = join(fixturesDir, "skip_finds_parent", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("skip: auto-discovered vite.config.ts with function export uses defaults", async () => {
    const cwd = join(fixturesDir, "skip_fn_export");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("vp: oxfmt.config.ts is ignored, vite.config.ts is used", async () => {
    // `vite.config.ts` has `semi: true`, `oxfmt.config.ts` has `semi: false`
    // In VP mode, oxfmt.config.ts is ignored and vite.config.ts is used
    // So `const a = 1;` (with semicolon) matches semi: true → check passes
    const cwd = join(fixturesDir, "priority");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("oxfmt: vite.config.ts is ignored, oxfmt.config.ts is used", async () => {
    // `vite.config.ts` has `semi: true`, `oxfmt.config.ts` has `semi: false`
    // In non-VP mode, vite.config.ts is ignored and oxfmt.config.ts is used
    // So `const a = 1;` (with semicolon) does not match semi: false → check fails
    const cwd = join(fixturesDir, "priority");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });
});
