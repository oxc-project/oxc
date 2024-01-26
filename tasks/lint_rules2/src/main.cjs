const { parseArgs } = require("node:util");
const {
  ALL_TARGET_PLUGIN_NAMES,
  createESLintLinter,
  loadTargetPluginRules,
} = require("./eslint-rules.cjs");
const {
  createRuleEntries,
  updateNotSupportedStatus,
  updateImplementedStatus,
} = require("./oxlint-rules.cjs");
const { renderMarkdown } = require("./output-markdown.cjs");

const HELP = `
Usage:
  $ cmd [--target=<pluginName>]... [--update] [--help]

Options:
  --target, -t: Which plugin to target, multiple allowed
  --update: Update the issue instead of printing to stdout
  --help, -h: Print this help message

Plugins: ${[...ALL_TARGET_PLUGIN_NAMES].join(", ")}
`;

(async () => {
  //
  // Parse arguments
  //
  const { values } = parseArgs({
    options: {
      // Mainly for debugging
      target: { type: "string", short: "t", multiple: true },
      update: { type: "boolean" },
      help: { type: "boolean", short: "h" },
    },
  });

  if (values.help) return console.log(HELP);

  const targetPluginNames = new Set(values.target ?? ALL_TARGET_PLUGIN_NAMES);
  for (const pluginName of targetPluginNames) {
    if (!ALL_TARGET_PLUGIN_NAMES.has(pluginName))
      throw new Error(`Unknown plugin name: ${pluginName}`);
  }

  //
  // Load linter and all plugins
  //
  const linter = createESLintLinter();
  loadTargetPluginRules(linter);

  //
  // Generate entry and update status
  //
  const ruleEntries = createRuleEntries(linter.getRules());
  await updateImplementedStatus(ruleEntries);
  updateNotSupportedStatus(ruleEntries);

  //
  // Render list and update if necessary
  //
  await Promise.allSettled(
    Array.from(targetPluginNames).map(async (pluginName) => {
      const content = renderMarkdown(pluginName, ruleEntries);

      if (!values.update) return console.log(content);
      // TODO: Update issue
    }),
  );
})();
