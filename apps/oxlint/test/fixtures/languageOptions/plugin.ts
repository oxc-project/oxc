import assert from "node:assert";

import type { Plugin, Node } from "#oxlint/plugin";

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
    name: "language-options-plugin",
  },
  rules: {
    lang: {
      create(context) {
        const { languageOptions } = context;

        assert(context.parserOptions === languageOptions.parserOptions);

        context.report({
          message:
            "languageOptions:\n" +
            `sourceType: ${languageOptions.sourceType}\n` +
            `ecmaVersion: ${languageOptions.ecmaVersion}\n` +
            `parserOptions: ${JSON.stringify(languageOptions.parserOptions)}\n` +
            `globals: ${JSON.stringify(languageOptions.globals)}\n` +
            `env: ${JSON.stringify(languageOptions.env)}`,
          node: SPAN,
        });

        // Only output once, as it's the same for all files
        if (context.filename.endsWith("index.js")) {
          const { parser } = languageOptions;

          context.report({
            message:
              "parser:\n" +
              // oxlint-disable-next-line typescript/restrict-template-expressions
              `object keys: ${Reflect.ownKeys(parser)}\n` +
              `name: ${parser.name}\n` +
              // Don't include `version` in the message, as it'll change each time we do a release
              `typeof version: ${typeof parser.version}\n` +
              `typeof parse: ${typeof parser.parse}\n` +
              `latestEcmaVersion: ${parser.latestEcmaVersion}\n` +
              // oxlint-disable-next-line typescript/restrict-template-expressions
              `supportedEcmaVersions: ${parser.supportedEcmaVersions}\n` +
              `Syntax: ${JSON.stringify(parser.Syntax, null, 2)}\n` +
              `VisitorKeys: ${JSON.stringify(parser.VisitorKeys, null, 2)}`,
            node: SPAN,
          });
        }

        return {};
      },
    },
  },
};

export default plugin;
