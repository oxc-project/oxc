import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("write_mode", () => {
  it("should format unformatted files", async () => {
    const fixtureDir = join(fixturesDir, "write_mode");
    const files = ["unformatted.js", "complex.js"];

    const snapshot = await runWriteModeAndSnapshot(fixtureDir, files);
    expect(snapshot).toMatchSnapshot();
  });

  it("should format unformatted files with explicit --write", async () => {
    const fixtureDir = join(fixturesDir, "write_mode");
    const files = ["unformatted.js", "complex.js"];

    const snapshot = await runWriteModeAndSnapshot(fixtureDir, files, ["--write"]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should not modify already formatted files", async () => {
    const fixtureDir = join(fixturesDir, "write_mode");
    const files = ["already_formatted.js"];

    const snapshot = await runWriteModeAndSnapshot(fixtureDir, files);
    expect(snapshot).toMatchSnapshot();
  });
});
