// Tests large numbers that may overflow integer parsing, but are all small
// enough to be valid f64s. Related to PR #4072 and issue #3347.

let HEX_INT_LARGER_THAN_MAX_SAFE_INTEGER: number = 0x10000000000000000
let OCT_INT_LARGER_THAN_MAX_SAFE_INTEGER: number = 0o40000000000000000
let BIN_INT_LARGER_THAN_MAX_SAFE_INTEGER: number = 0b00010000000000000000000000000000000000000000000000000000000000000000;

if (
    HEX_INT_LARGER_THAN_MAX_SAFE_INTEGER === 0 ||
    OCT_INT_LARGER_THAN_MAX_SAFE_INTEGER === 0 ||
    BIN_INT_LARGER_THAN_MAX_SAFE_INTEGER === 0
) {
    throw new Error('Large numeric literals are overflowing when parsed')
}
