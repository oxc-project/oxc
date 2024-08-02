use std::fmt::Write;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{rules::RULES, Linter, RuleCategory};

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
    pub documentation: Option<&'static str>,
    pub turned_on_by_default: bool,
}

impl Default for RuleTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleTable {
    pub fn new() -> Self {
        let default_rules = Linter::default()
            .rules
            .into_iter()
            .map(|rule| rule.name())
            .collect::<FxHashSet<&str>>();

        let mut rows = RULES
            .iter()
            .map(|rule| {
                let name = rule.name();
                RuleTableRow {
                    name,
                    documentation: rule.documentation(),
                    plugin: rule.plugin_name().to_string(),
                    category: rule.category(),
                    turned_on_by_default: default_rules.contains(name),
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
    pub fn render_markdown_table(&self) -> String {
        let mut s = String::new();
        let category = &self.category;
        let rows = &self.rows;
        let rule_width = self.rule_column_width;
        let plugin_width = self.plugin_column_width;
        writeln!(s, "## {} ({}):", category, rows.len()).unwrap();

        writeln!(s, "{}", category.description()).unwrap();

        let x = "";
        writeln!(s, "| {:<rule_width$} | {:<plugin_width$} | Default |", "Rule name", "Source")
            .unwrap();
        writeln!(s, "| {x:-<rule_width$} | {x:-<plugin_width$} | {x:-<7} |").unwrap();

        for row in rows {
            let rule_name = row.name;
            let plugin_name = &row.plugin;
            let (default, default_width) =
                if row.turned_on_by_default { ("✅", 6) } else { ("", 7) };
            writeln!(s, "| {rule_name:<rule_width$} | {plugin_name:<plugin_width$} | {default:<default_width$} |").unwrap();
        }

        s
    }
}
