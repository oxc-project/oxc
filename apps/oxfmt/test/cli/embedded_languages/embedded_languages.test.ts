import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");
const languages = ["css.js", "graphql.js", "html.js", "markdown.js", "angular.ts"];

describe("embedded_languages", () => {
  it.each(languages)(`should format %s (auto)`, async (lang) => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, [lang]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should not format any language (off)", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, languages, [
      "--config",
      "off_embedded.json",
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should format with CLRF", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, languages, [
      "--config",
      "crlf_embedded.json",
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  describe("Misc", () => {
    it("should format multiple embedded languages in one file", async () => {
      const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["mixed.js"]);
      expect(snapshot).toMatchSnapshot();
    });

    it("should not format regular templates, prettier-ignore, unsupported tags, and invalid syntax", async () => {
      const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["misc.js"]);
      expect(snapshot).toMatchSnapshot();
    });
  });
});
