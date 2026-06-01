#![expect(clippy::print_stdout)]
//! # JSON Formatter Example
//!
//! This example demonstrates how to use the Oxc JSON formatter to format JSON / JSONC / JSON5 code.
//!
//! Handy for ad-hoc Prettier-compatibility checks: feed the same input to both
//! `prettier` and this example, then diff the outputs.
//!
//! ## Usage
//!
//! Create a `test.json` file and run:
//! ```bash
//! cargo run -p oxc_formatter_json --example formatter [filename]
//! cargo run -p oxc_formatter_json --example formatter -- --print-width 100 [filename]
//! cargo run -p oxc_formatter_json --example formatter -- --variant json5 [filename]
//! cargo run -p oxc_formatter_json --example formatter -- --diff [filename]
//! ```
//!
//! The parser variant is inferred from the file extension (`.jsonc`, `.json5`),
//! and can be overridden with `--variant <json|jsonc|json5|json-stringify>`.

use std::{fs, path::Path};

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_formatter_core::LineWidth;
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_tasks_common::print_diff_in_terminal;

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    // Show diff between original and formatted code
    let show_diff = args.contains("--diff");
    let print_width = args.opt_value_from_str::<&'static str, u16>("--print-width").unwrap_or(None);
    let variant_arg: Option<String> = args.opt_value_from_str("--variant").unwrap();
    let name = args.free_from_str().unwrap_or_else(|_| "test.json".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;

    // `--variant` wins; then `package.json` is always `json-stringify` (like Prettier);
    // otherwise infer from the file extension; default to `json`.
    let file_name = path.file_name().and_then(|name| name.to_str());
    let ext = path.extension().and_then(|ext| ext.to_str());
    let variant = if variant_arg.is_none() && file_name == Some("package.json") {
        JsonVariant::JsonStringify
    } else {
        match variant_arg.as_deref().or(ext) {
            Some("jsonc") => JsonVariant::Jsonc,
            Some("json5") => JsonVariant::Json5,
            Some("json-stringify") => JsonVariant::JsonStringify,
            _ => JsonVariant::Json,
        }
    };

    let line_width = match print_width {
        Some(width) => LineWidth::try_from(width).unwrap(),
        None => LineWidth::try_from(80).unwrap(),
    };
    let options = JsonFormatOptions { variant, line_width, ..Default::default() };

    let allocator = Allocator::new();
    let formatted = match oxc_formatter_json::format(&allocator, &source_text, options) {
        Ok(formatted) => formatted,
        Err(error) => {
            println!("{error}");
            return Err("Parsed with Errors.".to_string());
        }
    };

    let formatted_code = formatted.print().unwrap().into_code();

    if show_diff {
        // Compare formatted output to original input
        if source_text == formatted_code {
            println!("{formatted_code}");
        } else {
            print_diff_in_terminal(&source_text, &formatted_code);
        }
    } else {
        println!("{formatted_code}");
    }

    Ok(())
}
