// Operator precedence levels, on the scale `oxc_syntax::precedence::Precedence` uses.
//
// A printer is told the precedence of the position it is printing into, and parenthesizes itself
// when its own precedence is lower. The numbering is Oxc's, so the two agree on what needs parens.

/** No precedence at all - nothing parenthesizes itself against this. */
export const PREC_LOWEST = 0;
/** The comma operator, as in a sequence expression. */
export const PREC_COMMA = 1;
/** `yield` and `yield*`. */
export const PREC_YIELD = 3;
/** Assignment, including the compound and logical forms, and arrow functions. */
export const PREC_ASSIGN = 4;
/** The `?:` conditional operator. */
export const PREC_CONDITIONAL = 5;
/** The equality operators, `==` through `!==`. */
export const PREC_EQUALS = 12;
/** The relational operators, including `in` and `instanceof`. */
export const PREC_COMPARE = 13;
/** `**`, the one right-associative binary operator. */
export const PREC_EXPONENTIATION = 17;
/** The prefix unary and update operators, `await`, and `!`. */
export const PREC_PREFIX = 18;
/** The postfix update operators, and the position a member expression's object prints at. */
export const PREC_POSTFIX = 19;
/** `new` without an argument list, which binds tighter than a call. */
export const PREC_NEW = 20;
/** Calls, member access and tagged templates - the tightest binding there is. */
export const PREC_CALL = 21;
