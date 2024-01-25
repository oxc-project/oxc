const { parseArgs } = require("node:util");
const {
  createESLintLinter,
  loadPluginUnicornRules,
  loadPluginJSDocRules,
  loadPluginImportRules,
  loadPluginJestRules,
} = require("./eslint-rules.cjs");
const {
  createRuleEntries,
  readAllImplementedRuleNames,
  updateNotSupportedStatus,
  updateImplementedStatus,
} = require("./oxlint-rules.cjs");
const { renderRulesList, renderLayout } = require("./output-markdown.cjs");

const ALL_TARGET_PLUGIN_NAMES = new Set([
  "eslint",
  "unicorn",
  "jsdoc",
  "import",
  "jest",
]);

const HELP = `
Usage:
  $ cmd [--target=<pluginName>] [--update] [--help]

Options:
  --target, -t: Which plugin to target, one of ${[...ALL_TARGET_PLUGIN_NAMES].join(", ")}
  --update: Update the issue instead of printing to stdout
  --help, -h: Print this help message
`;

(async () => {
  //
  // Parse arguments
  //
  const { values } = parseArgs({
    options: {
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
  loadPluginUnicornRules(linter);
  loadPluginJSDocRules(linter);
  loadPluginImportRules(linter);
  loadPluginJestRules(linter);
  // TODO: more plugins

  //
  // Generate entry and update status
  //
  const ruleEntries = createRuleEntries(linter.getRules());
  const implementedRuleNames = await readAllImplementedRuleNames();
  updateImplementedStatus(ruleEntries, implementedRuleNames);
  updateNotSupportedStatus(ruleEntries);

  //
  // Render list and update if necessary
  //
  await Promise.allSettled(
    Array.from(targetPluginNames).map(async (pluginName) => {
      const listPart = renderRulesList(ruleEntries, pluginName);
      const content = renderLayout(listPart, pluginName);

      if (!values.update) return console.log(content);
      // TODO: Update issue
    }),
  );
})();
