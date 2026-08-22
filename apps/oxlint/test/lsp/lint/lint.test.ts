import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { DiagnosticSeverity, DiagnosticTag } from "vscode-languageserver-protocol/node";
import { fixFixture, lintFixture, lintFixtureDiagnostics, lintMultiFileFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP linting", () => {
  describe("basic linting", () => {
    it.each([
      ["default/test.tsx", "typescriptreact"],
      ["default/test.ts", "typescript"],
    ])("should handle %s", async (path, languageId) => {
      expect(await lintFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });

  describe("config options", () => {
    it.each([
      ["config-default/test.ts", "typescript"],
      ["config-disabled/test.ts", "typescript"],
      ["config-severity/test.ts", "typescript"],
      ["config-js-plugin/test.js", "javascript"],
      ["config-ts-config/test.js", "javascript"],
      ["config-ts-type-aware/test.ts", "typescript"],
      ["config-ts-nested-type-aware-invalid/nested/test.ts", "typescript"],
      ["unused-disable-directive-from-config/test.ts", "typescript"],
      ["vite-config-skip-finds-parent/child/test.js", "javascript"],
      ["config-ts-stdout-pollution/test.js", "javascript"],
    ])("should apply config from %s", async (path, languageId) => {
      expect(await lintFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });

    it("should allow LSP typeAware option to override ts config", async () => {
      expect(
        await lintFixture(
          FIXTURES_DIR,
          "config-ts-type-aware/test-with-lsp-config.ts",
          "typescript",
          {
            typeAware: false,
          },
        ),
      ).toMatchSnapshot();
    });
  });

  describe("nested config", () => {
    it("should apply nested config", async () => {
      expect(
        await lintMultiFileFixture(FIXTURES_DIR, [
          { path: "nested-config/test.js", languageId: "javascript" },
          { path: "nested-config/nested/test.js", languageId: "javascript" },
        ]),
      ).toMatchSnapshot();
    });
  });

  describe("initializationOptions", () => {
    it("should use custom config path from configPath", async () => {
      expect(
        await lintFixture(FIXTURES_DIR, "custom-config-path/test.ts", "typescript", {
          configPath: "./lint.json",
        }),
      ).toMatchSnapshot();
    });
  });

  describe("bulk suppressions", () => {
    it("shows suppressed diagnostics faded by default", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions",
        "default.js",
        "javascript",
      );
      const suppressed = diagnostics.filter(({ code }) => code === "eslint(no-console)");
      const surfaced = diagnostics.filter(({ code }) => code === "eslint(no-debugger)");

      expect(suppressed).toHaveLength(2);
      expect(surfaced).toHaveLength(1);
      for (const diagnostic of suppressed) {
        expect(diagnostic.tags).toEqual([DiagnosticTag.Unnecessary]);
        expect(diagnostic.severity).toBe(DiagnosticSeverity.Warning);
      }
      expect(surfaced[0]?.tags).toBeUndefined();
    });

    it("hides suppressed diagnostics when configured", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions",
        "hidden.js",
        "javascript",
        { showSuppressedViolations: false },
      );

      expect(diagnostics).toHaveLength(1);
      expect(diagnostics[0]?.code).toBe("eslint(no-debugger)");
    });

    it("overrides the severity of suppressed diagnostics", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions",
        "severity.js",
        "javascript",
        { suppressedViolationSeverity: "information" },
      );
      const suppressed = diagnostics.filter(({ code }) => code === "eslint(no-console)");

      expect(suppressed).toHaveLength(2);
      for (const diagnostic of suppressed) {
        expect(diagnostic.tags).toEqual([DiagnosticTag.Unnecessary]);
        expect(diagnostic.severity).toBe(DiagnosticSeverity.Information);
      }
    });

    it("surfaces a rule when its violation count increases", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions",
        "increased.js",
        "javascript",
      );
      const noConsole = diagnostics.filter(({ code }) => code === "eslint(no-console)");

      expect(noConsole).toHaveLength(3);
      for (const diagnostic of noConsole) {
        expect(diagnostic.tags).toBeUndefined();
      }
    });

    it("ignores a suppression baseline below the workspace root", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions-nested",
        "web/test.js",
        "javascript",
      );
      const suppressed = diagnostics.filter(({ code }) => code === "eslint(no-console)");

      expect(suppressed).toHaveLength(2);
      for (const diagnostic of suppressed) {
        expect(diagnostic.tags).toBeUndefined();
      }
    });

    it("surfaces diagnostics when the suppression baseline is malformed", async () => {
      const diagnostics = await lintFixtureDiagnostics(
        FIXTURES_DIR,
        "suppressions-malformed",
        "test.js",
        "javascript",
      );
      const noConsole = diagnostics.filter(({ code }) => code === "eslint(no-console)");

      expect(noConsole).toHaveLength(2);
      for (const diagnostic of noConsole) {
        expect(diagnostic.tags).toBeUndefined();
        expect(diagnostic.severity).toBe(DiagnosticSeverity.Error);
      }
    });

    it("keeps code actions for suppressed diagnostics", async () => {
      const codeActions = await fixFixture(
        FIXTURES_DIR,
        "suppressions/fix.js",
        "javascript",
      );

      expect(codeActions).toContain("Title : Remove the debugger statement");
    });
  });
});
