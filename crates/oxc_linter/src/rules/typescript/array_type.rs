use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct ArrayType(Box<ArrayTypeConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Require consistently using either `T[]` or `Array<T>` for arrays.
    ///
    /// ### Why is this bad?
    /// Using the `Array` type directly is not idiomatic. Instead, use the array type `T[]` or `Array<T>`.
    ///
    /// ### Example
    /// ```typescript
    /// const arr: Array<number> = new Array<number>();
    /// const arr: number[] = new Array<number>();
    /// ```
    ArrayType,
    style,
);

#[derive(Debug, Default, Clone)]
pub struct ArrayTypeConfig {
    // The array type expected for mutable cases.
    default: ArrayOption,
    // The array type expected for readonly cases. If omitted, the value for `default` will be used.
    readonly: Option<ArrayOption>,
}

impl std::ops::Deref for ArrayType {
    type Target = ArrayTypeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Debug, Default, Clone)]
pub enum ArrayOption {
    ArraySimple,
    #[default]
    Array,
    Generic,
}

impl Rule for ArrayType {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(ArrayTypeConfig {
            default: value
            .get(0)
            .and_then(|v| v.get("default"))
            .and_then(serde_json::Value::as_str)
            .map_or_else(|| ArrayOption::Array, |s| match s {
                "array" => ArrayOption::Array,
                "generic" => ArrayOption::Generic,
                _ => ArrayOption::ArraySimple,
            }),
            readonly: value
            .get(0)
            .and_then(|v| v.get("readonly"))
            .and_then(serde_json::Value::as_str)
            .map_or_else(|| None, |s| match s {
                "array" => Some(ArrayOption::Array),
                "generic" => Some(ArrayOption::Generic),
                _ => Some(ArrayOption::ArraySimple),
            }),
        }))
    }
    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {
        let default_config = &self.default;
        let readonly_config = &self.readonly.clone().unwrap_or(default_config.clone());

        println!("{:?} {:?}", default_config, readonly_config);
        // if let ArrayOption::Array = default_config {
        //     println!("111");
        // } else {
        //     println!("222");
        // }
    }
}

#[test]
fn test() {
  use crate::tester::Tester;

  let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
    ("let a: number[] = [];", None),
    // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array" }])))
  ];
  let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
    // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "generic" }]))),
  ];

  Tester::new(ArrayType::NAME, pass, fail).test();
}
