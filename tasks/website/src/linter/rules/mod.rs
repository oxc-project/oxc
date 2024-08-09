mod doc_page;
mod html;
mod table;

use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
    process,
};

use doc_page::render_rule_docs_page;
use oxc_linter::table::RuleTable;
use pico_args::Arguments;
use table::render_rules_table;

const HELP: &str = "
usage: linter-rules [args]

Arguments:
    -t,--table <path>     Path to file where rule markdown table will be saved.
    -r,--rule-docs <path> Path to directory where rule doc pages will be saved.
                          A directory will be created if one doesn't exist.
    -h,--help             Show this help message.

";

/// `cargo run -p website linter-rules --table
/// /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/generated-rules.md
/// --rule-docs /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/rules
/// `
/// <https://oxc.rs/docs/guide/usage/linter/rules.html>
pub fn print_rules(mut args: Arguments) {
    let pwd = PathBuf::from(env::var("PWD").unwrap());
    if args.contains(["-h", "--help"]) {
        println!("{HELP}");
        return;
    }

    let table = RuleTable::new();
    let table_path = args.opt_value_from_str::<_, PathBuf>(["-t", "--table"]).unwrap();
    let rules_dir = args.opt_value_from_str::<_, PathBuf>(["-r", "--rule-docs"]).unwrap();

    let (prefix, root) = rules_dir.as_ref().and_then(|p| p.as_os_str().to_str()).map_or(
        (Cow::Borrowed(""), None),
        |p| {
            if p.contains("src/docs") {
                let split = p.split("src/docs").collect::<Vec<_>>();
                assert!(split.len() > 1);
                let root = split[0];
                let root = pwd.join(root).canonicalize().unwrap();
                let prefix = Cow::Owned("/docs".to_string() + split.last().unwrap());
                (prefix, Some(root))
            } else {
                (Cow::Borrowed(p), None)
            }
        },
    );

    if let Some(table_path) = table_path {
        let table_path = pwd.join(table_path).canonicalize().unwrap();

        println!("Rendering rules table...");
        let rules_table = render_rules_table(&table, prefix.as_ref());
        fs::write(table_path, rules_table).unwrap();
    }

    if let Some(rules_dir) = rules_dir {
        println!("Rendering rule doc pages...");
        let rules_dir = pwd.join(rules_dir);
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir).unwrap();
        }
        let rules_dir = rules_dir.canonicalize().unwrap();
        assert!(
            !rules_dir.is_file(),
            "Cannot write rule docs to a file. Please specify a directory."
        );
        write_rule_doc_pages(&table, &rules_dir);

        // auto-fix code and natural language issues
        if let Some(root) = root {
            println!("Formatting rule doc pages...");
            prettier(&root, &rules_dir);
            println!("Fixing textlint issues...");
            textlint(&root);
        }
    }

    println!("Done.");
}

fn write_rule_doc_pages(table: &RuleTable, outdir: &Path) {
    for rule in table.sections.iter().flat_map(|section| &section.rows) {
        let plugin_path = outdir.join(&rule.plugin);
        fs::create_dir_all(&plugin_path).unwrap();
        let page_path = plugin_path.join(format!("{}.md", rule.name));
        println!("{}", page_path.display());
        let docs = render_rule_docs_page(rule).unwrap();
        fs::write(&page_path, docs).unwrap();
    }
}

/// Run prettier and fix style issues in generated rule doc pages.
fn prettier(website_root: &Path, rule_docs_path: &Path) {
    assert!(rule_docs_path.is_dir(), "Rule docs path must be a directory.");
    assert!(rule_docs_path.is_absolute(), "Rule docs path must be an absolute path.");
    let relative_path = rule_docs_path.strip_prefix(website_root).unwrap();
    let path_str =
        relative_path.to_str().expect("Invalid rule docs path: could not convert to str");
    let generated_md_glob = format!("{path_str}/**/*.md");

    process::Command::new("pnpm")
        .current_dir(website_root)
        .args(["run", "fmt", "--write", &generated_md_glob])
        .status()
        .unwrap();
}

/// Run textlint and fix any issues it finds.
fn textlint(website_root: &Path) {
    assert!(website_root.is_dir(), "Rule docs path must be a directory.");
    process::Command::new("pnpm")
        .current_dir(website_root)
        .args(["run", "textlint:fix"])
        .status()
        .unwrap();
}
