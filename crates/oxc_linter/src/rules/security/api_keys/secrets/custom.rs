use std::{borrow::Cow, num::NonZeroU32};

use regex::Regex;

use oxc_span::CompactStr;

use super::{Secret, SecretScanner, SecretScannerMeta};

#[derive(Debug, Clone)]
pub struct CustomSecret {
    pub(crate) rule_name: CompactStr,
    pub(crate) message: CompactStr,
    pub(crate) entropy: f32,
    pub(crate) min_len: NonZeroU32,
    pub(crate) max_len: Option<NonZeroU32>,
    pub(crate) pattern: Regex,
}

// // This default impl doesn't make logical sense, but it's useful for building structs out of
// // partial configs.
// impl Default for CustomSecret {
//     fn default() -> Self {
//         Self {
//             rule_name:
//             entropy: DEFAULT_MIN_ENTROPY,
//             min_len: DEFAULT_MIN_LEN,
//             message: String,
//             max_len: None,
//             pattern: Regex::new("").unwrap(),
//         }
//     }
// }

impl SecretScannerMeta for CustomSecret {
    fn rule_name(&self) -> Cow<'static, str> {
        self.rule_name.clone().into()
    }
    fn message(&self) -> Cow<'static, str> {
        self.message.clone().into()
    }
    fn min_len(&self) -> NonZeroU32 {
        self.min_len
    }
    fn max_len(&self) -> Option<NonZeroU32> {
        self.max_len
    }
    fn min_entropy(&self) -> f32 {
        self.entropy
    }
}

impl SecretScanner for CustomSecret {
    fn detect(&self, candidate: &Secret<'_>) -> bool {
        self.pattern.is_match(candidate)
    }
}
