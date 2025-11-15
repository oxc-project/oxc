use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::TypeOrValueSpecifier,
};

#[derive(Debug, Default, Clone)]
pub struct NoFloatingPromises(Box<NoFloatingPromisesConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoFloatingPromisesConfig {
    /// Allows specific calls to be ignored, specified as type or value specifiers.
    pub allow_for_known_safe_calls: Vec<TypeOrValueSpecifier>,
    /// Allows specific Promise types to be ignored, specified as type or value specifiers.
    pub allow_for_known_safe_promises: Vec<TypeOrValueSpecifier>,
    /// Check for thenable objects that are not necessarily Promises.
    pub check_thenables: bool,
    /// Ignore immediately invoked function expressions (IIFEs).
    #[serde(rename = "ignoreIIFE")]
    pub ignore_iife: bool,
    /// Ignore Promises that are void expressions.
    pub ignore_void: bool,
}

impl Default for NoFloatingPromisesConfig {
    fn default() -> Self {
        Self {
            allow_for_known_safe_calls: Vec::new(),
            allow_for_known_safe_promises: Vec::new(),
            check_thenables: false,
            ignore_iife: false,
            ignore_void: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows "floating" Promises in TypeScript code, which is a Promise that is created without any code to handle its resolution or rejection.
    ///
    /// This rule will report Promise-valued statements that are not treated in one of the following ways:
    ///
    /// - Calling its `.then()` with two arguments
    /// - Calling its `.catch()` with one argument
    /// - `await`ing it
    /// - `return`ing it
    /// - `void`ing it
    ///
    /// This rule also reports when an Array containing Promises is created and not properly handled. The main way to resolve this is by using one of the Promise concurrency methods to create a single Promise, then handling that according to the procedure above. These methods include:
    ///
    /// - `Promise.all()`
    /// - `Promise.allSettled()`
    /// - `Promise.any()`
    /// - `Promise.race()`
    ///
    /// ### Why is this bad?
    ///
    /// Floating Promises can cause several issues, such as improperly sequenced operations, ignored Promise rejections, and more.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    /// returnsPromise().then(() => {});
    ///
    /// Promise.reject('value').catch();
    ///
    /// Promise.reject('value').finally();
    ///
    /// [1, 2, 3].map(async x => x + 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// await promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    ///
    /// void returnsPromise();
    ///
    /// returnsPromise().then(
    ///   () => {},
    ///   () => {},
    /// );
    ///
    /// Promise.reject('value').catch(() => {});
    ///
    /// await Promise.reject('value').finally(() => {});
    ///
    /// await Promise.all([1, 2, 3].map(async x => x + 1));
    /// ```
    NoFloatingPromises(tsgolint),
    typescript,
    correctness,
    pending,
    config = NoFloatingPromisesConfig,
);

impl Rule for NoFloatingPromises {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoFloatingPromisesConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let rule = NoFloatingPromises::default();
        let config = rule.to_configuration().unwrap().unwrap();

        // Verify the default values
        assert_eq!(config["allowForKnownSafeCalls"], json!([]));
        assert_eq!(config["allowForKnownSafePromises"], json!([]));
        assert_eq!(config["checkThenables"], json!(false));
        assert_eq!(config["ignoreIIFE"], json!(false));
        assert_eq!(config["ignoreVoid"], json!(true));
    }

    #[test]
    fn test_from_configuration() {
        let config_value = json!([{
            "allowForKnownSafeCalls": [{"from": "package", "name": "foo", "package": "some-package"}],
            "checkThenables": true,
            "ignoreVoid": false
        }]);

        let rule = NoFloatingPromises::from_configuration(config_value);

        assert!(rule.0.check_thenables);
        assert!(!rule.0.ignore_void);
        assert_eq!(rule.0.allow_for_known_safe_calls.len(), 1);
    }

    #[test]
    fn test_round_trip() {
        let original_config = json!([{
            "allowForKnownSafeCalls": [{"from": "package", "name": "bar", "package": "test-pkg"}],
            "allowForKnownSafePromises": [{"from": "lib", "name": "Promise"}],
            "checkThenables": true,
            "ignoreIIFE": true,
            "ignoreVoid": false
        }]);

        let rule = NoFloatingPromises::from_configuration(original_config);
        let serialized = rule.to_configuration().unwrap().unwrap();

        // Verify all fields are present in serialized output
        assert_eq!(
            serialized["allowForKnownSafeCalls"],
            json!([{"from": "package", "name": "bar", "package": "test-pkg"}])
        );
        assert_eq!(
            serialized["allowForKnownSafePromises"],
            json!([{"from": "lib", "name": "Promise"}])
        );
        assert_eq!(serialized["checkThenables"], json!(true));
        assert_eq!(serialized["ignoreIIFE"], json!(true));
        assert_eq!(serialized["ignoreVoid"], json!(false));
    }

    #[test]
    fn test_all_specifier_types() {
        let config_value = json!([{
            "allowForKnownSafeCalls": [
                "SomeType",  // string specifier
                {"from": "file", "name": "MyType", "path": "./types.ts"},  // file specifier with path
                {"from": "file", "name": ["Type1", "Type2"]},  // file specifier with multiple names
                {"from": "lib", "name": "Promise"},  // lib specifier
                {"from": "package", "name": "Observable", "package": "rxjs"}  // package specifier
            ],
            "checkThenables": false,
            "ignoreVoid": true
        }]);

        let rule = NoFloatingPromises::from_configuration(config_value);

        assert_eq!(rule.0.allow_for_known_safe_calls.len(), 5);
        assert!(!rule.0.check_thenables);
        assert!(rule.0.ignore_void);

        // Verify serialization preserves all types
        let serialized = rule.to_configuration().unwrap().unwrap();
        assert_eq!(serialized["allowForKnownSafeCalls"].as_array().unwrap().len(), 5);
    }
}
