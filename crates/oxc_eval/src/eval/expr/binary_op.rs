use crate::{Eval, EvalContext};
use oxc_ast::ast::{BinaryExpression, BinaryOperator, LogicalExpression, LogicalOperator};
use oxc_diagnostics::OxcDiagnostic;

use crate::{
    completion::TypeError,
    value::{Number, Numeric, Value},
    EvalResult,
};

macro_rules! ok {
    ($val:expr) => {
        Ok(From::from($val))
    };
    (num; $num:expr) => {
        Ok(Value::Number($num))
    };
}

macro_rules! err {
    ($msg:expr) => {
        Err(Some(OxcDiagnostic::error($msg)))
    };
}

// ### 13.15.4 EvaluateStringOrNumericBinaryExpression ( `leftOperand`, `opText`, `rightOperand` )
//
// > The abstract operation EvaluateStringOrNumericBinaryExpression takes
// > arguments `leftOperand` (a Parse Node), `opText` (a sequence of Unicode code
// > points), and `rightOperand` (a Parse Node) and returns either a normal
// > completion containing either a `String`, a `BigInt`, or a `Number`, or an abrupt
// > completion. It performs the following steps when called:
impl<'a> Eval<'a> for BinaryExpression<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        // 1. Let lref be ? Evaluation of leftOperand.
        // 2. Let lval be ? GetValue(lref).
        // 3. Let rref be ? Evaluation of rightOperand.
        // 4. Let rval be ? GetValue(rref).
        // 5. Return ? ApplyStringOrNumericBinaryOperator(lval, opText, rval).
        // NOTE: ReferenceRecord isn't implemented yet, so GetValue is a no-op.
        let lval = self.left.eval(ctx)?;
        let rval = self.right.eval(ctx)?;
        apply_string_or_numeric_binary_op(lval, self.operator, rval)
    }
}

/// ### 13.15.3 ApplyStringOrNumericBinaryOperator ( `lval`, `opText`, `rval` )
///
/// > The abstract operation ApplyStringOrNumericBinaryOperator takes arguments lval (an ECMAScript
/// > language value), opText (`**`, `*`, `/`, `%`, `+`, `-`, `<<`, `>>`, `>>>`, `&`, `^`, or `|`),
/// > and rval (an [ECMAScript language value](`Value`)) and returns either a normal completion
/// > containing either a `String`, a `BigInt`, or a `Number`, or a throw completion.
fn apply_string_or_numeric_binary_op<'a>(
    left: Value<'a>,
    op: BinaryOperator,
    right: Value<'a>,
) -> EvalResult<'a> {
    // 1. is opText is +, then
    if matches!(op, BinaryOperator::Addition) {
        // a. let lprim be ? ToPrimitive(lval).
        // b. let rprim be ? ToPrimitive(rval).
        let lprim = left.clone().to_primitive(None)?;
        let rprim = right.clone().to_primitive(None)?;
        // c. if lprim is a String or rprim is a String, then
        if lprim.is_string() || rprim.is_string() {
            // i. let lstr be ? ToString(lprim).
            // ii. let rstr be ? ToString(rprim).
            // iii. return the String that is the result of concatenating lstr and rstr.
            return ok!(format!("{}{}", lprim.to_string()?, rprim.to_string()?));
        }
    }
    // 2. NOTE: ath this point, it must be a numeric operation
    // 3. let lnum be ? ToNumeric(lval).
    // 4. let rnum be ? ToNumeric(rval).
    assert!(op.is_arithmetic() || op.is_bitwise());
    let lnum = left.to_numeric()?;
    let rnum = right.to_numeric()?;
    // 5. if Type(lnum) is not Type(rnum), throw a TypeError exception
    match (lnum, rnum) {
        // 6. if lnum is a BigInt, then
        (Numeric::BigInt(left), Numeric::BigInt(right)) => {
            todo!("BigInt::exponentiate et al")
        }
        (Numeric::Number(left), Numeric::Number(right)) => {
            match op {
                BinaryOperator::Exponential => ok!(left.powf(*right)),
                BinaryOperator::Multiplication => ok!(num; left * right),
                BinaryOperator::Division => {
                    if right == Number::ZERO {
                        err!(format!("Cannot divide {left} by 0."))
                    } else {
                        ok!(left / right)
                    }
                }
                BinaryOperator::Remainder => ok!(num; left % right),
                BinaryOperator::Addition => ok!(num; left + right),
                BinaryOperator::Subtraction => ok!(num; left - right),
                // https://262.ecma-international.org/15.0/index.html#sec-numeric-types-number-leftShift
                BinaryOperator::ShiftLeft => ok!(num; left << right),
                BinaryOperator::ShiftRight => ok!(num; left >> right),
                BinaryOperator::BitwiseAnd => ok!(num; left & right),
                BinaryOperator::BitwiseXOR => ok!(num; left ^ right),
                BinaryOperator::BitwiseOR => ok!(num; left | right),
                _ => unreachable!("Invalid binary operator"),
            }
        }
        // TODO: make this message better
        _ => Err(Some(TypeError::error("Cannot perform binary operation on different types"))),
    }
}

// 13.13.1 Binary Logical Operators - Runtime Semantics: Evaluation
// https://262.ecma-international.org/15.0/index.html#sec-binary-logical-operators-runtime-semantics-evaluation
impl<'a> Eval<'a> for LogicalExpression<'a> {
    fn eval(&self, ctx: &mut crate::EvalContext<'a>) -> EvalResult<'a> {
        // All three parts of this piecewise function share steps 1 and 2.
        let left = self.left.eval(ctx)?;
        // 2. Let lval be ? GetValue(lref).
        // NOTE: ReferenceRecord ils not supported yet, so this will always be a "regular" value.
        let lval = left;

        match self.operator {
            // LogicalANDExpression : LogicalANDExpression && BitwiseORExpression
            LogicalOperator::And => {
                // 3. let lbool be ToBoolean(lval).
                let lbool = lval.to_boolean();
                // 4. if lbool is false, return lval.
                if !lbool {
                    return Ok(lval);
                }
                // 5. Let rref be ? Evaluation of BitwiseORExpression
                // 6. return ? GetValue(rref).
                self.right.eval(ctx)
            }

            // LogicalORExpression : LogicalORExpression || LogicalANDExpression
            LogicalOperator::Or => {
                // 3. let lbool be ToBoolean(lval).
                let lbool = lval.to_boolean();
                // 4. if lbool is true, return lval.
                if lbool {
                    return Ok(lval);
                }
                // 5. Let rref be ? Evaluation of LogicalANDExpression
                // 6. return ? GetValue(rref).
                self.right.eval(ctx)
            }

            // CoalesceExpression : CoalesceExpression ?? BitwiseORExpression
            LogicalOperator::Coalesce => {
                // 3. if lval is either undefined or null, then
                if lval.is_null_or_undefined() {
                    // a. let rref be ? Evaluation of BitwiseORExpression.
                    // b. return ? GetValue(rref).
                    return self.right.eval(ctx);
                }
                // 4. Else,
                //   a. return lval.
                Ok(lval)
            }
        }
    }
}
