/**
 * @typedef {({ name: string } & import("./oxlint-rules.cjs").RuleEntry)} RuleEntryView
 * @typedef {{ isImplemented: number; isNotSupported: number; total: number }} CounterView
 */

/** @param {{ npm: string; }} props */
const renderIntroduction = ({ npm }) => `
> [!WARNING]
> This comment is maintained by CI. Do not edit this comment directly.
> To update comment template, see https://github.com/oxc-project/oxc/tree/main/tasks/lint_rules

This is tracking issue for \`${npm}\`.
`;

/**
 * @param {{
 *   counters: {
 *     recommended: CounterView;
 *     others: CounterView;
 *     deprecated: CounterView;
 *   };
 * }} props
 */
const renderCounters = ({ counters: { recommended, others, deprecated } }) => `
- There are ${recommended.total + others.total}(+ ${deprecated.total} deprecated) rules
- ${recommended.total - (recommended.isImplemented + recommended.isNotSupported)}/${recommended.total} recommended rules are remaining as TODO
- ${others.total - (others.isImplemented + others.isNotSupported)}/${others.total} not recommended rules are remaining as TODO
`;

/** @param {{ pluginName: string }} props */
const renderGettingStarted = ({ pluginName }) => `
To get started, run the following command:

\`\`\`sh
just new-${pluginName}-rule <RULE_NAME>
\`\`\`

Then register the rule in \`crates/oxc_linter/src/rules.rs\` and also \`declare_all_lint_rules\` at the bottom.
`;

/**
 * @param {{
 *   title: string;
 *   counters: CounterView;
 *   views: RuleEntryView[];
 *   defaultOpen?: boolean;
 * }} props */
const renderRulesList = ({ title, counters, views, defaultOpen = true }) => `
## ${title}

<details ${defaultOpen ? "open" : ""}>
<summary>
 ✅: ${counters.isImplemented}, 🚫: ${counters.isNotSupported} / total: ${counters.total}
</summary>

✅ = Implemented, 🚫 = Not supported

| Status | Name | Docs |
| :----: | :--- | :--- |
${views
  .map(
    (v) =>
      `| ${v.isImplemented ? "✅" : ""}${v.isNotSupported ? "🚫" : ""} | ${v.name} | ${v.docsUrl} |`,
  )
  .join("\n")}

</details>
`;

/**
 * @param {string} pluginName
 * @param {import("./eslint-rules.cjs").TargetPluginMeta} pluginMeta
 * @param {import("./oxlint-rules.cjs").RuleEntries} ruleEntries
 */
exports.renderMarkdown = (pluginName, pluginMeta, ruleEntries) => {
  /** @type {Record<string, RuleEntryView[]>} */
  const views = {
    deprecated: [],
    recommended: [],
    others: [],
  };
  const counters = {
    deprecated: { isImplemented: 0, isNotSupported: 0, total: 0 },
    recommended: { isImplemented: 0, isNotSupported: 0, total: 0 },
    others: { isImplemented: 0, isNotSupported: 0, total: 0 },
  };
  for (const [name, entry] of ruleEntries) {
    if (!name.startsWith(`${pluginName}/`)) continue;

    let viewsRef, counterRef;

    switch (true) {
      case entry.isDeprecated: {
        viewsRef = views.deprecated;
        counterRef = counters.deprecated;
        break;
      }
      case entry.isRecommended: {
        viewsRef = views.recommended;
        counterRef = counters.recommended;
        break;
      }
      default: {
        viewsRef = views.others;
        counterRef = counters.others;
      }
    }

    viewsRef.push({ name, ...entry });

    if (entry.isImplemented) counterRef.isImplemented++;
    if (entry.isNotSupported) counterRef.isNotSupported++;
    counterRef.total++;
  }

  return [
    renderIntroduction({ npm: pluginMeta.npm }),
    renderCounters({ counters }),
    renderGettingStarted({ pluginName }),
    0 < views.recommended.length &&
      renderRulesList({
        title: "Recommended rules",
        counters: counters.recommended,
        views: views.recommended,
      }),
    0 < views.others.length &&
      renderRulesList({
        title: "Not recommended rules",
        counters: counters.others,
        views: views.others,
      }),
    0 < views.deprecated.length &&
      renderRulesList({
        title: "Deprecated rules",
        counters: counters.deprecated,
        views: views.deprecated,
        defaultOpen: false,
      }),
  ]
    .filter(Boolean)
    .join("\n");
};
