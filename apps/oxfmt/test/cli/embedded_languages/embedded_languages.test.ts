import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("embedded_languages", () => {
  it("should format embedded languages by default", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["embedded_languages.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should format embedded languages when embeddedLanguageFormatting is auto", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["embedded_languages.js"],
      ["--config", "on_embedded.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("should not format embedded languages when embeddedLanguageFormatting is off", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["embedded_languages.js"],
      ["--config", "off_embedded.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
