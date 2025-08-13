use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct PreferReturnThisType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces using `this` types for return types when possible.
    ///
    /// ### Why is this bad?
    ///
    /// Classes that have methods which return the instance itself should use `this` as the return type instead of the class name. This provides better type safety for inheritance, as the return type will be the actual subclass type rather than the base class type.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Builder {
    ///   private value: string = '';
    ///
    ///   setValue(value: string): Builder { // Should return 'this'
    ///     this.value = value;
    ///     return this;
    ///   }
    ///
    ///   build(): string {
    ///     return this.value;
    ///   }
    /// }
    ///
    /// class FluentAPI {
    ///   method1(): FluentAPI { // Should return 'this'
    ///     return this;
    ///   }
    ///
    ///   method2(): FluentAPI { // Should return 'this'
    ///     return this;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Builder {
    ///   private value: string = '';
    ///
    ///   setValue(value: string): this {
    ///     this.value = value;
    ///     return this;
    ///   }
    ///
    ///   build(): string {
    ///     return this.value;
    ///   }
    /// }
    ///
    /// class FluentAPI {
    ///   method1(): this {
    ///     return this;
    ///   }
    ///
    ///   method2(): this {
    ///     return this;
    ///   }
    /// }
    ///
    /// // Now inheritance works correctly
    /// class ExtendedBuilder extends Builder {
    ///   setPrefix(prefix: string): this {
    ///     // The return type is 'this' (ExtendedBuilder), not Builder
    ///     return this.setValue(prefix + this.getValue());
    ///   }
    /// }
    /// ```
    PreferReturnThisType(tsgolint),
    typescript,
    style,
    pending,
);

impl Rule for PreferReturnThisType {}
