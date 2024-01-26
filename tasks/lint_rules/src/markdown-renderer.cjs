/**
 * @typedef {({ name: string } & import("./oxlint-rules.cjs").RuleEntry)} RuleEntryView
 */

const renderWarning = () => `
> [!WARNING]
> This comment is maintained by CI. Do not edit this comment directly.
> To update comment template, see https://github.com/oxc-project/oxc/tree/main/tasks/lint_rules
`;

/** @param {{ npm: string; }} props */
const renderHeader = ({ npm }) => `
This is tracking issue for \`${npm}\`.
`;

/** @param {{ pluginName: string }} props */
const renderGettingStarted = ({ pluginName }) => `
## Getting started

\`\`\`sh
just new-${pluginName}-rule <RULE_NAME>
\`\`\`

Then register the rule in \`crates/oxc_linter/src/rules.rs\` and also \`declare_all_lint_rules\` at the bottom.
`;

/**
 * @param {{
 *   title: string;
 *   views: RuleEntryView[];
 *   defaultOpen?: boolean;
 * }} props */
const renderRulesList = ({ title, views, defaultOpen = true }) => `
## ${title}

<details ${defaultOpen ? "open" : ""}>

| Status | Name | Docs |
| :----: | :--- | :--- |
${views
  .map(
    (v) =>
      `| ${v.isImplemented ? "✅" : ""}${v.isNotSupported ? "🚫" : ""} | ${v.name} | ${v.docsUrl} |`,
  )
  .join("\n")}

✅ = Implemented, 🚫 = Not supported

</details>
`;

/**
 * @param {string} pluginName
 * @param {import("./eslint-rules.cjs").TargetPluginMeta} pluginMeta
 * @param {import("./oxlint-rules.cjs").RuleEntries} ruleEntries
 */
exports.renderMarkdown = (pluginName, pluginMeta, ruleEntries) => {
  /** @type {RuleEntryView[][]} */
  const [deprecated, recommended, others] = [[], [], []];
  for (const [name, entry] of ruleEntries) {
    if (!name.startsWith(`${pluginName}/`)) continue;

    const view = { name, ...entry };

    if (entry.isDeprecated) {
      deprecated.push(view);
      continue;
    }
    if (entry.isRecommended) {
      recommended.push(view);
      continue;
    }
    others.push(view);
  }

  return [
    renderWarning(),
    renderHeader({ npm: pluginMeta.npm }),
    0 < recommended.length &&
      renderRulesList({ title: "Recommended rules", views: recommended }),
    0 < others.length &&
      renderRulesList({ title: "Not recommended rules", views: others }),
    0 < deprecated.length &&
      renderRulesList({
        title: "Deprecated rules",
        views: deprecated,
        defaultOpen: false,
      }),
    renderGettingStarted({ pluginName }),
  ]
    .filter(Boolean)
    .join("\n");
};
