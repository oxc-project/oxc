import { describe, expect, it } from "vitest";
import { createRequiredParserCallOptions } from "../src-js/plugins/parser_call_options.ts";

describe("createRequiredParserCallOptions", () => {
  it("merges top-level languageOptions.sourceType when parserOptions omit it", () => {
    expect(createRequiredParserCallOptions("/test/App.svelte", null, "script")).toMatchObject({
      sourceType: "script",
      filePath: "/test/App.svelte",
      loc: true,
      range: true,
      raw: true,
      tokens: true,
      comment: true,
      eslintVisitorKeys: true,
      eslintScopeManager: true,
    });
  });

  it("keeps explicit parserOptions.sourceType over the top-level sourceType", () => {
    expect(
      createRequiredParserCallOptions(
        "/test/App.svelte",
        { sourceType: "commonjs" },
        "module",
      ),
    ).toMatchObject({
      sourceType: "commonjs",
    });
  });

  it("merges top-level languageOptions.ecmaVersion when parserOptions omit it", () => {
    expect(
      createRequiredParserCallOptions("/test/App.svelte", null, undefined, 2022),
    ).toMatchObject({
      ecmaVersion: 2022,
      filePath: "/test/App.svelte",
    });
  });

  it("infers tsconfigRootDir from cwd for type-aware parser options", () => {
    expect(
      createRequiredParserCallOptions(
        "/workspace/packages/app/src/App.svelte",
        { projectService: true, extraFileExtensions: [".svelte"] },
        undefined,
        undefined,
        "/workspace/packages/app",
      ),
    ).toMatchObject({
      projectService: true,
      extraFileExtensions: [".svelte"],
      tsconfigRootDir: "/workspace/packages/app",
    });
  });

  it("falls back to the file directory when inferring tsconfigRootDir without cwd", () => {
    expect(
      createRequiredParserCallOptions(
        "/workspace/packages/app/src/App.svelte",
        { project: true },
      ),
    ).toMatchObject({
      project: true,
      tsconfigRootDir: "/workspace/packages/app/src",
    });
  });

  it("keeps explicit parserOptions.ecmaVersion over the top-level ecmaVersion", () => {
    expect(
      createRequiredParserCallOptions(
        "/test/App.svelte",
        { ecmaVersion: 2020 },
        undefined,
        2024,
      ),
    ).toMatchObject({
      ecmaVersion: 2020,
    });
  });

  it("deep-clones nested parser options while preserving parser objects and functions", () => {
    const tsParser = {
      parseForESLint() {
        return { ast: null };
      },
    };
    const jsParser = {
      parse() {
        return null;
      },
    };
    const preprocess = () => "preprocessed";
    const parserOptions = {
      parser: {
        ts: tsParser,
        js: jsParser,
      },
      projectService: true,
      extraFileExtensions: [".svelte"],
      svelteConfig: {
        compilerOptions: {
          runes: true,
        },
        preprocess,
      },
    };

    const options = createRequiredParserCallOptions(
      "/workspace/packages/app/src/App.svelte",
      parserOptions,
      undefined,
      undefined,
      "/workspace/packages/app",
    );

    expect(options).not.toBe(parserOptions);
    expect(options.parser).not.toBe(parserOptions.parser);
    expect((options.parser as { ts: unknown }).ts).toBe(tsParser);
    expect((options.parser as { js: unknown }).js).toBe(jsParser);
    expect(options.extraFileExtensions).not.toBe(parserOptions.extraFileExtensions);
    expect(options.extraFileExtensions).toEqual([".svelte"]);
    expect(options.svelteConfig).not.toBe(parserOptions.svelteConfig);
    expect(
      (options.svelteConfig as { compilerOptions: unknown }).compilerOptions,
    ).not.toBe(parserOptions.svelteConfig.compilerOptions);
    expect(
      (options.svelteConfig as { preprocess: unknown }).preprocess,
    ).toBe(preprocess);

    (options.extraFileExtensions as string[]).push(".astro");
    (options.svelteConfig as { compilerOptions: { runes: boolean } }).compilerOptions.runes = false;

    expect(parserOptions.extraFileExtensions).toEqual([".svelte"]);
    expect(parserOptions.svelteConfig.compilerOptions.runes).toBe(true);
    expect(options.tsconfigRootDir).toBe("/workspace/packages/app");
  });
});
