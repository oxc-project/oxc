#![expect(clippy::print_stdout)]
//! # YAML Formatter Example
//!
//! Handy for ad-hoc Prettier-compatibility checks: feed the same input to both
//! `prettier` and this example, then diff the outputs.
//!
//! ## Usage
//!
//! Create a `test.yaml` file and run:
//! ```bash
//! cargo run -p oxc_formatter_yaml --example yaml_formatter [filename]
//! cargo run -p oxc_formatter_yaml --example yaml_formatter -- --print-width 100 [filename]
//! cargo run -p oxc_formatter_yaml --example yaml_formatter -- --diff [filename]
//! ```

use std::fs;

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_formatter_core::LineWidth;
use oxc_formatter_yaml::YamlFormatOptions;
use oxc_tasks_common::print_diff_in_terminal;

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    // Show diff between original and formatted code
    let show_diff = args.contains("--diff");
    let print_width = args.opt_value_from_str::<&'static str, u16>("--print-width").unwrap_or(None);
    let prose_wrap =
        args.opt_value_from_str::<&'static str, String>("--prose-wrap").unwrap_or(None);
    let name = args.free_from_str().unwrap_or_else(|_| "test.yaml".to_string());

    let source_text = fs::read_to_string(&name).map_err(|_| format!("Missing '{name}'"))?;

    let line_width = match print_width {
        Some(width) => LineWidth::try_from(width).unwrap(),
        None => LineWidth::try_from(80).unwrap(),
    };
    let prose_wrap = match prose_wrap.as_deref() {
        Some("always") => oxc_formatter_yaml::ProseWrap::Always,
        Some("never") => oxc_formatter_yaml::ProseWrap::Never,
        _ => oxc_formatter_yaml::ProseWrap::Preserve,
    };
    let options = YamlFormatOptions { line_width, prose_wrap, ..Default::default() };

    let allocator = Allocator::new();
    let formatted = match oxc_formatter_yaml::format(&allocator, &source_text, options) {
        Ok(formatted) => formatted,
        Err(error) => {
            println!("{error:?}");
            return Err("Parsed with Errors.".to_string());
        }
    };

    if std::env::var("DUMP_IR").is_ok() {
        println!("{:#?}", formatted.document());
    }

    let formatted_code = formatted.print().unwrap().into_code();

    if show_diff {
        if source_text == formatted_code {
            print!("{formatted_code}");
        } else {
            print_diff_in_terminal(&source_text, &formatted_code);
        }
    } else {
        print!("{formatted_code}");
    }

    Ok(())
}
