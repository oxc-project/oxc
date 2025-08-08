use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct OnlyThrowError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows throwing non-Error values.
    ///
    /// ### Why is this bad?
    ///
    /// It's considered good practice to only throw Error objects (or subclasses of Error). This is because Error objects automatically capture a stack trace, which is useful for debugging. Additionally, some tools and environments expect thrown values to be Error objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// throw 'error'; // throwing string
    ///
    /// throw 42; // throwing number
    ///
    /// throw true; // throwing boolean
    ///
    /// throw { message: 'error' }; // throwing plain object
    ///
    /// throw null; // throwing null
    ///
    /// throw undefined; // throwing undefined
    ///
    /// const error = 'Something went wrong';
    /// throw error; // throwing non-Error variable
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// throw new Error('Something went wrong');
    ///
    /// throw new TypeError('Invalid type');
    ///
    /// throw new RangeError('Value out of range');
    ///
    /// // Custom Error subclasses
    /// class CustomError extends Error {
    ///   constructor(message: string) {
    ///     super(message);
    ///     this.name = 'CustomError';
    ///   }
    /// }
    /// throw new CustomError('Custom error occurred');
    ///
    /// // Variables that are Error objects
    /// const error = new Error('Error message');
    /// throw error;
    /// ```
    OnlyThrowError(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for OnlyThrowError {}
