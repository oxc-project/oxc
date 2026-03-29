// oxlint-disable no-console

import { ALL_TARGET_PLUGINS, createESLintLinter, loadTargetPluginRules } from "./eslint-rules.mjs";
import {
  createRuleEntries,
  overrideTypeScriptPluginStatusWithEslintPluginStatus as syncTypeScriptPluginStatusWithEslintPluginStatus,
  syncUnicornPluginStatusWithEslintPluginStatus,
  syncVitestPluginStatusWithJestPluginStatus,
  updateImplementedStatus,
  updateNotSupportedStatus,
  updatePendingFixStatus,
} from "./oxlint-rules.mjs";

// Initialize linter and load plugin rules
const linter = createESLintLinter();
// Include type-checked rules, for accurate counting of the total implemented number.
loadTargetPluginRules(linter, true);

// Build rule entries and update statuses
const ruleEntries = createRuleEntries(linter.getRules());
await updateImplementedStatus(ruleEntries);
updateNotSupportedStatus(ruleEntries);
await updatePendingFixStatus(ruleEntries);
await syncTypeScriptPluginStatusWithEslintPluginStatus(ruleEntries);
await syncVitestPluginStatusWithJestPluginStatus(ruleEntries);
syncUnicornPluginStatusWithEslintPluginStatus(ruleEntries);

// Helper to gather stats for a plugin
const statsForPlugin = (pluginName) => {
  const prefix = `${pluginName}/`;
  const entries = Array.from(ruleEntries.entries()).filter(([name]) => name.startsWith(prefix));
  const filtered = entries.filter(([_, e]) => !e.isNotSupported);
  const total = filtered.length;
  const implemented = filtered.filter(([_, e]) => e.isImplemented).length;
  const notImplemented = total - implemented;
  const notSupported = entries.length - filtered.length;
  return { pluginName, total, implemented, notImplemented, notSupported };
};

// Compute per-plugin and overall stats
const pluginNames = Array.from(ALL_TARGET_PLUGINS.keys());
const perPlugin = pluginNames.map((name) => statsForPlugin(name));

const overall = perPlugin.reduce(
  (acc, cur) => ({
    total: acc.total + cur.total,
    implemented: acc.implemented + cur.implemented,
    notImplemented: acc.notImplemented + cur.notImplemented,
    notSupported: acc.notSupported + cur.notSupported,
  }),
  { total: 0, implemented: 0, notImplemented: 0, notSupported: 0 },
);

// print the numbers of rules that are implemented, not implemented, and not supported
for (const { pluginName, total, implemented, notImplemented, notSupported } of perPlugin) {
  const implementedPct = total === 0 ? "-" : `${((implemented / total) * 100).toFixed(2)}%`;
  console.log(
    `${pluginName.padEnd(12)} Implemented: ${implemented.toString().padStart(4)} / ${total
      .toString()
      .padStart(4)} (${implementedPct.padStart(7)}), Not Implemented: ${notImplemented
      .toString()
      .padStart(3)}, Not Supported: ${notSupported.toString().padStart(3)}`,
  );
}

// Print overall summary
const overallPct =
  overall.total === 0 ? "-" : `${((overall.implemented / overall.total) * 100).toFixed(2)}%`;
console.log(
  "-----------------------------------------------------------------------------------------",
);
console.log(
  `${"total".padEnd(12)} Implemented: ${overall.implemented.toString().padStart(4)} / ${overall.total
    .toString()
    .padStart(4)} (${overallPct.padStart(7)}), Not Implemented: ${overall.notImplemented
    .toString()
    .padStart(3)}, Not Supported: ${overall.notSupported.toString().padStart(3)}`,
);
