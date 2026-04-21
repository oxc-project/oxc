// Test case for git conflict markers detection
// Based on https://github.com/rust-lang/rust/pull/106242
//
// NOTE: Parser stops at the first conflict marker encountered (fatal error),
// so only the first conflict in this file will be reported.
// Subsequent conflicts are included for completeness but won't be tested
// until the earlier conflicts are removed.

function test() {
<<<<<<< HEAD
    const x = 1;
=======
    const y = 2;
>>>>>>> branch
    return x;
}

// Test with diff3 format
function test2() {
<<<<<<< HEAD
    const a = 1;
||||||| parent
    const b = 2;
=======
    const c = 3;
>>>>>>> branch
    return a;
}

// Test in enum/object-like structure
const obj = {
<<<<<<< HEAD
    x: 1,
=======
    y: 2;
>>>>>>> branch
};

// Test incomplete conflict (only start marker)
function test3() {
<<<<<<< HEAD
    const incomplete = true;
    return incomplete;
}

// Test nested conflicts (only outermost conflict will be detected)
function nested() {
<<<<<<< OUTER
    const outer = 1;
<<<<<<< INNER
    const inner = 2;
=======
    const innerAlt = 3;
>>>>>>> INNER
=======
    const outerAlt = 4;
>>>>>>> OUTER
}

// Test different lexer contexts for >>>>>>>
// Context 1: After expression (may lex as ShiftRight3 + ShiftRight3 + RAngle)
const expr = a
>>>>>>> branch

// Context 2: At statement start (may lex as individual RAngle tokens)
>>>>>>> branch

// Context 3: After binary operator (may lex as ShiftRight + ShiftRight + ShiftRight + RAngle)
const x = 1 >>
>>>>>>> branch
