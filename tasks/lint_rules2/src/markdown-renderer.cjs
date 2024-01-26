/**
 * @param {string} pluginName
 * @param {import("./eslint-rules.cjs").TargetPluginMeta} pluginMeta
 * @param {string} listPart
 */
const renderLayout = (pluginName, pluginMeta, listPart) => `
> [!WARNING]
> This comment is maintained by CI. Do not edit this comment directly.
> To update comment template, see https://github.com/oxc-project/oxc/tree/main/tasks/lint_rules

This is tracking issue for \`${pluginMeta.npm}\`.

## Rules
${listPart}

## Getting started

\`\`\`sh
just new-${pluginName}-rule <RULE_NAME>
\`\`\`

Then register the rule in \`crates/oxc_linter/src/rules.rs\` and also \`declare_all_lint_rules\` at the bottom.
`;

/** @param {[string, import("./oxlint-rules.cjs").RuleEntry][]} ruleEntries */
const renderRulesList = (ruleEntries) => {
  /* prettier-ignore */
  const list = [
    "| Name | Kind | Status | Docs |",
    "| :--- | :--: | :----: | :--- |",
  ];

  for (const [name, entry] of ruleEntries) {
    // These should be exclusive, but show it for sure...
    let kind = "";
    if (entry.isRecommended) kind += "🍀";
    if (entry.isDeprecated) kind += "⚠️";

    let status = "";
    if (entry.isImplemented) status += "✨";
    if (entry.isNotSupported) status += "🚫";

    list.push(`| ${name} | ${kind} | ${status} | ${entry.docsUrl} |`);
  }

  return `
- Kind: 🍀 = recommended | ⚠️ = deprecated
- Status: ✨ = implemented | 🚫 = not supported

${list.join("\n")}
`;
};

/**
 * @param {string} pluginName
 * @param {import("./eslint-rules.cjs").TargetPluginMeta} pluginMeta
 * @param {import("./oxlint-rules.cjs").RuleEntries} ruleEntries
 */
exports.renderMarkdown = (pluginName, pluginMeta, ruleEntries) => {
  const pluginRules = Array.from(ruleEntries).filter(([name]) =>
    name.startsWith(`${pluginName}/`),
  );
  return renderLayout(pluginName, pluginMeta, renderRulesList(pluginRules));
};
