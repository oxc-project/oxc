import { TargetPluginMeta } from "./eslint-rules";
import type { RuleEntry } from "./oxlint-rules";

type RuleEntryView = { name: string } & RuleEntry;
type CounterView = {
  isImplemented: number;
  isNotSupported: number;
  isPendingFix: number;
  total: number;
};

const renderIntroduction = ({ npm }: { npm: string[] }) => `
> [!WARNING]
> This comment is maintained by CI. Do not edit this comment directly.
> To update comment template, see https://github.com/oxc-project/oxc/tree/main/tasks/lint_rules

This is tracking issue for ${npm.map((n) => "`" + n + "`").join(", ")}.
`;

const renderCounters = ({
  counters: { recommended, notRecommended, deprecated },
}: {
  counters: { recommended: CounterView; notRecommended: CounterView; deprecated: CounterView };
}) => {
  const recommendedTodos =
    recommended.total - (recommended.isImplemented + recommended.isNotSupported);
  const notRecommendedTodos =
    notRecommended.total - (notRecommended.isImplemented + notRecommended.isNotSupported);

  const countersList = [
    `- ${recommendedTodos}/${recommended.total} recommended rules are remaining as TODO`,
    recommended.isPendingFix > 0 && `  - ${recommended.isPendingFix} of which have pending fixes`,
    recommendedTodos === 0 && `  - All done! 🎉`,
    `- ${notRecommendedTodos}/${notRecommended.total} not recommended rules are remaining as TODO`,
    notRecommended.isPendingFix > 0 &&
      `  - ${notRecommended.isPendingFix} of which have pending fixes`,
    notRecommendedTodos === 0 && `  - All done! 🎉`,
  ]
    .filter(Boolean)
    .join("\n");

  return `
There are ${recommended.total + notRecommended.total}(+ ${deprecated.total} deprecated) rules.

${countersList}
`;
};

const renderGettingStarted = ({ pluginName }: { pluginName: string }) => `
To get started, run the following command:

\`\`\`sh
just new-${pluginName}-rule <RULE_NAME>
\`\`\`

Then register the rule in \`crates/oxc_linter/src/rules.rs\` and also \`declare_all_lint_rules\` at the bottom.
`;

const renderRulesList = ({
  title,
  counters,
  views,
  defaultOpen = true,
}: {
  title: string;
  counters: CounterView;
  views: RuleEntryView[];
  defaultOpen?: boolean;
}) => `
## ${title}

<details ${defaultOpen ? "open" : ""}>
<summary>
  ✅: ${counters.isImplemented}, 🚫: ${counters.isNotSupported}, ⏳: ${counters.isPendingFix} / total: ${counters.total}
</summary>

| Status | Name |
| :----: | :--- |
${views
  .map((v) => {
    let status = "";
    if (v.isImplemented) status += "✅";
    if (v.isNotSupported) status += "🚫";
    if (v.isPendingFix) status += "⏳";
    const name = v.docsUrl ? `[${v.name}](${v.docsUrl})` : v.name;
    return `| ${status} | ${name} |`;
  })
  .join("\n")}

✅ = Implemented, 🚫 = No need to implement, ⏳ = Fix pending

</details>
`;

export const renderMarkdown = (
  pluginName: string,
  pluginMeta: TargetPluginMeta,
  ruleEntries: Map<string, RuleEntry>,
) => {
  const views: Record<string, RuleEntryView[]> = {
    deprecated: [],
    recommended: [],
    notRecommended: [],
  };
  const counters = {
    deprecated: { isImplemented: 0, isNotSupported: 0, isPendingFix: 0, total: 0 },
    recommended: { isImplemented: 0, isNotSupported: 0, isPendingFix: 0, total: 0 },
    notRecommended: { isImplemented: 0, isNotSupported: 0, isPendingFix: 0, total: 0 },
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
        viewsRef = views.notRecommended;
        counterRef = counters.notRecommended;
      }
    }

    viewsRef.push({ name, ...entry });

    if (entry.isImplemented) counterRef.isImplemented++;
    else if (entry.isNotSupported) counterRef.isNotSupported++;
    if (entry.isPendingFix && entry.isImplemented) counterRef.isPendingFix++;
    counterRef.total++;
  }

  const sortedViews = sortViews(views);

  return [
    renderIntroduction({ npm: pluginMeta.npm }),
    renderCounters({ counters }),
    renderGettingStarted({ pluginName }),
    0 < sortedViews.recommended.length &&
      renderRulesList({
        title: "Recommended rules",
        counters: counters.recommended,
        views: sortedViews.recommended,
      }),
    0 < sortedViews.notRecommended.length &&
      renderRulesList({
        title: "Not recommended rules",
        counters: counters.notRecommended,
        views: sortedViews.notRecommended,
      }),
    0 < sortedViews.deprecated.length &&
      renderRulesList({
        title: "Deprecated rules",
        counters: counters.deprecated,
        views: sortedViews.deprecated,
        defaultOpen: false,
      }),
  ]
    .filter(Boolean)
    .join("\n");
};

function sortViews(views: Record<string, RuleEntryView[]>): Record<string, RuleEntryView[]> {
  const copy = { ...views };

  const unprefix = (name: string) => name.split("/").pop() || "";

  const byRuleName = (a: RuleEntryView, b: RuleEntryView) =>
    unprefix(a.name).localeCompare(unprefix(b.name));

  for (const key in views) copy[key] = views[key].toSorted(byRuleName);

  return copy;
}
