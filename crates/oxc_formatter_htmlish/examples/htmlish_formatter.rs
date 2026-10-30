#![expect(clippy::print_stdout)]
//! Format a file and print the result.
//!
//! ```bash
//! cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]
//! cargo run -p oxc_formatter_htmlish --example htmlish_formatter -- --print-width 100 [filename]
//! cargo run -p oxc_formatter_htmlish --example htmlish_formatter -- --use-tabs --bracket-same-line \
//!   --single-attribute-per-line --html-whitespace-sensitivity ignore [filename]
//! DUMP_IR=1 cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]
//! ```

use std::fs;

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_formatter_core::{IndentStyle, LineWidth};
use oxc_formatter_htmlish::{HtmlishFormatOptions, HtmlishVariant, WhitespaceSensitivity};

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let print_width = args.opt_value_from_str::<&'static str, u16>("--print-width").unwrap_or(None);
    let use_tabs = args.contains("--use-tabs");
    let bracket_same_line = args.contains("--bracket-same-line");
    let single_attribute_per_line = args.contains("--single-attribute-per-line");
    let whitespace_sensitivity = match args
        .opt_value_from_str::<&'static str, String>("--html-whitespace-sensitivity")
        .unwrap_or(None)
        .as_deref()
    {
        Some("strict") => WhitespaceSensitivity::Strict,
        Some("ignore") => WhitespaceSensitivity::Ignore,
        _ => WhitespaceSensitivity::Css,
    };
    let name = args.free_from_str().unwrap_or_else(|_| "test.html".to_string());
    let source_text = fs::read_to_string(&name).map_err(|_| format!("Missing '{name}'"))?;
    let variant = match std::path::Path::new(&name).extension().and_then(|ext| ext.to_str()) {
        Some("vue") => HtmlishVariant::Vue,
        Some("xhtml" | "xht") => HtmlishVariant::Xhtml,
        _ if name.ends_with(".component.html") => HtmlishVariant::Angular,
        _ => HtmlishVariant::Html,
    };
    let options = HtmlishFormatOptions {
        line_width: LineWidth::try_from(print_width.unwrap_or(80)).unwrap(),
        indent_style: if use_tabs { IndentStyle::Tab } else { IndentStyle::Space },
        whitespace_sensitivity,
        bracket_same_line,
        single_attribute_per_line,
        variant,
        ..Default::default()
    };

    let allocator = Allocator::new();
    let formatted = oxc_formatter_htmlish::format(&allocator, &source_text, options)
        .map_err(|error| error.to_string())?;
    if std::env::var("DUMP_IR").is_ok() {
        println!("{:#?}", formatted.document());
    }
    print!("{}", formatted.print().map_err(|error| error.to_string())?.into_code());
    Ok(())
}
