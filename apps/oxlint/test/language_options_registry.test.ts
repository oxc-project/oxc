import { describe, expect, it } from "vitest";
import {
  registerLanguageOptions,
  resolveLanguageOptionsIds,
} from "../src-js/js_language_options_registry.ts";

describe("language options registry", () => {
  it("deep merges parser options and preserves non-serializable values", () => {
    const preprocess = () => "preprocessed";
    const nestedParser = {
      parseForESLint() {
        return { ast: null };
      },
    };
    const topLevelParser = {
      parseForESLint() {
        return { ast: null };
      },
    };

    const baseId = registerLanguageOptions({
      parserOptions: {
        parser: nestedParser,
        svelteConfig: {
          compilerOptions: {
            runes: false,
            generate: "dom",
          },
          preprocess,
        },
        tsFlavor: "base-ts-parser",
        nested: {
          one: 1,
          two: "base",
        },
      },
    });
    const overrideId = registerLanguageOptions({
      parser: topLevelParser,
      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"],
        svelteConfig: {
          compilerOptions: {
            runes: true,
          },
        },
        nested: {
          two: "override",
        },
      },
    });

    const resolved = resolveLanguageOptionsIds([baseId, overrideId]);
    expect(resolved?.parser).toBe(topLevelParser);

    const parserOptions = resolved?.parserOptions as Record<string, unknown> | undefined;
    expect(parserOptions).toBeDefined();
    expect(parserOptions?.projectService).toBe(true);
    expect(parserOptions?.extraFileExtensions).toEqual([".svelte"]);
    expect(parserOptions?.parser).toBe(nestedParser);
    expect(parserOptions?.tsFlavor).toBe("base-ts-parser");
    expect(parserOptions?.nested).toEqual({ one: 1, two: "override" });

    const svelteConfig = parserOptions?.svelteConfig as
      | {
          compilerOptions?: { runes?: unknown; generate?: unknown };
          preprocess?: unknown;
        }
      | undefined;
    expect(svelteConfig?.preprocess).toBe(preprocess);
    expect(svelteConfig?.compilerOptions).toEqual({
      runes: true,
      generate: "dom",
    });
  });


  it("replaces parser objects instead of recursively merging them", () => {
    const baseParser = {
      name: "base-parser",
      parseForESLint() {
        return { ast: null };
      },
      latestEcmaVersion: 2024,
    };
    const overrideParser = {
      name: "override-parser",
      parseForESLint() {
        return { ast: null };
      },
      supportedEcmaVersions: [2025],
    };

    const baseId = registerLanguageOptions({
      parserOptions: {
        parser: baseParser,
      },
    });
    const overrideId = registerLanguageOptions({
      parserOptions: {
        parser: overrideParser,
      },
    });

    const resolved = resolveLanguageOptionsIds([baseId, overrideId]);
    const parserOptions = resolved?.parserOptions as { parser?: unknown } | undefined;
    expect(parserOptions?.parser).toBe(overrideParser);
    expect(parserOptions?.parser).not.toBe(baseParser);
  });

  it("shallow merges parser maps without merging parser objects inside them", () => {
    const baseTsParser = {
      name: "base-ts-parser",
      parseForESLint() {
        return { ast: null };
      },
    };
    const baseJsParser = {
      name: "base-js-parser",
      parseForESLint() {
        return { ast: null };
      },
    };
    const overrideTsParser = {
      name: "override-ts-parser",
      parseForESLint() {
        return { ast: null };
      },
    };
    const overrideTsxParser = {
      name: "override-tsx-parser",
      parseForESLint() {
        return { ast: null };
      },
    };

    const baseId = registerLanguageOptions({
      parserOptions: {
        parser: {
          js: baseJsParser,
          ts: baseTsParser,
        },
      },
    });
    const overrideId = registerLanguageOptions({
      parserOptions: {
        parser: {
          ts: overrideTsParser,
          tsx: overrideTsxParser,
        },
      },
    });

    const resolved = resolveLanguageOptionsIds([baseId, overrideId]);
    const parserMap = (resolved?.parserOptions as { parser?: Record<string, unknown> } | undefined)
      ?.parser;

    expect(parserMap).toEqual({
      js: baseJsParser,
      ts: overrideTsParser,
      tsx: overrideTsxParser,
    });
    expect(parserMap?.js).toBe(baseJsParser);
    expect(parserMap?.ts).toBe(overrideTsParser);
    expect(parserMap?.tsx).toBe(overrideTsxParser);
  });
});
