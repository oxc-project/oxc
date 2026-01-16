// oxlint-disable no-console

import { parseArgs } from "node:util";
import { ALL_TARGET_PLUGINS, createESLintLinter, loadTargetPluginRules } from "./eslint-rules.mts";
import { renderMarkdown } from "./markdown-renderer.mts";
import {
  createRuleEntries,
  overrideTypeScriptPluginStatusWithEslintPluginStatus as syncTypeScriptPluginStatusWithEslintPluginStatus,
  syncUnicornPluginStatusWithEslintPluginStatus,
  syncVitestPluginStatusWithJestPluginStatus,
  updateImplementedStatus,
  updateNotSupportedStatus,
  updatePendingFixStatus,
} from "./oxlint-rules.mts";
import { updateGitHubIssue } from "./result-reporter.mts";

const HELP = `
Usage:
  $ cmd [--target=<pluginName>]... [--update] [--help]

Options:
  --target, -t: Which plugin to target, multiple allowed
  --update: Update the issue instead of printing to stdout
  --help, -h: Print this help message

Plugins: ${Array.from(ALL_TARGET_PLUGINS.keys()).join(", ")}
`;

void (async () => {
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

  const targetPluginNames = new Set(values.target ?? ALL_TARGET_PLUGINS.keys());
  for (const pluginName of targetPluginNames) {
    if (!ALL_TARGET_PLUGINS.has(pluginName)) {
      console.error(`Unknown plugin name: ${String(pluginName)}`);
      return;
    }
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
  await updatePendingFixStatus(ruleEntries);
  await syncTypeScriptPluginStatusWithEslintPluginStatus(ruleEntries);
  await syncVitestPluginStatusWithJestPluginStatus(ruleEntries);
  syncUnicornPluginStatusWithEslintPluginStatus(ruleEntries);

  //
  // Render list and update if necessary
  //
  const results = await Promise.allSettled(
    Array.from(targetPluginNames).map((pluginName) => {
      const pluginMeta = ALL_TARGET_PLUGINS.get(pluginName);
      const content = renderMarkdown(pluginName, pluginMeta!, ruleEntries);

      if (!values.update) return Promise.resolve(content);
      // Requires `env.GITHUB_TOKEN`
      return updateGitHubIssue(pluginMeta!, content);
    }),
  );
  for (const result of results) {
    if (result.status === "fulfilled") console.log(result.value);
    if (result.status === "rejected") console.error(result.reason);
  }
})();
