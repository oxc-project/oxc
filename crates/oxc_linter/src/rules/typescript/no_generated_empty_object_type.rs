use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoGeneratedEmptyObjectType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows type operations that resolve to the empty object type `{}`.
    /// This includes generic type references and intersections, but excludes
    /// mapped types whose keys have not yet been resolved.
    ///
    /// ### Why is this bad?
    ///
    /// The empty object type `{}` accepts any non-nullish value, including
    /// primitives such as strings and numbers. Accidentally producing this type
    /// with a utility type can therefore allow values that were not intended.
    ///
    /// This rule checks generated types. Use `typescript/no-empty-object-type`
    /// to disallow explicitly written empty object types.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Data = { name: string; value: number };
    /// type Empty = Omit<Data, 'name' | 'value'>;
    /// type NoProperties = Pick<Data, never>;
    /// type NonNullish = NonNullable<unknown>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Data = { name: string; value: number };
    /// type WithValue = Omit<Data, 'name'>;
    /// type WithOther = Omit<Data, 'name' | 'value'> & { other: string };
    ///
    /// type Keys<T> = T extends infer U ? keyof U : never;
    /// type Mapped<T extends object> = { [Key in Keys<T>]: Key };
    /// type Referenced<T extends object> = Mapped<T>;
    /// ```
    NoGeneratedEmptyObjectType(tsgolint),
    typescript,
    suspicious,
    version = "next",
    short_description = "Disallow type operations that resolve to the empty object type.",
);

impl Rule for NoGeneratedEmptyObjectType {}
