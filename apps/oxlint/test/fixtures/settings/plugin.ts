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
    const { settings } = context;

    // Report each setting key and value
    Object.keys(settings).forEach((key) => {
      const value = settings[key];
      context.report({
        message: `setting ${key}: ${JSON.stringify(value)}`,
        node: SPAN,
      });
    });

    // Also report if settings object is empty
    if (Object.keys(settings).length === 0) {
      context.report({
        message: "settings is empty",
        node: SPAN,
      });
    }

    return {};
  },
};

const plugin: Plugin = {
  meta: {
    name: "context-settings-plugin",
  },
  rules: {
    "log-settings": rule,
  },
};

export default plugin;
