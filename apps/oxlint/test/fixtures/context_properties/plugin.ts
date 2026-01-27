import type { Node, Plugin, Rule } from "#oxlint/plugin";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const rule: Rule = {
  create(context) {
    context.report({
      message:
        "\n" +
        `this === rule: ${this === rule}\n` +
        `id: ${context.id}\n` +
        `filename: ${context.filename}\n` +
        `getFilename(): ${context.getFilename()}\n` +
        `physicalFilename: ${context.physicalFilename}\n` +
        `getPhysicalFilename(): ${context.getPhysicalFilename()}\n` +
        `cwd: ${context.cwd}\n` +
        `getCwd(): ${context.getCwd()}`,
      node: SPAN,
    });

    return {
      VariableDeclaration(node) {
        context.report({ message: `\nthis === undefined: ${this === undefined}`, node });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "context-plugin",
  },
  rules: {
    "log-context": rule,
  },
};

export default plugin;
