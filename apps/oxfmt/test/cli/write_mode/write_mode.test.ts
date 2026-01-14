import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("write_mode", () => {
  it("should format unformatted files", async () => {
    const files = ["unformatted.js", "complex.js"];

    const snapshot = await runWriteModeAndSnapshot(fixturesDir, files);
    expect(snapshot).toMatchSnapshot();
  });

  it("should format unformatted files with explicit --write", async () => {
    const files = ["unformatted.js", "complex.js"];

    const snapshot = await runWriteModeAndSnapshot(fixturesDir, files, ["--write"]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should not modify already formatted files", async () => {
    const files = ["already_formatted.js"];

    const snapshot = await runWriteModeAndSnapshot(fixturesDir, files);
    expect(snapshot).toMatchSnapshot();
  });
});
