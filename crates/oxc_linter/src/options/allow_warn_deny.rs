use std::convert::From;

use oxc_diagnostics::{OxcDiagnostic, Severity};
use schemars::{schema::SchemaObject, JsonSchema};
use serde_json::{Number, Value};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AllowWarnDeny {
    Allow, // Off
    Warn,  // Warn
    Deny,  // Error
}

impl AllowWarnDeny {
    pub fn is_warn_deny(self) -> bool {
        self != Self::Allow
    }

    pub fn is_allow(self) -> bool {
        self == Self::Allow
    }
}

impl TryFrom<&str> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "allow" | "off" => Ok(Self::Allow),
            "deny" | "error" => Ok(Self::Deny),
            "warn" => Ok(Self::Warn),
            _ => Err(OxcDiagnostic::error(format!(
                r#"Failed to parse rule severity, expected one of "allow", "off", "deny", "error" or "warn", but got {s:?}"#
            ))),
        }
    }
}

impl TryFrom<&Value> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(value: &Value) -> Result<Self, OxcDiagnostic> {
        match value {
            Value::String(s) => Self::try_from(s.as_str()),
            Value::Number(n) => Self::try_from(n),
            _ => Err(OxcDiagnostic::error(format!(
                "Failed to parse rule severity, expected a string or a number, but got {value:?}"
            ))),
        }
    }
}

impl TryFrom<&Number> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(value: &Number) -> Result<Self, Self::Error> {
        match value.as_i64() {
            Some(0) => Ok(Self::Allow),
            Some(1) => Ok(Self::Warn),
            Some(2) => Ok(Self::Deny),
            _ => Err(OxcDiagnostic::error(format!(
                r#"Failed to parse rule severity, expected one of `0`, `1` or `2`, but got {value:?}"#
            ))),
        }
    }
}

impl JsonSchema for AllowWarnDeny {
    fn schema_name() -> String {
        "AllowWarnDeny".to_string()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "AllowWarnDeny".into()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut string_schema = <String as JsonSchema>::json_schema(gen).into_object();
        string_schema.enum_values =
            Some(vec!["allow".into(), "off".into(), "warn".into(), "error".into(), "deny".into()]);
        string_schema.metadata().description = Some(
            r#"Oxlint rule.
- "allow" or "off": Turn off the rule.
- "warn": Turn the rule on as a warning (doesn't affect exit code).
- "error" or "deny": Turn the rule on as an error (will exit with a failure code)."#
                .to_string(),
        );
        let mut int_schema = <u32 as JsonSchema>::json_schema(gen).into_object();
        int_schema.number().minimum = Some(0.0);
        int_schema.number().maximum = Some(2.0);
        int_schema.metadata().description = Some(
            "Oxlint rule.
    
- 0: Turn off the rule.
- 1: Turn the rule on as a warning (doesn't affect exit code).
- 2: Turn the rule on as an error (will exit with a failure code)."
                .to_string(),
        );

        let mut schema = SchemaObject::default();
        schema.subschemas().one_of = Some(vec![string_schema.into(), int_schema.into()]);

        schema.into()
    }
}

impl From<AllowWarnDeny> for Severity {
    fn from(value: AllowWarnDeny) -> Self {
        match value {
            AllowWarnDeny::Allow => Self::Advice,
            AllowWarnDeny::Warn => Self::Warning,
            AllowWarnDeny::Deny => Self::Error,
        }
    }
}
