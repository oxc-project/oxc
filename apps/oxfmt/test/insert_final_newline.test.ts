import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { runWriteModeAndSnapshot } from "./utils";

const fixtureDir = join(import.meta.dirname, "fixtures", "insert_final_newline");

describe("insertFinalNewline option", () => {
  describe("auto mode (default - preserve original)", () => {
    const testDir = join(fixtureDir, "auto");

    it("should preserve EOF newline behavior", async () => {
      // Reset files to original state
      writeFileSync(join(testDir, "without_newline.js"), "const x=1;");
      writeFileSync(join(testDir, "with_newline.js"), "const x=1;\n");
      writeFileSync(join(testDir, "multiple_newlines.js"), "const x=1;\n\n\n");

      const snapshot = await runWriteModeAndSnapshot(testDir, [
        "without_newline.js",
        "with_newline.js",
        "multiple_newlines.js",
      ]);

      // Check the formatted results
      const withoutNewline = readFileSync(join(testDir, "without_newline.js"), "utf8");
      const withNewline = readFileSync(join(testDir, "with_newline.js"), "utf8");
      const multipleNewlines = readFileSync(join(testDir, "multiple_newlines.js"), "utf8");

      expect(withoutNewline).toBe("const x = 1;"); // No newline added
      expect(withNewline).toBe("const x = 1;\n"); // Single newline preserved
      expect(multipleNewlines).toBe("const x = 1;\n"); // Normalized to one

      expect(snapshot).toMatchSnapshot();
    });
  });

  describe("always mode", () => {
    const testDir = join(fixtureDir, "always");

    it("should always ensure EOF newline", async () => {
      // Reset files to original state
      writeFileSync(join(testDir, "without_newline.js"), "const x=1;");
      writeFileSync(join(testDir, "with_newline.js"), "const x=1;\n");
      writeFileSync(join(testDir, "multiple_newlines.js"), "const x=1;\n\n\n");

      const snapshot = await runWriteModeAndSnapshot(testDir, [
        "without_newline.js",
        "with_newline.js",
        "multiple_newlines.js",
      ]);

      // Check the formatted results
      const withoutNewline = readFileSync(join(testDir, "without_newline.js"), "utf8");
      const withNewline = readFileSync(join(testDir, "with_newline.js"), "utf8");
      const multipleNewlines = readFileSync(join(testDir, "multiple_newlines.js"), "utf8");

      expect(withoutNewline).toBe("const x = 1;\n"); // Newline added
      expect(withNewline).toBe("const x = 1;\n"); // Single newline preserved
      expect(multipleNewlines).toBe("const x = 1;\n"); // Normalized to one

      expect(snapshot).toMatchSnapshot();
    });
  });

  describe("never mode", () => {
    const testDir = join(fixtureDir, "never");

    it("should never add EOF newline", async () => {
      // Reset files to original state
      writeFileSync(join(testDir, "without_newline.js"), "const x=1;");
      writeFileSync(join(testDir, "with_newline.js"), "const x=1;\n");
      writeFileSync(join(testDir, "multiple_newlines.js"), "const x=1;\n\n\n");

      const snapshot = await runWriteModeAndSnapshot(testDir, [
        "without_newline.js",
        "with_newline.js",
        "multiple_newlines.js",
      ]);

      // Check the formatted results
      const withoutNewline = readFileSync(join(testDir, "without_newline.js"), "utf8");
      const withNewline = readFileSync(join(testDir, "with_newline.js"), "utf8");
      const multipleNewlines = readFileSync(join(testDir, "multiple_newlines.js"), "utf8");

      expect(withoutNewline).toBe("const x = 1;"); // No newline
      expect(withNewline).toBe("const x = 1;"); // Newline removed
      expect(multipleNewlines).toBe("const x = 1;"); // All newlines removed

      expect(snapshot).toMatchSnapshot();
    });
  });
});
