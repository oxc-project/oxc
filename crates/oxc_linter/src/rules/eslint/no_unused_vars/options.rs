use std::{borrow::Cow, ops::Deref};

use oxc_diagnostics::OxcDiagnostic;
use regex::Regex;
use serde_json::Value;

/// See [ESLint - no-unused-vars config schema](https://github.com/eslint/eslint/blob/53b1ff047948e36682fade502c949f4e371e53cd/lib/rules/no-unused-vars.js#L61)
#[derive(Debug, Clone)]
#[must_use]
#[non_exhaustive]
pub struct NoUnusedVarsOptions {
    /// Controls how usage of a variable in the global scope is checked.
    ///
    /// This option has two settings:
    /// 1. `all` checks all variables for usage, including those in the global
    ///    scope. This is the default setting.
    /// 2. `local` checks only that locally-declared variables are used but will
    ///    allow global variables to be unused.
    pub vars: VarsOption,

    /// Specifies exceptions to this rule for unused variables. Variables whose
    /// names match this pattern will be ignored.
    ///
    /// By default, this pattern is `^_` unless options are configured with an
    /// object. In this case it will default to [`None`]. Note that this
    /// behavior deviates from both ESLint and TypeScript-ESLint, which never
    /// provide a default pattern.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option when the pattern is `^_`:
    /// ```javascript
    /// var _a = 10;
    /// var b = 10;
    /// console.log(b);
    /// ```
    pub vars_ignore_pattern: Option<Regex>,

    /// Controls how unused arguments are checked.
    ///
    /// This option has three settings:
    /// 1. `after-used` - Unused positional arguments that occur before the last
    ///    used argument will not be checked, but all named arguments and all
    ///    positional arguments after the last used argument will be checked.
    ///    This is the default setting.
    /// 2. `all` - All named arguments must be used.
    /// 3. `none` - Do not check arguments.
    pub args: ArgsOption,

    /// Specifies exceptions to this rule for unused arguments. Arguments whose
    /// names match this pattern will be ignored.
    ///
    /// By default, this pattern is `^_` unless options are configured with an
    /// object. In this case it will default to [`None`]. Note that this
    /// behavior deviates from both ESLint and TypeScript-ESLint, which never
    /// provide a default pattern.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option when the pattern is `^_`:
    ///
    /// ```javascript
    /// function foo(_a, b) {
    ///    console.log(b);
    /// }
    /// foo(1, 2);
    /// ```
    pub args_ignore_pattern: Option<Regex>,

    /// Using a Rest property it is possible to "omit" properties from an
    /// object, but by default the sibling properties are marked as "unused".
    /// With this option enabled the rest property's siblings are ignored.
    ///
    /// By default this option is `false`.
    ///
    /// ## Example
    /// Examples of **correct** code when this option is set to `true`:
    /// ```js
    /// // 'foo' and 'bar' were ignored because they have a rest property sibling.
    /// var { foo, ...coords } = data;
    ///
    /// var bar;
    /// ({ bar, ...coords } = data);
    /// ```
    pub ignore_rest_siblings: bool,

    /// Used for `catch` block validation.
    ///
    /// It has two settings:
    /// * `none` - do not check error objects. This is the default setting.
    /// * `all` - all named arguments must be used`.
    #[doc(hidden)]
    /// `none` corresponds to `false`, while `all` corresponds to `true`.
    pub caught_errors: CaughtErrors,

    /// Specifies exceptions to this rule for errors caught within a `catch` block.
    /// Variables declared within a `catch` block whose names match this pattern
    /// will be ignored.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code when the pattern is `^ignore`:
    ///
    /// ```javascript
    /// try {
    ///   // ...
    /// } catch (ignoreErr) {
    ///   console.error("Error caught in catch block");
    /// }
    /// ```
    pub caught_errors_ignore_pattern: Option<Regex>,

    /// This option specifies exceptions within destructuring patterns that will
    /// not be checked for usage. Variables declared within array destructuring
    /// whose names match this pattern will be ignored.
    ///
    /// By default this pattern is [`None`].
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option, when the pattern is `^_`:
    /// ```javascript
    /// const [a, _b, c] = ["a", "b", "c"];
    /// console.log(a + c);
    ///
    /// const { x: [_a, foo] } = bar;
    /// console.log(foo);
    ///
    /// let _m, n;
    /// foo.forEach(item => {
    ///     [_m, n] = item;
    ///     console.log(n);
    /// });
    /// ```
    pub destructured_array_ignore_pattern: Option<Regex>,

    /// The `ignoreClassWithStaticInitBlock` option is a boolean (default:
    /// `false`). Static initialization blocks allow you to initialize static
    /// variables and execute code during the evaluation of a class definition,
    /// meaning the static block code is executed without creating a new
    /// instance of the class. When set to true, this option ignores classes
    /// containing static initialization blocks.
    ///
    /// ## Example
    ///
    /// Examples of **incorrect** code for the `{ "ignoreClassWithStaticInitBlock": true }` option
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: ["error", { "ignoreClassWithStaticInitBlock": true }]*/
    ///
    /// class Foo {
    ///     static myProperty = "some string";
    ///     static mymethod() {
    ///         return "some string";
    ///     }
    /// }
    ///
    /// class Bar {
    ///     static {
    ///         let baz; // unused variable
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for the `{ "ignoreClassWithStaticInitBlock": true }` option
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: ["error", { "ignoreClassWithStaticInitBlock": true }]*/
    ///
    /// class Foo {
    ///     static {
    ///         let bar = "some string";
    ///
    ///         console.log(bar);
    ///     }
    /// }
    /// ```
    pub ignore_class_with_static_init_block: bool,

    /// The `reportUsedIgnorePattern` option is a boolean (default: `false`).
    /// Using this option will report variables that match any of the valid
    /// ignore pattern options (`varsIgnorePattern`, `argsIgnorePattern`,
    /// `caughtErrorsIgnorePattern`, or `destructuredArrayIgnorePattern`) if
    /// they have been used.
    ///
    /// ## Example
    ///
    /// Examples of **incorrect** code for the `{ "reportUsedIgnorePattern": true }` option:
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: ["error", { "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]*/
    ///
    /// var firstVarIgnored = 1;
    /// var secondVar = 2;
    /// console.log(firstVarIgnored, secondVar);
    /// ```
    ///
    /// Examples of **correct** code for the `{ "reportUsedIgnorePattern": true }` option:
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: ["error", { "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]*/
    ///
    /// var firstVar = 1;
    /// var secondVar = 2;
    /// console.log(firstVar, secondVar);
    /// ```
    pub report_used_ignore_pattern: bool,
}

impl Default for NoUnusedVarsOptions {
    fn default() -> Self {
        let underscore = Some(Regex::new("^_").unwrap());
        Self {
            vars: VarsOption::default(),
            vars_ignore_pattern: underscore.clone(),
            args: ArgsOption::default(),
            args_ignore_pattern: underscore,
            ignore_rest_siblings: false,
            caught_errors: CaughtErrors::default(),
            caught_errors_ignore_pattern: None,
            destructured_array_ignore_pattern: None,
            ignore_class_with_static_init_block: false,
            report_used_ignore_pattern: false,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum VarsOption {
    /// All variables are checked for usage, including those in the global scope.
    #[default]
    All,
    /// Checks only that locally-declared variables are used but will allow
    /// global variables to be unused.
    Local,
}
impl VarsOption {
    pub const fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ArgsOption {
    /// Unused positional arguments that occur before the last used argument
    /// will not be checked, but all named arguments and all positional
    /// arguments after the last used argument will be checked.
    #[default]
    AfterUsed,
    /// All named arguments must be used
    All,
    /// Do not check arguments
    None,
}
impl ArgsOption {
    #[inline]
    pub const fn is_after_used(&self) -> bool {
        matches!(self, Self::AfterUsed)
    }
    #[inline]
    pub const fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CaughtErrors(bool);

impl Default for CaughtErrors {
    fn default() -> Self {
        Self::all()
    }
}

impl CaughtErrors {
    pub const fn all() -> Self {
        Self(true)
    }
    pub const fn none() -> Self {
        Self(false)
    }
}

impl Deref for CaughtErrors {
    type Target = bool;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::Not for CaughtErrors {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

fn invalid_option_mismatch_error<E, A>(option_name: &str, expected: E, actual: A) -> OxcDiagnostic
where
    E: IntoIterator<Item = &'static str>,
    A: AsRef<str>,
{
    let expected = expected.into_iter();
    let initial_capacity = expected.size_hint().0 * 8;
    let expected =
        expected.fold(String::with_capacity(initial_capacity), |acc, s| acc + " or " + s);
    let actual = actual.as_ref();

    invalid_option_error(option_name, format!("Expected {expected}, got {actual}"))
}

fn invalid_option_error<M: Into<Cow<'static, str>>>(
    option_name: &str,
    message: M,
) -> OxcDiagnostic {
    let message = message.into();
    OxcDiagnostic::error(format!("Invalid '{option_name}' option for no-unused-vars: {message}"))
}

impl TryFrom<&String> for VarsOption {
    type Error = OxcDiagnostic;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(Self::All),
            "local" => Ok(Self::Local),
            v => Err(invalid_option_mismatch_error("vars", ["all", "local"], v)),
        }
    }
}

impl TryFrom<&Value> for VarsOption {
    type Error = OxcDiagnostic;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Self::try_from(s),
            _ => Err(invalid_option_error("vars", format!("Expected a string, got {value}"))),
        }
    }
}

impl TryFrom<&Value> for ArgsOption {
    type Error = OxcDiagnostic;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => match s.as_str() {
                "after-used" => Ok(Self::AfterUsed),
                "all" => Ok(Self::All),
                "none" => Ok(Self::None),
                s => Err(invalid_option_mismatch_error("args", ["after-used", "all", "none"], s)),
            },
            v => Err(invalid_option_error("args", format!("Expected a string, got {v}"))),
        }
    }
}

impl TryFrom<&String> for CaughtErrors {
    type Error = OxcDiagnostic;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(Self(true)),
            "none" => Ok(Self(false)),
            v => Err(invalid_option_mismatch_error("caughtErrors", ["all", "none"], v)),
        }
    }
}

impl From<bool> for CaughtErrors {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
impl TryFrom<&Value> for CaughtErrors {
    type Error = OxcDiagnostic;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Self::try_from(s),
            Value::Bool(b) => Ok(Self(*b)),
            v => Err(invalid_option_error("caughtErrors", format!("Expected a string, got {v}"))),
        }
    }
}

/// Parses a potential pattern into a [`Regex`] that accepts unicode characters.
fn parse_unicode_rule(value: Option<&Value>, name: &str) -> Option<Regex> {
    value
        .and_then(Value::as_str)
        .map(|pattern| regex::RegexBuilder::new(pattern).unicode(true).build())
        .transpose()
        .map_err(|err| panic!("Invalid '{name}' option for no-unused-vars: {err}"))
        .unwrap()
}
impl From<Value> for NoUnusedVarsOptions {
    fn from(value: Value) -> Self {
        let Some(config) = value.get(0) else { return Self::default() };
        match config {
            Value::String(vars) => {
                let vars: VarsOption = vars
                    .try_into()
                    .unwrap();
                Self { vars, ..Default::default() }
            }
            Value::Object(config) => {
                let vars = config
                    .get("vars")
                    .map(|vars| {
                        let vars: VarsOption = vars
                            .try_into()
                            .unwrap();
                        vars
                    })
                    .unwrap_or_default();

                // NOTE: when a configuration object is provided, do not provide
                // a default ignore pattern here. They've opted into configuring
                // this rule, and we'll give them full control over it.
                let vars_ignore_pattern: Option<Regex> =
                    parse_unicode_rule(config.get("varsIgnorePattern"), "varsIgnorePattern");

                let args: ArgsOption = config
                    .get("args")
                    .map(|args| {
                        let args: ArgsOption = args
                            .try_into()
                            .unwrap();
                        args
                    })
                    .unwrap_or_default();

                let args_ignore_pattern: Option<Regex> =
                    parse_unicode_rule(config.get("argsIgnorePattern"), "argsIgnorePattern");

                let caught_errors: CaughtErrors = config
                    .get("caughtErrors")
                    .map(|caught_errors| {
                        let caught_errors: CaughtErrors = caught_errors
                            .try_into()
                            .unwrap();
                        caught_errors
                    })
                    .unwrap_or_default();

                let caught_errors_ignore_pattern = parse_unicode_rule(
                    config.get("caughtErrorsIgnorePattern"),
                    "caughtErrorsIgnorePattern",
                );

                let destructured_array_ignore_pattern: Option<Regex> = parse_unicode_rule(
                    config.get("destructuredArrayIgnorePattern"),
                    "destructuredArrayIgnorePattern",
                );

                let ignore_rest_siblings: bool = config
                    .get("ignoreRestSiblings")
                    .map_or(Some(false), Value::as_bool)
                    .unwrap_or(false);

                let ignore_class_with_static_init_block: bool = config
                    .get("ignoreClassWithStaticInitBlock")
                    .map_or(Some(false), Value::as_bool)
                    .unwrap_or(false);

                let report_used_ignore_pattern: bool = config
                    .get("reportUsedIgnorePattern")
                    .map_or(Some(false), Value::as_bool)
                    .unwrap_or(false);

                Self {
                    vars,
                    vars_ignore_pattern,
                    args,
                    args_ignore_pattern,
                    ignore_rest_siblings,
                    caught_errors,
                    caught_errors_ignore_pattern,
                    destructured_array_ignore_pattern,
                    ignore_class_with_static_init_block,
                    report_used_ignore_pattern
                }
            }
            Value::Null => Self::default(),
            _ => panic!(
                "Invalid 'vars' option for no-unused-vars: Expected a string or an object, got {config}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_options_default() {
        let rule = NoUnusedVarsOptions::default();
        assert_eq!(rule.vars, VarsOption::All);
        assert!(rule.vars_ignore_pattern.is_some_and(|v| v.as_str() == "^_"));
        assert_eq!(rule.args, ArgsOption::AfterUsed);
        assert!(rule.args_ignore_pattern.is_some_and(|v| v.as_str() == "^_"));
        assert_eq!(rule.caught_errors, CaughtErrors::all());
        assert!(rule.caught_errors_ignore_pattern.is_none());
        assert!(rule.destructured_array_ignore_pattern.is_none());
        assert!(!rule.ignore_rest_siblings);
        assert!(!rule.ignore_class_with_static_init_block);
        assert!(!rule.report_used_ignore_pattern);
    }

    #[test]
    fn test_options_from_string() {
        let rule: NoUnusedVarsOptions = json!(["all"]).into();
        assert_eq!(rule.vars, VarsOption::All);

        let rule: NoUnusedVarsOptions = json!(["local"]).into();
        assert_eq!(rule.vars, VarsOption::Local);
    }

    #[test]
    fn test_options_from_object() {
        let rule: NoUnusedVarsOptions = json!([
            {
                "vars": "local",
                "varsIgnorePattern": "^_",
                "args": "all",
                "argsIgnorePattern": "^_",
                "caughtErrors": "all",
                "caughtErrorsIgnorePattern": "^_",
                "destructuredArrayIgnorePattern": "^_",
                "ignoreRestSiblings": true,
                "reportUsedIgnorePattern": true
            }
        ])
        .into();

        assert_eq!(rule.vars, VarsOption::Local);
        assert_eq!(rule.vars_ignore_pattern.unwrap().as_str(), "^_");
        assert_eq!(rule.args, ArgsOption::All);
        assert_eq!(rule.args_ignore_pattern.unwrap().as_str(), "^_");
        assert_eq!(rule.caught_errors, CaughtErrors::all());
        assert_eq!(rule.caught_errors_ignore_pattern.unwrap().as_str(), "^_");
        assert_eq!(rule.destructured_array_ignore_pattern.unwrap().as_str(), "^_");
        assert!(rule.ignore_rest_siblings);
        assert!(!rule.ignore_class_with_static_init_block);
        assert!(rule.report_used_ignore_pattern);
    }

    #[test]
    fn test_options_from_sparse_object() {
        let rule: NoUnusedVarsOptions = json!([
            {
                "argsIgnorePattern": "^_",
            }
        ])
        .into();
        // option object provided, no default varsIgnorePattern
        assert!(rule.vars_ignore_pattern.is_none());
        assert!(rule.args_ignore_pattern.unwrap().as_str() == "^_");

        let rule: NoUnusedVarsOptions = json!([
            {
                "varsIgnorePattern": "^_",
            }
        ])
        .into();

        // option object provided, no default argsIgnorePattern
        assert!(rule.vars_ignore_pattern.unwrap().as_str() == "^_");
        assert!(rule.args_ignore_pattern.is_none());
    }

    #[test]
    fn test_ignore_rest_siblings_only() {
        let rule: NoUnusedVarsOptions = json!([
            {
                "ignoreRestSiblings": true,
            }
        ])
        .into();
        assert!(rule.ignore_rest_siblings);
        // an options object is provided, so no default pattern is set.
        assert!(rule.vars_ignore_pattern.is_none());
    }

    #[test]
    fn test_options_from_null() {
        let opts = NoUnusedVarsOptions::from(json!(null));
        let default = NoUnusedVarsOptions::default();
        assert_eq!(opts.vars, default.vars);
        assert!(default.vars_ignore_pattern.unwrap().as_str() == "^_");

        assert_eq!(opts.args, default.args);
        assert!(default.args_ignore_pattern.unwrap().as_str() == "^_");

        assert_eq!(opts.caught_errors, default.caught_errors);
        assert!(opts.caught_errors_ignore_pattern.is_none());
        assert!(default.caught_errors_ignore_pattern.is_none());

        assert_eq!(opts.ignore_rest_siblings, default.ignore_rest_siblings);
    }

    #[test]
    fn test_parse_unicode_regex() {
        let pat = json!("^_");
        parse_unicode_rule(Some(&pat), "varsIgnorePattern")
            .expect("json strings should get parsed into a regex");
    }
}
