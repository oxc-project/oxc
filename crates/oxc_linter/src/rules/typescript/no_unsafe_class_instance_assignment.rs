use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeClassInstanceAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using structurally compatible values where an instance of a class
    /// or one of its subclasses is expected. It also checks values nested in types
    /// such as arrays and promises.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript's structural typing can accept an object with the same public
    /// members as a class instance. The object can bypass initialization in the
    /// class constructor and fail an `instanceof` check.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Session {
    ///   constructor(readonly id: string) {}
    /// }
    ///
    /// const session: Session = { id: "guest" };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Session {
    ///   constructor(readonly id: string) {}
    /// }
    ///
    /// const session: Session = new Session("guest");
    /// ```
    NoUnsafeClassInstanceAssignment(tsgolint),
    typescript,
    suspicious,
    version = "1.95.0",
    short_description = "Disallow using non-instance values where a class instance is expected.",
);

impl Rule for NoUnsafeClassInstanceAssignment {}
