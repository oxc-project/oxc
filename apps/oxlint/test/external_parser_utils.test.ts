import { describe, expect, it } from "vitest";
import {
  detectExternalSourceFlags,
  normalizeExternalProgramSourceType,
} from "../src-js/plugins/external_parser_utils.ts";

describe("external parser utils", () => {
  describe("detectExternalSourceFlags", () => {
    it("infers JSX and TypeScript flags from external AST nodes when config hints are missing", () => {
      const ast = {
        type: "Program",
        body: [
          {
            type: "SvelteElement",
            children: [
              { type: "JSXElement" },
              { type: "TSInterfaceDeclaration" },
            ],
          },
        ],
      };

      expect(detectExternalSourceFlags(null, ast)).toEqual({ isJsx: true, isTs: true });
    });

    it("keeps explicit parser options over AST fallback", () => {
      const ast = {
        type: "Program",
        body: [
          {
            type: "SvelteElement",
            children: [
              { type: "JSXElement" },
              { type: "TSInterfaceDeclaration" },
            ],
          },
        ],
      };

      expect(
        detectExternalSourceFlags(
          {
            lang: "ts",
            ecmaFeatures: { jsx: false },
          } as Record<string, unknown>,
          ast,
        ),
      ).toEqual({ isJsx: false, isTs: true });
    });

    it("lets parserOptions.lang override conflicting ecmaFeatures.jsx hints", () => {
      const ast = {
        type: "Program",
        body: [
          {
            type: "SvelteElement",
            children: [
              { type: "JSXElement" },
              { type: "TSInterfaceDeclaration" },
            ],
          },
        ],
      };

      expect(
        detectExternalSourceFlags(
          {
            lang: "tsx",
            ecmaFeatures: { jsx: false },
          } as Record<string, unknown>,
          ast,
        ),
      ).toEqual({ isJsx: true, isTs: true });

      expect(
        detectExternalSourceFlags(
          {
            lang: "js",
            ecmaFeatures: { jsx: true },
          } as Record<string, unknown>,
          ast,
        ),
      ).toEqual({ isJsx: false, isTs: false });
    });



    it("respects explicit empty visitor keys while scanning external syntax flags", () => {
      const ast = {
        type: "Program",
        hiddenSyntax: {
          type: "SvelteElement",
          children: [
            { type: "JSXElement" },
            { type: "TSInterfaceDeclaration" },
          ],
        },
      };

      expect(
        detectExternalSourceFlags(null, ast, {
          Program: ["hiddenSyntax"],
          SvelteElement: [],
        }),
      ).toEqual({ isJsx: false, isTs: false });
    });
    it("ignores cyclic parser metadata while scanning external syntax flags", () => {
      const ast = {
        type: "Program",
        body: [
          {
            type: "SvelteElement",
            children: [{ type: "TSInterfaceDeclaration" }],
          },
        ],
      } as Record<string, unknown> & { metadata?: Record<string, unknown> };
      const metadata: Record<string, unknown> = { owner: ast };
      metadata.self = metadata;
      ast.metadata = metadata;

      expect(() => detectExternalSourceFlags(null, ast)).not.toThrow();
      expect(detectExternalSourceFlags(null, ast)).toEqual({ isJsx: false, isTs: true });
    });
  });

  describe("normalizeExternalProgramSourceType", () => {
    it("prefers explicit parser call options over Program.sourceType defaults", () => {
      expect(normalizeExternalProgramSourceType("module", "script", [])).toBe("script");
      expect(normalizeExternalProgramSourceType("module", "commonjs", [])).toBe("commonjs");
    });

    it("falls back to parser call options when the parser omits Program.sourceType", () => {
      expect(normalizeExternalProgramSourceType(undefined, "script", [])).toBe("script");
      expect(normalizeExternalProgramSourceType(undefined, "commonjs", [])).toBe("commonjs");
    });

    it("defaults missing source type to module unless the parser requested unambiguous mode", () => {
      expect(normalizeExternalProgramSourceType(undefined, undefined, [])).toBe("module");
      expect(normalizeExternalProgramSourceType(undefined, "unambiguous", [])).toBe("script");
      expect(
        normalizeExternalProgramSourceType(undefined, "unambiguous", [
          { type: "ExportNamedDeclaration" },
        ]),
      ).toBe("module");
    });
  });
});
