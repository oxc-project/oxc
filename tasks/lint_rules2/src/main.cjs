const { parseArgs } = require("node:util");
const {
  createESLintLinter,
  loadPluginTypeScriptRules,
  loadPluginNRules,
  loadPluginUnicornRules,
  loadPluginJSDocRules,
  loadPluginImportRules,
  loadPluginJSXA11yRules,
  loadPluginJestRules,
  loadPluginReactRules,
  loadPluginReactHooksRules,
  loadPluginReactPerfRules,
  loadPluginNextRules,
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
  "typescript",
  "n",
  "unicorn",
  "jsdoc",
  "import",
  "jsx-a11y",
  "jest",
  "react",
  "react-hooks",
  "react-perf",
  "nextjs",
]);

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
  loadPluginTypeScriptRules(linter);
  loadPluginNRules(linter);
  loadPluginUnicornRules(linter);
  loadPluginJSDocRules(linter);
  loadPluginImportRules(linter);
  loadPluginJSXA11yRules(linter);
  loadPluginJestRules(linter);
  loadPluginReactRules(linter);
  loadPluginReactHooksRules(linter);
  loadPluginReactPerfRules(linter);
  loadPluginNextRules(linter);

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
