use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct UnboundMethod(Box<UnboundMethodConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UnboundMethodConfig {
    /// Whether to ignore unbound methods that are static.
    /// When true, static methods can be referenced without binding.
    pub ignore_static: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces unbound methods are called with their expected scope.
    ///
    /// ### Why is this bad?
    ///
    /// When you extract a method from an object and call it separately, the `this` context is lost. This can lead to runtime errors or unexpected behavior, especially with methods that rely on `this` to access instance properties or other methods.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class MyClass {
    ///   private value = 42;
    ///
    ///   getValue() {
    ///     return this.value;
    ///   }
    ///
    ///   processValue() {
    ///     return this.value * 2;
    ///   }
    /// }
    ///
    /// const instance = new MyClass();
    ///
    /// // Unbound method - loses 'this' context
    /// const getValue = instance.getValue;
    /// getValue(); // Runtime error: cannot read property 'value' of undefined
    ///
    /// // Passing unbound method as callback
    /// [1, 2, 3].map(instance.processValue); // 'this' will be undefined
    ///
    /// // Destructuring methods
    /// const { getValue: unboundGetValue } = instance;
    /// unboundGetValue(); // Runtime error
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class MyClass {
    ///   private value = 42;
    ///
    ///   getValue() {
    ///     return this.value;
    ///   }
    ///
    ///   processValue() {
    ///     return this.value * 2;
    ///   }
    /// }
    ///
    /// const instance = new MyClass();
    ///
    /// // Call method on instance
    /// const value = instance.getValue(); // Correct
    ///
    /// // Bind method to preserve context
    /// const boundGetValue = instance.getValue.bind(instance);
    /// boundGetValue(); // Correct
    ///
    /// // Use arrow function to preserve context
    /// [1, 2, 3].map(() => instance.processValue()); // Correct
    ///
    /// // Use arrow function in class for auto-binding
    /// class MyClassWithArrow {
    ///   private value = 42;
    ///
    ///   getValue = () => {
    ///     return this.value;
    ///   };
    /// }
    ///
    /// const instance2 = new MyClassWithArrow();
    /// const getValue = instance2.getValue; // Safe - arrow function preserves 'this'
    /// getValue(); // Correct
    /// ```
    UnboundMethod(tsgolint),
    typescript,
    correctness,
    pending,
    config = UnboundMethodConfig,
);

impl Rule for UnboundMethod {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<UnboundMethod>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
