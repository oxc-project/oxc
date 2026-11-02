use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeEnumAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using values that do not share an enum type where an enum value is expected.
    /// This includes assignments, arguments, return values, type assertions, mutations,
    /// and computed keys in objects keyed by enum members.
    ///
    /// ### Why is this bad?
    ///
    /// Using the underlying number or string instead of an enum member bypasses the
    /// enum's type safety. Numeric mutations can also produce values outside the enum.
    /// Referencing enum members keeps values tied to the enum when its members change.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Fruit {
    ///   Apple,
    ///   Banana,
    /// }
    ///
    /// const fruit: Fruit = 0;
    /// function eat(fruit: Fruit) {}
    /// eat(1);
    ///
    /// function getFruit(): Fruit {
    ///   return 0;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// enum Fruit {
    ///   Apple,
    ///   Banana,
    /// }
    ///
    /// const fruit: Fruit = Fruit.Apple;
    /// function eat(fruit: Fruit) {}
    /// eat(Fruit.Banana);
    ///
    /// function getFruit(): Fruit {
    ///   return Fruit.Apple;
    /// }
    /// ```
    NoUnsafeEnumAssignment(tsgolint),
    typescript,
    suspicious,
    version = "next",
    short_description = "Disallow unsafe enum assignments.",
);

impl Rule for NoUnsafeEnumAssignment {}
