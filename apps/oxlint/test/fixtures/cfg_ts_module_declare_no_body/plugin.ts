import type { Plugin, Rule } from "#oxlint/plugins";

const rule: Rule = {
  create() {
    return {
      onCodePathStart() {
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "cfg-plugin",
  },
  rules: {
    cfg: rule,
  },
};

export default plugin;
