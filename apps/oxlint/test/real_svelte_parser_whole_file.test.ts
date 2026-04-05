import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFile, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";
import { resolveRealSveltePackageProfileName } from "../scripts/svelte-real-package-metadata.ts";
import {
  tryLoadRealSvelteParser,
  tryLoadRealSvelteTypeAwarePackages,
} from "./real_svelte_runtime_packages.ts";

const realSvelteParser = await tryLoadRealSvelteParser();
const realSvelteTypeAwarePackages = await tryLoadRealSvelteTypeAwarePackages();
const realSveltePackageProfileName = resolveRealSveltePackageProfileName(undefined);
const allowMissingRealSvelteCommentRoundTrip = realSveltePackageProfileName === "latest-svelte";
const REQUIRE_REAL_SVELTE_PACKAGES = process.env.OXLINT_SVELTE_REAL_PACKAGES_CI === "1";

function loadRealSvelteParser() {
  if (realSvelteParser === null) {
    throw new Error(
      "Real Svelte parser tests require loadable `svelte-eslint-parser` and `svelte` packages.",
    );
  }

  return realSvelteParser;
}

function loadRealSvelteTypeAwarePackages() {
  if (realSvelteTypeAwarePackages === null) {
    throw new Error(
      "Real Svelte type-aware tests require loadable `svelte-eslint-parser`, `@typescript-eslint/parser`, and `svelte` packages.",
    );
  }

  return realSvelteTypeAwarePackages;
}

function setSingleRuleOptions() {
  setOptions(
    JSON.stringify({
      options: [[]],
      ruleIds: [0],
      cwd: "/workspace",
      workspaceUri: null,
    }),
  );
}

const realParserIt = realSvelteParser === null && !REQUIRE_REAL_SVELTE_PACKAGES ? it.skip : it;
const realTypeAwareIt =
  realSvelteTypeAwarePackages === null && !REQUIRE_REAL_SVELTE_PACKAGES ? it.skip : it;

describe("real svelte-eslint-parser whole-file integration", () => {
  afterEach(() => {
    registeredRules.length = 0;
    diagnostics.length = 0;
    if (allOptions !== null) allOptions.length = 1;
    resetStateAfterError();
  });

  realParserIt("traverses real Svelte template nodes through the whole-file parser lane", async () => {
    const parser = loadRealSvelteParser();
    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-real-svelte-element": {
            create(context) {
              return {
                SvelteElement(node) {
                  const services = context.sourceCode.parserServices;
                  context.report({
                    node,
                    message: [
                      `isSvelte:${services.isSvelte === true}`,
                      `hasStyleContext:${typeof services.getStyleContext === "function"}`,
                    ].join("; "),
                  });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      "<h1>Hello</h1>",
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0].message).toBe("isSvelte:true; hasStyleContext:true");
  });

  realParserIt("surfaces malformed template expressions from the real parser as parse errors", async () => {
    const parser = loadRealSvelteParser();
    const sourceText = `{#if page.data.user && }\n{/if}`;
    const templateLineStart = sourceText.indexOf("{#if");
    const templateLineEnd = sourceText.indexOf("\n");
    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "never-runs": {
            create() {
              return {
                Program() {
                  throw new Error("rule should not run when the parser fails");
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      sourceText,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.diagnostics).toEqual([]);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success.parseError).toMatchObject({
      start: expect.any(Number),
      end: expect.any(Number),
    });
    expect(typeof payload.Success.parseError.message).toBe("string");
    expect(payload.Success.parseError.message.length).toBeGreaterThan(0);
    expect(payload.Success.parseError.end - payload.Success.parseError.start).toBe(1);
    expect(payload.Success.parseError.start).toBeGreaterThanOrEqual(templateLineStart);
    expect(payload.Success.parseError.end).toBeLessThanOrEqual(templateLineEnd);
  });

  realTypeAwareIt("preserves real nested TypeScript parser services and traverses mixed script layouts", async () => {
    const { parser, tsParser } = loadRealSvelteTypeAwarePackages();
    const languageOptionsId = registerLanguageOptions({
      parser,
      parserOptions: {
        parser: tsParser,
        extraFileExtensions: [".svelte"],
        svelteConfig: {
          compilerOptions: {
            runes: true,
          },
          preprocess() {
            return "preprocessed";
          },
        },
      },
    });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-real-svelte-type-aware": {
            create(context) {
              let scriptCount = 0;
              let styleCount = 0;
              let tsNodeCount = 0;
              let firstElementName = "none";

              return {
                SvelteScriptElement() {
                  scriptCount += 1;
                },
                SvelteStyleElement() {
                  styleCount += 1;
                },
                TSInterfaceDeclaration() {
                  tsNodeCount += 1;
                },
                SvelteElement(node) {
                  if (firstElementName !== "none") return;
                  const name = (node as { name?: { name?: string } }).name;
                  if (typeof name?.name === "string") {
                    firstElementName = name.name;
                  }
                },
                "Program:exit"(node) {
                  const parserServices = context.sourceCode.parserServices as {
                    isSvelte?: unknown;
                    isSvelteScript?: unknown;
                    svelteParseContext?: { runes?: unknown };
                    esTreeNodeToTSNodeMap?: { get?: unknown };
                    tsNodeToESTreeNodeMap?: { get?: unknown };
                  };
                  const parserOptions = context.languageOptions.parserOptions as {
                    parser?: { parseForESLint?: unknown };
                    extraFileExtensions?: unknown;
                    svelteConfig?: { compilerOptions?: { runes?: unknown } };
                  };
                  const hasTsMaps =
                    typeof parserServices.esTreeNodeToTSNodeMap?.get === "function" &&
                    typeof parserServices.tsNodeToESTreeNodeMap?.get === "function";

                  context.report({
                    node,
                    message: [
                      `parser:${context.languageOptions.parser?.name === "svelte-eslint-parser"}`,
                      `services:${parserServices.isSvelte === true && parserServices.isSvelteScript === false}`,
                      `nestedParser:${typeof parserOptions.parser?.parseForESLint === "function"}`,
                      `extraFileExtensions:${Array.isArray(parserOptions.extraFileExtensions) && parserOptions.extraFileExtensions.includes(".svelte")}`,
                      `runes:${parserServices.svelteParseContext?.runes === true && parserOptions.svelteConfig?.compilerOptions?.runes === true}`,
                      `tsMaps:${hasTsMaps}`,
                      `scripts:${scriptCount}`,
                      `styles:${styleCount}`,
                      `tsNodes:${tsNodeCount}`,
                      `element:${firstElementName}`,
                    ].join("; "),
                  });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      `<script context="module" lang="ts">\n  export const prerender = true;\n</script>\n<script lang="ts">\n  interface User {\n    name: string;\n  }\n\n  export let user: User = { name: "world" };\n</script>\n<style>h1 { color: red; }</style>\n<h1>Hello {user.name}</h1>`,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0].message).toBe(
      "parser:true; services:true; nestedParser:true; extraFileExtensions:true; runes:true; tsMaps:true; scripts:2; styles:1; tsNodes:1; element:h1",
    );
  });

  realParserIt("exposes real comments/tokens APIs and round-trips HTML directive comment spans", async () => {
    const parser = await loadRealSvelteParser();
    const languageOptionsId = registerLanguageOptions({ parser });
    const sourceText = `<!-- eslint-disable-next-line test-plugin/report-real-svelte-comments -->
<h1>Hello</h1>`;
    const commentEnd = sourceText.indexOf("-->") + 3;

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-real-svelte-comments": {
            create(context) {
              return {
                SvelteElement(node) {
                  const allComments = context.sourceCode.getAllComments() as Array<{
                    type: string;
                    value: string;
                    start: number;
                    end: number;
                  }>;
                  const commentsBefore = context.sourceCode.getCommentsBefore(node) as Array<{
                    type: string;
                    value: string;
                    start: number;
                    end: number;
                  }>;
                  const tokens = context.sourceCode.getTokens(node) as Array<{
                    range: [number, number];
                  }>;
                  const merged = context.sourceCode.tokensAndComments as Array<{
                    type?: string;
                    start?: number;
                    end?: number;
                  }>;
                  const firstToken = context.sourceCode.getFirstToken(node) as
                    | { range: [number, number] }
                    | null;
                  const lastToken = context.sourceCode.getLastToken(node) as
                    | { range: [number, number] }
                    | null;
                  const astTokens = context.sourceCode.ast.tokens as Array<{
                    range: [number, number];
                  }>;

                  context.report({
                    node,
                    message: [
                      `commentSpan:${allComments.length === 1 && allComments[0].start === 0 && allComments[0].end === commentEnd}`,
                      `commentValue:${allComments[0]?.value.includes("eslint-disable-next-line") === true}`,
                      `beforeHasComment:${commentsBefore.length === 1 && commentsBefore[0] === allComments[0]}`,
                      `hasTokens:${tokens.length > 0}`,
                      `mergedStartsWithComment:${merged[0] === allComments[0]}`,
                      `firstInsideNode:${firstToken !== null && firstToken.range[0] >= node.range[0] && firstToken.range[1] <= node.range[1]}`,
                      `lastInsideNode:${lastToken !== null && lastToken.range[0] >= node.range[0] && lastToken.range[1] <= node.range[1]}`,
                      `astComments:${context.sourceCode.ast.comments === allComments}`,
                      `astTokens:${Array.isArray(astTokens) && astTokens.length > 0 && astTokens.some((token) => token === firstToken)}`,
                    ].join("; "),
                  });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      sourceText,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    const hasCommentRoundTrip = payload.Success.comments.length === 1;
    if (allowMissingRealSvelteCommentRoundTrip) {
      expect(payload.Success.comments).toEqual(
        hasCommentRoundTrip ? [expect.objectContaining({ start: 0, end: commentEnd })] : [],
      );
    } else {
      expect(payload.Success.comments).toEqual([
        expect.objectContaining({ start: 0, end: commentEnd }),
      ]);
    }
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0].message).toBe(
      hasCommentRoundTrip
        ? "commentSpan:true; commentValue:true; beforeHasComment:true; hasTokens:true; mergedStartsWithComment:true; firstInsideNode:true; lastInsideNode:true; astComments:true; astTokens:true"
        : "commentSpan:false; commentValue:false; beforeHasComment:false; hasTokens:true; mergedStartsWithComment:false; firstInsideNode:true; lastInsideNode:true; astComments:true; astTokens:true",
    );
  });

  realParserIt("uses real parser-provided scope helpers on Svelte files", async () => {
    const parser = await loadRealSvelteParser();
    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-real-svelte-scope": {
            create(context) {
              let localDeclaration: unknown = null;
              let globalIdentifier: unknown = null;

              return {
                VariableDeclaration(node) {
                  const declaration = (
                    node as { declarations?: Array<{ id?: { type?: string; name?: string } }> }
                  ).declarations?.[0];

                  if (
                    localDeclaration === null &&
                    declaration?.id?.type === "Identifier" &&
                    declaration.id.name === "local"
                  ) {
                    localDeclaration = node;
                  }
                },
                Identifier(node) {
                  if (
                    globalIdentifier === null &&
                    (node as { name?: string }).name === "window"
                  ) {
                    globalIdentifier = node;
                  }
                },
                "Program:exit"(node) {
                  const scope = context.sourceCode.getScope(node);
                  const declared =
                    localDeclaration === null
                      ? []
                      : context.sourceCode.getDeclaredVariables(localDeclaration as any);
                  const hasLocal = declared.some((variable) => variable.name === "local");
                  const globalRef =
                    globalIdentifier !== null &&
                    context.sourceCode.isGlobalReference(globalIdentifier as any);

                  context.report({
                    node,
                    message: [
                      `hasScope:${scope != null && typeof scope.type === "string"}`,
                      `hasLocal:${hasLocal}`,
                      `globalRef:${globalRef}`,
                      `markUsed:${context.sourceCode.markVariableAsUsed("local", node)}`,
                      `hasGlobalScope:${context.sourceCode.scopeManager.globalScope != null}`,
                    ].join("; "),
                  });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{"window":"readonly"},"envs":{}}',
      [languageOptionsId],
      null,
      `<script>
  const local = 1;
  console.log(local);
  console.log(window);
</script>
<h1>{local}</h1>`,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0].message).toBe(
      "hasScope:true; hasLocal:true; globalRef:true; markUsed:true; hasGlobalScope:true",
    );
  });


  realParserIt("passes required parser call flags and top-level options to the real parser", async () => {
    const realParser = await loadRealSvelteParser();
    let receivedOptions: Record<string, unknown> | undefined;

    const parser = {
      name: realParser.name,
      parseForESLint(code: string, options?: Record<string, unknown>) {
        receivedOptions = options;
        return realParser.parseForESLint(code, options);
      },
    };

    const languageOptionsId = registerLanguageOptions({
      parser,
      sourceType: "script",
      ecmaVersion: 2022,
      parserOptions: {
        comment: false,
        tokens: false,
        loc: false,
        range: false,
        raw: false,
        eslintVisitorKeys: false,
        eslintScopeManager: false,
      },
    });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-real-svelte-parser-contract": {
            create(context) {
              return {
                Program(node) {
                  const comments = context.sourceCode.getAllComments();
                  const tokens = context.sourceCode.getTokens(node);
                  const parserServices = context.sourceCode.parserServices as {
                    isSvelte?: unknown;
                  };

                  context.report({
                    node,
                    message: [
                      `sourceType:${context.languageOptions.sourceType === "script"}`,
                      `ecmaVersion:${context.languageOptions.ecmaVersion === 2022}`,
                      `services:${parserServices.isSvelte === true}`,
                      `comments:${comments.length === 1}`,
                      `tokens:${tokens.length > 0}`,
                    ].join("; "),
                  });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      `<!--A--><script>console.log(1)</script><h1>Hello</h1>`,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    expect(receivedOptions).toMatchObject({
      filePath: "/workspace/App.svelte",
      sourceType: "script",
      ecmaVersion: 2022,
      comment: true,
      tokens: true,
      loc: true,
      range: true,
      raw: true,
      eslintVisitorKeys: true,
      eslintScopeManager: true,
    });

    const payload = JSON.parse(result);
    const hasCommentRoundTrip = payload.Success.comments.length === 1;
    if (allowMissingRealSvelteCommentRoundTrip) {
      expect(payload.Success.comments).toEqual(
        hasCommentRoundTrip ? [expect.objectContaining({ start: 0, end: 8 })] : [],
      );
    } else {
      expect(payload.Success.comments).toEqual([
        expect.objectContaining({ start: 0, end: 8 }),
      ]);
    }
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0].message).toBe(
      `sourceType:true; ecmaVersion:true; services:true; comments:${hasCommentRoundTrip}; tokens:true`,
    );
  });

  realParserIt("walks real Svelte nodes with selectors and wildcards via parser visitor keys", async () => {
    const parser = await loadRealSvelteParser();
    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-real-svelte": {
            create(context) {
              return {
                SvelteElement(node) {
                  const name = (node as { name?: { name?: string } }).name;
                  if (name?.name !== "h1") return;
                  context.report({ node, message: `element:${(node as { type: string }).type}` });
                },
                "SvelteElement > SvelteText"(node) {
                  context.report({ node, message: `selector:${(node as { value: string }).value}` });
                },
                "*"(node) {
                  if ((node as { type: string }).type === "SvelteText") {
                    context.report({ node, message: `wildcard:${(node as { type: string }).type}` });
                  }
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      "<h1>Hello</h1>",
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(
      payload.Success.diagnostics
        .map((diagnostic: { message: string }) => diagnostic.message)
        .sort(),
    ).toEqual(["element:SvelteElement", "selector:Hello", "wildcard:SvelteText"]);
  });

  realParserIt("respects real parser visitor key order across script and template nodes", async () => {
    const parser = await loadRealSvelteParser();
    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-real-order": {
            create(context) {
              return {
                SvelteScriptElement(node) {
                  context.report({ node, message: "script-element" });
                },
                VariableDeclaration(node) {
                  context.report({ node, message: "script-body" });
                },
                SvelteElement(node) {
                  const name = (node as { name?: { name?: string } }).name;
                  if (name?.name !== "h1") return;
                  context.report({ node, message: "template-element" });
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    setSingleRuleOptions();

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      `<script>const local = 1;</script><h1>Hello</h1>`,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(
      payload.Success.diagnostics.map((diagnostic: { message: string }) => diagnostic.message),
    ).toEqual(["script-element", "script-body", "template-element"]);
  });

});
