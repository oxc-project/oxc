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
  const counters = { deprecated: [0, 0], recommended: [0, 0], others: [0, 0] };
  for (const [name, entry] of ruleEntries) {
    if (!name.startsWith(`${pluginName}/`)) continue;

    const view = { name, ...entry };
    const isMarked = entry.isImplemented || entry.isNotSupported;

    if (entry.isDeprecated) {
      deprecated.push(view);
      if (isMarked) counters.deprecated[0] += 1;
      continue;
    }

    if (entry.isRecommended) {
      recommended.push(view);
      if (isMarked) counters.recommended[0] += 1;
      continue;
    }
    others.push(view);
    if (isMarked) counters.others[0] += 1;
  }

  counters.deprecated[1] = deprecated.length;
  counters.recommended[1] = recommended.length;
  counters.others[1] = others.length;

  // TODO: How to display counters?

  return [
    renderWarning(),
    renderHeader({ npm: pluginMeta.npm }),
    0 < recommended.length &&
      renderRulesList({
        title: "Recommended",
        views: recommended,
      }),
    0 < others.length &&
      renderRulesList({
        title: "Not recommended",
        views: others,
      }),
    0 < deprecated.length &&
      renderRulesList({
        title: "Deprecated",
        views: deprecated,
        defaultOpen: false,
      }),
    renderGettingStarted({ pluginName }),
  ]
    .filter(Boolean)
    .join("\n");
};
