use std::{borrow::Cow, fmt::Write};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{RuleCategory, RuleFixMeta, rules::RULES};

pub struct RuleTable {
    pub sections: Vec<RuleTableSection>,
    pub total: usize,
    pub turned_on_by_default_count: usize,
}

pub struct RuleTableSection {
    pub rows: Vec<RuleTableRow>,
    pub category: RuleCategory,
    pub rule_column_width: usize,
    pub plugin_column_width: usize,
}

pub struct RuleTableRow {
    pub name: &'static str,
    pub plugin: String,
    pub category: RuleCategory,
    #[cfg(feature = "ruledocs")]
    pub documentation: Option<&'static str>,
    #[cfg(feature = "ruledocs")]
    pub schema: Option<schemars::schema::Schema>,

    pub turned_on_by_default: bool,
    pub autofix: RuleFixMeta,
    pub is_tsgolint_rule: bool,
}

impl Default for RuleTable {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RuleTable {
    #[expect(clippy::allow_attributes)]
    #[allow(unused, unused_mut)]
    pub fn new(mut generator: Option<&mut schemars::SchemaGenerator>) -> Self {
        let default_plugin_names = ["eslint", "unicorn", "typescript", "oxc"];

        let default_rules = RULES
            .iter()
            .filter(|rule| {
                rule.category() == RuleCategory::Correctness
                    && default_plugin_names.contains(&rule.plugin_name())
            })
            .map(super::rules::RuleEnum::name)
            .collect::<FxHashSet<&str>>();

        let mut rows = RULES
            .iter()
            .map(|rule| {
                let name = rule.name();
                RuleTableRow {
                    name,
                    #[cfg(feature = "ruledocs")]
                    documentation: rule.documentation(),
                    #[cfg(feature = "ruledocs")]
                    schema: generator.as_mut().and_then(|g| rule.schema(g)),
                    plugin: rule.plugin_name().to_string(),
                    category: rule.category(),
                    turned_on_by_default: default_rules.contains(name),
                    autofix: rule.fix(),
                    is_tsgolint_rule: rule.is_tsgolint_rule(),
                }
            })
            .collect::<Vec<_>>();

        let total = rows.len();

        rows.sort_by_key(|row| (row.plugin.clone(), row.name));

        let mut rows_by_category = rows.into_iter().fold(
            FxHashMap::default(),
            |mut map: FxHashMap<RuleCategory, Vec<RuleTableRow>>, row| {
                map.entry(row.category).or_default().push(row);
                map
            },
        );

        let sections = [
            RuleCategory::Correctness,
            RuleCategory::Perf,
            RuleCategory::Restriction,
            RuleCategory::Suspicious,
            RuleCategory::Pedantic,
            RuleCategory::Style,
            RuleCategory::Nursery,
        ]
        .into_iter()
        .filter_map(|category| {
            let rows = rows_by_category.remove(&category)?;
            let rule_column_width = rows.iter().map(|r| r.name.len()).max()?;
            let plugin_column_width = rows.iter().map(|r| r.plugin.len()).max()?;
            Some(RuleTableSection { rows, category, rule_column_width, plugin_column_width })
        })
        .collect::<Vec<_>>();

        RuleTable { total, sections, turned_on_by_default_count: default_rules.len() }
    }
}

impl RuleTableSection {
    /// Renders all the rules in this section as a markdown table.
    ///
    /// Provide [`Some`] prefix to render the rule name as a link. Provide
    /// [`None`] to just display the rule name as text.
    pub fn render_markdown_table(&self, link_prefix: Option<&str>) -> String {
        const FIX_EMOJI_COL_WIDTH: usize = 10;
        const DEFAULT_EMOJI_COL_WIDTH: usize = 9;
        /// text width, leave 2 spaces for padding
        const FIX: usize = FIX_EMOJI_COL_WIDTH - 2;
        const DEFAULT: usize = DEFAULT_EMOJI_COL_WIDTH - 2;

        let mut s = String::new();
        let category = &self.category;
        let rows = &self.rows;
        let rule_width = self.rule_column_width;
        let plugin_width = self.plugin_column_width;
        writeln!(s, "## {} ({}):", category, rows.len()).unwrap();

        writeln!(s, "{}", category.description()).unwrap();

        let x = "";
        writeln!(
            s,
            "| {:<rule_width$} | {:<plugin_width$} | Default | Fixable? |",
            "Rule name", "Source"
        )
        .unwrap();
        writeln!(s, "| {x:-<rule_width$} | {x:-<plugin_width$} | {x:-<7} | {x:-<8} |").unwrap();

        for row in rows {
            let rule_name = row.name;
            let plugin_name = &row.plugin;
            let (default, default_width) =
                if row.turned_on_by_default { ("✅", DEFAULT - 1) } else { ("", DEFAULT) };
            let rendered_name = if let Some(prefix) = link_prefix {
                Cow::Owned(format!("[{rule_name}]({prefix}/{plugin_name}/{rule_name}.html)"))
            } else {
                Cow::Borrowed(rule_name)
            };
            let (fix_emoji, fix_emoji_width) = row.autofix.emoji().map_or(("", FIX), |emoji| {
                let len = emoji.len();
                if len > FIX { (emoji, 0) } else { (emoji, FIX - len) }
            });
            writeln!(s, "| {rendered_name:<rule_width$} | {plugin_name:<plugin_width$} | {default:<default_width$} | {fix_emoji:<fix_emoji_width$} |").unwrap();
        }

        s
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use markdown::{Options, to_html_with_options};

    use super::*;

    static TABLE: OnceLock<RuleTable> = OnceLock::new();

    fn table() -> &'static RuleTable {
        TABLE.get_or_init(RuleTable::default)
    }

    #[test]
    fn test_table_no_links() {
        let options = Options::gfm();
        for section in &table().sections {
            let rendered_table = section.render_markdown_table(None);
            assert!(!rendered_table.is_empty());
            assert_eq!(rendered_table.split('\n').count(), 5 + section.rows.len());

            let html = to_html_with_options(&rendered_table, &options).unwrap();
            assert!(!html.is_empty());
            assert!(html.contains("<table>"));
        }
    }

    #[test]
    fn test_table_with_links() {
        const PREFIX: &str = "/foo/bar";
        const PREFIX_WITH_SLASH: &str = "/foo/bar/";

        let options = Options::gfm();

        for section in &table().sections {
            let rendered_table = section.render_markdown_table(Some(PREFIX));
            assert!(!rendered_table.is_empty());
            assert_eq!(rendered_table.split('\n').count(), 5 + section.rows.len());

            let html = to_html_with_options(&rendered_table, &options).unwrap();
            assert!(!html.is_empty());
            assert!(html.contains("<table>"));
            assert!(html.contains(PREFIX_WITH_SLASH));
        }
    }
}
