import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const projectRoot = join(import.meta.dirname, "fixtures", "project-root");

describe("basePath", () => {
  // Structure:
  //   project-root/
  //     foo/bar/.oxfmtrc.json   # basePath: "../..", ignorePatterns: ["dist/**", "foo/**"], overrides for src/**/*.js
  //     foo/bar/baz/            # empty subdirectory for auto-discovery test
  //     dist/bundle.js          # broken JS, should be ignored by ignorePatterns
  //     src/app.js              # formatted with tabWidth=4 (overrides match)
  //     lib/utils.ts            # formatted with tabWidth=2 (default)

  it("explicit --config from project root", async () => {
    const snapshot = await runAndSnapshot(projectRoot, [
      ["--check", "--config", "foo/bar/.oxfmtrc.json", "."],
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  it("explicit --config from subdirectory", async () => {
    const cwd = join(projectRoot, "src");
    const snapshot = await runAndSnapshot(cwd, [
      ["--check", "--config", "../foo/bar/.oxfmtrc.json", projectRoot],
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  it("auto-discovery from config directory", async () => {
    const cwd = join(projectRoot, "foo", "bar");
    const snapshot = await runAndSnapshot(cwd, [["--check", projectRoot]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("auto-discovery from subdirectory below config", async () => {
    const cwd = join(projectRoot, "foo", "bar", "baz");
    const snapshot = await runAndSnapshot(cwd, [["--check", projectRoot]]);
    expect(snapshot).toMatchSnapshot();
  });
});
