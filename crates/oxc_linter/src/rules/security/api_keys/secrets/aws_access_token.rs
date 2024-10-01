use std::num::NonZeroU32;

use oxc_macros::declare_oxc_secret;
use phf::{map::Map, phf_map};

use super::{Secret, SecretScanner, SecretViolation};

/// See: <https://swisskyrepo.github.io/InternalAllTheThings/cloud/aws/aws-access-token/#access-key-id-secret>
#[derive(Debug, Default, Clone)]
pub struct AwsAccessToken;

declare_oxc_secret! {
    AwsAccessToken,
    "Detected an AWS Access Key ID, which may lead to unauthorized access to AWS resources.",
    entropy = 2.0,
    min_len = 20,
    max_len = 20,
}

impl SecretScanner for AwsAccessToken {
    // '''(?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA|<other-keys-below>)[A-Z0-9]{16}'''
    fn detect(&self, candidate: &Secret<'_>) -> bool {
        if !candidate.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return false;
        }

        let prefix = &candidate[..4];
        AWS_TOKEN_PREFIXES.contains_key(prefix) || &prefix[0..3] == "A3T"
    }

    fn verify(&self, violation: &mut SecretViolation<'_>) -> bool {
        let prefix = &violation[..4];
        // Detect false positives on DNA sequences
        if prefix == "ACCA" && violation.chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }

        let name = AWS_TOKEN_PREFIXES.get(prefix).copied().unwrap_or("AWS access token");
        let a_or_an = match name.chars().next().unwrap() {
            'A' | 'E' | 'I' | 'O' | 'U' => "an",
            _ => "a",
        };

        violation.set_message(format!(
            "Detected {a_or_an} {name}, which may lead to unauthorized access to AWS resources."
        ));

        true
    }
}

/// List taken from:
/// <https://swisskyrepo.github.io/InternalAllTheThings/cloud/aws/aws-access-token/#access-key-id-secret>
static AWS_TOKEN_PREFIXES: Map<&'static str, &'static str> = phf_map! {
    "ABIA" => "AWS STS service bearer token",
    "ACCA" => "AWS Context-specific credential",
    "AGPA" => "AWS User Group ID",
    "AIDA" => "AWS IAM User ID",
    "AIPA" => "Amazon EC2 instance profile",
    "AKIA" => "AWS Access Key ID",
    "ANPA" => "managed AWS Policy ID",
    "ANVA" => "managed AWS Policy Version ID",
    "APKA" => "AWS Public key",
    "AROA" => "AWS Role",
    "ASCA" => "AWS Certificate",
    "ASIA" => "temporary (AWS STS) Access Key",
};

#[test]
fn test() {
    use crate::{rules::ApiKeys, tester::Tester, RuleMeta};

    let pass = vec![
        "let x = ''",
        "let x = 'AKIA'",
        "let not_a_key = 'abcdabcdabcdabcdabcd' ", // no prefix, has lowercase
        "let not_a_key = 'AKIA ABCD1099FAM9KEY' ", // whitespace
        "let not_a_key = 'AKIA-ABCD1099FAM9KEY' ", // special characters
        "let not_a_key = 'AKIA_ABCD1099FAM9KEY' ", // special characters
        "let not_a_key = 'AKIA%ABCD1099FAM9KEY' ", // special characters
        "let not_a_key = 'AKIA$ABCD1099FAM9KEY' ", // special characters
        "let not_a_key = 'AKIAAABcD1099FAM9KEY' ", // has lowercase
        "let not_a_key = 'AKIAAABCD1099FAM9KEY9'", // too long
        "let dna = 'ACCATGGCTACCGCTGTGCT'       ", // DNA sequence
    ];

    let fail = vec![
        r#"let key = "AKIAAABCD1099FAM9KEY""#,
        "let key = `AKIAAABCD1099FAM9KEY`", // no-expression template literal
        "let key = 'ABIAAABCD1099FAM9KEY'",
        "let key = 'ACCAAABCD1099FAM9KEY'",
        "let key = 'AKIAAABCD1099FAM9KEY'",
        "let key = 'AKIAAABCD1099FAM9KEY'",
        "let key = 'AKIAAABCD1099FAM9KEY'",
    ];

    Tester::new(ApiKeys::NAME, pass, fail)
        .with_snapshot_suffix("aws_access_token")
        .test_and_snapshot();
}
