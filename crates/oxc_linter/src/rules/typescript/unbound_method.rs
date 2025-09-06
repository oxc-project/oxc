use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct UnboundMethod;

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
);

impl Rule for UnboundMethod {}
