mod aws_access_token;
mod custom;

use std::{borrow::Cow, num::NonZeroU32};

use super::{Secret, SecretScanner, SecretScannerMeta, SecretViolation};

pub use custom::CustomSecret;

#[derive(Debug, Clone)]
pub enum SecretsEnum {
    AwsAccessKeyId(aws_access_token::AwsAccessToken),
    Custom(custom::CustomSecret),
}

impl SecretsEnum {
    pub fn rule_name(&self) -> Cow<'static, str> {
        match self {
            Self::AwsAccessKeyId(rule) => rule.rule_name(),
            Self::Custom(rule) => rule.rule_name(),
        }
    }

    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::AwsAccessKeyId(rule) => rule.message(),
            Self::Custom(rule) => rule.message(),
        }
    }

    pub fn min_len(&self) -> NonZeroU32 {
        match self {
            Self::AwsAccessKeyId(rule) => rule.min_len(),
            Self::Custom(rule) => rule.min_len(),
        }
    }

    pub fn max_len(&self) -> Option<NonZeroU32> {
        match self {
            Self::AwsAccessKeyId(rule) => rule.max_len(),
            Self::Custom(rule) => rule.max_len(),
        }
    }

    pub fn min_entropy(&self) -> f32 {
        match self {
            Self::AwsAccessKeyId(rule) => rule.min_entropy(),
            Self::Custom(rule) => rule.min_entropy(),
        }
    }

    pub fn verify(&self, violation: &mut SecretViolation<'_>) -> bool {
        match self {
            Self::AwsAccessKeyId(rule) => rule.verify(violation),
            Self::Custom(rule) => rule.verify(violation),
        }
    }

    pub fn detect(&self, candidate: &Secret<'_>) -> bool {
        match self {
            Self::AwsAccessKeyId(rule) => rule.detect(candidate),
            Self::Custom(rule) => rule.detect(candidate),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref ALL_RULES: Vec<SecretsEnum> = vec![
        SecretsEnum::AwsAccessKeyId(aws_access_token::AwsAccessToken),
    ];
}
