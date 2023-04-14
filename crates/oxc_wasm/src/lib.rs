mod driver;
mod utils;

use driver::Driver;
use oxc_ast::SourceType;
use serde::{Deserialize, Serialize};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "javascript")]
    JavaScript,
    #[serde(rename = "typescript")]
    TypeScript,
}

impl Default for Language {
    fn default() -> Self {
        Self::TypeScript
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    pub language: Option<Language>,

    pub jsx: Option<bool>,

    pub eslintrc: Option<String>,
}

#[wasm_bindgen]
#[allow(deprecated)]
#[must_use]
pub fn parse(text: &str, js_options: &JsValue) -> JsValue {
    set_panic_hook();
    let options: Options = js_options.into_serde().unwrap_or_default();
    let path_str = format!(
        "test.{}{}",
        if matches!(options.language, Some(Language::TypeScript)) { "ts" } else { "js" },
        if options.jsx.unwrap_or_default() { "x" } else { "" }
    );

    let source_type = SourceType::from_path(&path_str).unwrap_or_default();

    let driver = Driver::new();

    driver.run(&path_str, text, source_type, &options.eslintrc.unwrap_or_default())
}
