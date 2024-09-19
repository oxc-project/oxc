use std::sync::{Arc, OnceLock};

use markdown::{to_html_with_options, Options};
use oxc_allocator::Allocator;
use oxc_diagnostics::NamedSource;
use oxc_linter::table::RuleTable;
use oxc_parser::Parser;
use oxc_span::SourceType;
use scraper::{ElementRef, Html, Selector};

use super::{render_rule_docs_page, render_rules_table};

static TABLE: OnceLock<RuleTable> = OnceLock::new();

fn table() -> &'static RuleTable {
    TABLE.get_or_init(RuleTable::new)
}

fn parse(filename: &str, jsx: &str) -> Result<(), String> {
    let filename = format!("{filename}.tsx");
    let source_type = SourceType::from_path(&filename).unwrap();
    parse_type(&filename, jsx, source_type)
}

fn parse_type(filename: &str, source_text: &str, source_type: SourceType) -> Result<(), String> {
    let alloc = Allocator::default();
    let ret = Parser::new(&alloc, source_text, source_type).parse();

    if ret.errors.is_empty() {
        Ok(())
    } else {
        let num_errs = ret.errors.len();
        let source = Arc::new(NamedSource::new(filename, source_text.to_string()));
        ret.errors
            .into_iter()
            .map(|e| e.with_source_code(Arc::clone(&source)))
            .for_each(|e| println!("{e:?}"));
        Err(format!("{num_errs} errors occurred while parsing {filename}.jsx"))
    }
}

#[test]
fn test_rules_table() {
    const PREFIX: &str = "/docs/guide/usage/linter/rules";
    let options = Options::gfm();
    let rendered_table = render_rules_table(table(), PREFIX);
    let rendered_html = to_html_with_options(&rendered_table, &options).unwrap();
    assert!(rendered_html.contains("<table>"));
    let html = Html::parse_fragment(&rendered_html);
    assert!(html.errors.is_empty(), "{:#?}", html.errors);
    let jsx = format!("const Table = () => <>{rendered_html}</>");
    parse("rules-table", &jsx).unwrap();
}

#[test]
fn test_doc_pages() {
    let mut options = Options::gfm();
    options.compile.allow_dangerous_html = true;

    for section in &table().sections {
        let category = section.category;
        let code = Selector::parse("code").unwrap();

        for row in &section.rows {
            let filename = format!("{category}/{}/{}", row.plugin, row.name);
            let docs = render_rule_docs_page(row).unwrap();
            let docs = to_html_with_options(&docs, &options).unwrap();
            let docs = if let Some(end_of_autogen_comment) = docs.find("-->") {
                &docs[end_of_autogen_comment + 4..]
            } else {
                &docs
            };

            // ensure code examples are valid
            {
                let html = Html::parse_fragment(docs);
                for code_el in html.select(&code) {
                    let inner = code_el.inner_html();
                    let inner =
                        inner.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&");
                    let filename = filename.clone() + "/code-snippet";
                    let Some(source_type) = source_type_from_code_element(code_el) else {
                        continue;
                    };
                    if row.plugin == "nextjs" {
                        // Almost all Next.js rules are missing docs.
                        continue;
                    }
                    assert!(
                        !inner.trim().is_empty(),
                        "Rule '{}' has an empty code snippet",
                        row.name
                    );
                    parse_type(&filename, &inner, source_type).unwrap();
                }
            }
        }
    }
}

fn source_type_from_code_element(code: ElementRef) -> Option<SourceType> {
    let class = code.attr("class")?;
    let maybe_class = class.split('-').collect::<Vec<_>>();
    let ["language", lang] = maybe_class.as_slice() else {
        return None;
    };

    match *lang {
        "javascript" | "js" => Some(SourceType::default()),
        "typescript" | "ts" => Some(SourceType::default().with_typescript(true)),
        "tsx" => Some(SourceType::tsx()),
        // FIXME: lots of jsx examples are usefully succinct but not valid JSX.
        // "jsx" => Some(SourceType::default().with_jsx(true).with_always_strict(true)),
        _ => None,
    }
}
