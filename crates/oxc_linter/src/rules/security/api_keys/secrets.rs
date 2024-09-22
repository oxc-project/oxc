mod aws_access_token;

use std::num::NonZeroU32;

use super::{Secret, SecretScanner, SecretScannerMeta, SecretViolation};

#[derive(Debug, Clone)]
pub enum SecretsEnum {
    AwsAccessKeyId(aws_access_token::AwsAccessToken),
}

impl SecretsEnum {
    pub fn rule_name(&self) -> &'static str {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.rule_name(),
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.message(),
        }
    }

    pub fn min_len(&self) -> NonZeroU32 {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.min_len(),
        }
    }

    pub fn max_len(&self) -> Option<NonZeroU32> {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.max_len(),
        }
    }

    pub fn min_entropy(&self) -> f32 {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.min_entropy(),
        }
    }

    pub fn verify(&self, violation: &mut SecretViolation<'_>) -> bool {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.verify(violation),
        }
    }

    pub fn detect(&self, candidate: &Secret<'_>) -> bool {
        match self {
            SecretsEnum::AwsAccessKeyId(rule) => rule.detect(candidate),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref ALL_RULES: Vec<SecretsEnum> = vec![
        SecretsEnum::AwsAccessKeyId(aws_access_token::AwsAccessToken),
    ];
}
