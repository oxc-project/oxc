mod unicode_sets_regex;

use serde::Deserialize;

use crate::impl_preset_transformation;
use crate::options::default_as_true;
use crate::preset_plugin::BoxedTransformation;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2024Options {
    /// https://babeljs.io/docs/babel-plugin-transform-unicode-sets-regex
    #[serde(default = "default_as_true")]
    pub unicode_sets_regex: bool,
}

impl Default for Es2024Options {
    fn default() -> Self {
        Self { unicode_sets_regex: default_as_true() }
    }
}

#[allow(dead_code)]
pub struct Es2024 {
    options: Es2024Options,
    plugins: Vec<BoxedTransformation>,
}

impl Es2024 {
    pub fn new(options: Es2024Options) -> Self {
        let mut plugins: Vec<BoxedTransformation> = vec![];

        // Ordered from most complex to least complex!
        if options.unicode_sets_regex {
            plugins.push(Box::new(unicode_sets_regex::UnicodeSetsRegex));
        }

        Self { options, plugins }
    }
}

impl_preset_transformation!(Es2024);
