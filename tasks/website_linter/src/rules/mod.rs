#![expect(clippy::print_stderr)]
mod doc_page;
mod html;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use doc_page::Context;
use html::HtmlWriter;
use oxc_linter::{
    Oxlintrc,
    table::{RuleTable, RuleTableRow},
};
use pico_args::Arguments;
use schemars::{SchemaGenerator, r#gen::SchemaSettings};

const HELP: &str = "
usage: linter-rules [args]

Arguments:
    -j,--rules-json <path> Path to file where rules json blob will be saved.
    -r,--rule-docs <path>  Path to directory where rule doc pages will be saved.
                           A directory will be created if one doesn't exist.
    -c,--rule-count <path> Path to directory where rule count data file will be saved.
    --git-ref <ref>        Git commit, branch, or tag to be used in the generated links.
                           If not supplied, `main` will be used.
    -h,--help              Show this help message.

";

/// `print_rules`
///
/// `cargo run -p website linter-rules
///   --rules-json /path/to/oxc/website/.vitepress/data/rules.json
///   --rule-docs /path/to/oxc/website/src/docs/guide/usage/linter/rules
///   --rule-count /path/to/oxc/website/src/docs/guide/usage
///   --git-ref dc9dc03872101c15b0d02f05ce45705565665829
/// `
/// <https://oxc.rs/docs/guide/usage/linter/rules.html>
#[expect(clippy::print_stdout)]
pub fn print_rules(mut args: Arguments) {
    let pwd = PathBuf::from(env::var("PWD").unwrap());
    if args.contains(["-h", "--help"]) {
        println!("{HELP}");
        return;
    }

    let git_ref: Option<String> = args.opt_value_from_str("--git-ref").unwrap();
    let rules_json_path = args.opt_value_from_str::<_, PathBuf>(["-j", "--rules-json"]).unwrap();
    let rules_dir = args.opt_value_from_str::<_, PathBuf>(["-r", "--rule-docs"]).unwrap();
    let rule_count_dir = args.opt_value_from_str::<_, PathBuf>(["-c", "--rule-count"]).unwrap();

    let mut generator = SchemaGenerator::new(SchemaSettings::default());
    let table = RuleTable::new(Some(&mut generator));

    if let Some(rules_json_path) = rules_json_path {
        let rules_json_path = pwd.join(rules_json_path).canonicalize().unwrap();

        eprintln!("Rendering rules JSON blob...");
        let rules_json = oxlint::get_all_rules_json();
        fs::write(rules_json_path, rules_json).unwrap();
    }

    if let Some(rules_dir) = &rules_dir {
        eprintln!("Rendering rule doc pages...");
        let rules_dir = pwd.join(rules_dir);
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir).unwrap();
        }
        let rules_dir = rules_dir.canonicalize().unwrap();
        assert!(
            !rules_dir.is_file(),
            "Cannot write rule docs to a file. Please specify a directory."
        );

        write_rule_doc_pages(generator, &table, &rules_dir);
        write_version_data(&rules_dir, git_ref.unwrap_or("main".to_string()).as_str());
    }

    if let Some(rule_count_dir) = &rule_count_dir {
        eprintln!("Generating rule count data...");
        let rule_count_dir = pwd.join(rule_count_dir).canonicalize().unwrap();

        write_rule_count_data(&rule_count_dir, table.total);
    }

    eprintln!(
        "Done. Written rules to {}",
        rules_dir.as_ref().map_or_else(|| "none".to_string(), |p| format!("{}", p.display()))
    );
}

fn render_rule_doc_pages(
    g: SchemaGenerator,
    table: &RuleTable,
) -> impl Iterator<Item = (String, String, &RuleTableRow)> + '_ {
    let mut ctx = Context::new::<Oxlintrc>(g);
    table.sections.iter().flat_map(|section| &section.rows).map(move |rule| {
        let key = format!("{}/{}.md", rule.plugin, rule.name);
        let docs = ctx.render_rule_docs_page(rule).unwrap();
        (key, docs, rule)
    })
}

fn write_rule_doc_pages(g: SchemaGenerator, table: &RuleTable, outdir: &Path) {
    for (key, docs, _) in render_rule_doc_pages(g, table) {
        let page_path = outdir.join(&key);
        let plugin_path = page_path.parent().unwrap();
        fs::create_dir_all(plugin_path).unwrap();
        if page_path.exists() {
            fs::remove_file(&page_path).unwrap();
        }
        fs::write(&page_path, docs).unwrap();
    }
}

fn write_version_data(outdir: &Path, git_ref: &str) {
    let data = format!(r#"export default {{ load() {{ return "{git_ref}" }} }} "#);
    fs::write(outdir.join("version.data.js"), data).unwrap();
}

fn write_rule_count_data(outdir: &Path, rule_count: usize) {
    let data = format!(r"export default {{ load() {{ return {rule_count} }} }} ");
    fs::write(outdir.join("rule-count.data.js"), data).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::{SchemaGenerator, r#gen::SchemaSettings};

    // Create a snapshot containing all of the generated rule doc pages
    #[test]
    fn test_docs_rule_pages() {
        let mut generator = SchemaGenerator::new(SchemaSettings::default());
        let table = RuleTable::new(Some(&mut generator));
        let pages = render_rule_doc_pages(generator, &table).filter(|(_, _, rule)| {
            let name_and_plugin = format!("{}/{}", rule.plugin, rule.name);
            match name_and_plugin.as_str() {
                // Include only a few rules in the snapshot for brevity
                "eslint/no-unused-vars"
                | "import/namespace"
                | "jest/expect-expect"
                | "jsdoc/require-returns"
                | "jsx-a11y/aria-role"
                | "nextjs/no-duplicate-head"
                | "oxc/no-barrel-file"
                | "promise/no-callback-in-promise"
                | "react/rules-of-hooks"
                | "typescript/no-floating-promises"
                | "typescript/no-explicit-any"
                | "unicorn/prefer-array-find"
                | "vue/no-lifecycle-after-await" => true,
                _ => false,
            }
        });

        let mut snapshot = String::new();
        for (key, docs, _) in pages {
            snapshot.push_str("--- ");
            snapshot.push_str(&key);
            snapshot.push_str(" ---\n");
            snapshot.push_str(&docs);
            snapshot.push_str("\n\n");
        }

        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(snapshot);
        });
    }
}
