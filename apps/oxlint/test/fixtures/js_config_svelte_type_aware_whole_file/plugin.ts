import type { Node, Plugin } from "#oxlint/plugins";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte-type-aware",
  },
  rules: {
    "options-visible": {
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
          "Program:exit"() {
            const parserServices = context.sourceCode.parserServices as {
              isSvelte?: unknown;
              isSvelteScript?: unknown;
              svelteParseContext?: { runes?: unknown };
              esTreeNodeToTSNodeMap?: { get?: unknown };
              tsNodeToESTreeNodeMap?: { get?: unknown };
            };
            const parserOptions = context.languageOptions.parserOptions as {
              parser?: { parseForESLint?: unknown };
              svelteConfig?: {
                compilerOptions?: { runes?: unknown };
                preprocess?: unknown;
              };
              projectService?: unknown;
              extraFileExtensions?: unknown;
              tsFlavor?: unknown;
              tsconfigRootDir?: unknown;
            };
            const extraFileExtensions = Array.isArray(parserOptions.extraFileExtensions)
              ? parserOptions.extraFileExtensions
              : [];
            const hasTsMaps =
              typeof parserServices.esTreeNodeToTSNodeMap?.get === "function" &&
              typeof parserServices.tsNodeToESTreeNodeMap?.get === "function";

            context.report({
              message: [
                `parser: ${context.languageOptions.parser?.name === "svelte-eslint-parser"}`,
                `services: ${parserServices.isSvelte === true && parserServices.isSvelteScript === false}`,
                `projectService: ${parserOptions.projectService === true && typeof parserOptions.tsconfigRootDir === "string"}`,
                `extraFileExtensions: ${extraFileExtensions.includes(".svelte")}`,
                `nestedParserFn: ${typeof parserOptions.parser?.parseForESLint === "function"}`,
                `svelteRunes: ${parserOptions.svelteConfig?.compilerOptions?.runes === true && parserServices.svelteParseContext?.runes === true}`,
                `preprocessFn: ${typeof parserOptions.svelteConfig?.preprocess === "function"}`,
                `mergedBaseOption: ${parserOptions.tsFlavor === "base-ts-parser"}`,
                `scriptCount: ${scriptCount}`,
                `styleCount: ${styleCount}`,
                `tsMaps: ${hasTsMaps}`,
                `tsNodes: ${tsNodeCount}`,
                `element: ${firstElementName}`,
              ].join("; "),
              node: SPAN,
            });
          },
        };
      },
    },
  },
};

export default plugin;
