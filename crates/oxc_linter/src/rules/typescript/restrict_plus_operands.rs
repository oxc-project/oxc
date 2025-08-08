use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct RestrictPlusOperands;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires both operands of addition to be the same type and be number, string, or any.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript's + operator can be used for both numeric addition and string concatenation. When the operands are of different types, JavaScript's type coercion rules can lead to unexpected results. This rule helps prevent these issues by requiring both operands to be of compatible types.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const num: number;
    /// declare const str: string;
    /// declare const bool: boolean;
    /// declare const obj: object;
    ///
    /// // Mixed types
    /// const result1 = num + str; // number + string
    /// const result2 = str + bool; // string + boolean
    /// const result3 = num + bool; // number + boolean
    /// const result4 = obj + str; // object + string
    ///
    /// // Literals with different types
    /// const result5 = 42 + 'hello'; // number literal + string literal
    /// const result6 = true + 5; // boolean literal + number literal
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const num1: number;
    /// declare const num2: number;
    /// declare const str1: string;
    /// declare const str2: string;
    ///
    /// // Same types
    /// const sum = num1 + num2; // number + number
    /// const concat = str1 + str2; // string + string
    ///
    /// // Explicit conversions
    /// const result1 = num1 + String(num2); // Convert to string first
    /// const result2 = String(num1) + str1; // Convert to string first
    /// const result3 = Number(str1) + num1; // Convert to number first
    ///
    /// // Template literals for string concatenation
    /// const result4 = `${num1}${str1}`; // Clear intent to concatenate
    ///
    /// // Literals of same type
    /// const numResult = 42 + 58; // number + number
    /// const strResult = 'hello' + 'world'; // string + string
    /// ```
    RestrictPlusOperands(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for RestrictPlusOperands {}
