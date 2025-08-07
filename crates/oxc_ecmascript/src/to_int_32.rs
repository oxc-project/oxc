/// Converts a 64-bit floating point number to an `i32` according to the [`ToInt32`][ToInt32] algorithm.
///
/// [ToInt32]: https://tc39.es/ecma262/#sec-toint32
///
/// This is copied from [Boa](https://github.com/boa-dev/boa/blob/95c8d4820ad10ce32892dd75673b1d8b8854f974/core/engine/src/builtins/number/conversions.rs)
pub trait ToInt32 {
    fn to_int_32(&self) -> i32;
}

impl ToInt32 for f64 {
    fn to_int_32(&self) -> i32 {
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        {
            // macOS aarch64 always has jsconv feature
            // SAFETY: macOS aarch64 always supports jsconv
            unsafe { f64_to_int32_arm64(*self) }
        }
        #[cfg(all(target_arch = "aarch64", not(target_os = "macos")))]
        {
            if std::arch::is_aarch64_feature_detected!("jsconv") {
                // SAFETY: Feature detection confirmed jsconv is available
                unsafe { f64_to_int32_arm64(*self) }
            } else {
                f64_to_int32_generic(*self)
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            f64_to_int32_generic(*self)
        }
    }
}

/// Converts a 64-bit floating point number to an `i32` using [`FJCVTZS`][FJCVTZS] instruction on ARM64.
///
/// This requires ARM v8.3-A or later with the JavaScript conversion (jsconv) feature.
///
/// [FJCVTZS]: https://developer.arm.com/documentation/dui0801/h/A64-Floating-point-Instructions/FJCVTZS
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "jsconv")]
unsafe fn f64_to_int32_arm64(number: f64) -> i32 {
    if number.is_nan() {
        return 0;
    }
    let ret: i32;
    // SAFETY: Number is not nan so no floating-point exception should throw.
    unsafe {
        std::arch::asm!(
            "fjcvtzs {dst:w}, {src:d}",
            src = in(vreg) number,
            dst = out(reg) ret,
        );
    }
    ret
}

/// Generic implementation of ToInt32 for non-ARM64 architectures or ARM64 without JSCVT
#[expect(clippy::float_cmp, clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn f64_to_int32_generic(number: f64) -> i32 {
    const SIGN_MASK: u64 = 0x8000_0000_0000_0000;
    const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;
    const SIGNIFICAND_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
    const HIDDEN_BIT: u64 = 0x0010_0000_0000_0000;
    const PHYSICAL_SIGNIFICAND_SIZE: i32 = 52; // Excludes the hidden bit.
    const SIGNIFICAND_SIZE: i32 = 53;

    const EXPONENT_BIAS: i32 = 0x3FF + PHYSICAL_SIGNIFICAND_SIZE;
    const DENORMAL_EXPONENT: i32 = -EXPONENT_BIAS + 1;

    fn is_denormal(number: f64) -> bool {
        (number.to_bits() & EXPONENT_MASK) == 0
    }

    fn exponent(number: f64) -> i32 {
        if is_denormal(number) {
            return DENORMAL_EXPONENT;
        }

        let d64 = number.to_bits();
        let biased_e = ((d64 & EXPONENT_MASK) >> PHYSICAL_SIGNIFICAND_SIZE) as i32;

        biased_e - EXPONENT_BIAS
    }

    fn significand(number: f64) -> u64 {
        let d64 = number.to_bits();
        let significand = d64 & SIGNIFICAND_MASK;

        if is_denormal(number) { significand } else { significand + HIDDEN_BIT }
    }

    fn sign(number: f64) -> i64 {
        if (number.to_bits() & SIGN_MASK) == 0 { 1 } else { -1 }
    }

    // NOTE: this also matches with negative zero
    if !number.is_finite() || number == 0.0 {
        return 0;
    }

    if number.is_finite() && number <= f64::from(i32::MAX) && number >= f64::from(i32::MIN) {
        let i = number as i32;
        if f64::from(i) == number {
            return i;
        }
    }

    let exponent = exponent(number);
    let bits = if exponent < 0 {
        if exponent <= -SIGNIFICAND_SIZE {
            return 0;
        }

        significand(number) >> -exponent
    } else {
        if exponent > 31 {
            return 0;
        }

        (significand(number) << exponent) & 0xFFFF_FFFF
    };

    (sign(number) * (bits as i64)) as i32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn f64_to_int32_conversion() {
        assert_eq!(0.0_f64.to_int_32(), 0);
        assert_eq!((-0.0_f64).to_int_32(), 0);
        assert_eq!(f64::NAN.to_int_32(), 0);
        assert_eq!(f64::INFINITY.to_int_32(), 0);
        assert_eq!(f64::NEG_INFINITY.to_int_32(), 0);
        assert_eq!(((i64::from(i32::MAX) + 1) as f64).to_int_32(), i32::MIN);
        assert_eq!(((i64::from(i32::MIN) - 1) as f64).to_int_32(), i32::MAX);

        // Test edge cases with maximum safe integers
        assert_eq!((9_007_199_254_740_992.0_f64).to_int_32(), 0); // 2^53
        assert_eq!((-9_007_199_254_740_992.0_f64).to_int_32(), 0); // -2^53
    }

    #[test]
    fn test_generic_and_arm64_consistency() {
        // Test that generic implementation gives same results as ARM64 implementation
        // when both are available (ARM64 with JSCVT support)
        let test_values = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            42.7,
            -42.7,
            f64::from(i32::MAX),
            f64::from(i32::MIN),
            f64::from(i32::MAX) + 1.0,
            f64::from(i32::MIN) - 1.0,
            9_007_199_254_740_992.0,  // 2^53
            -9_007_199_254_740_992.0, // -2^53
        ];

        for &value in &test_values {
            let generic_result = f64_to_int32_generic(value);
            let trait_result = value.to_int_32();
            assert_eq!(
                generic_result, trait_result,
                "Mismatch for value {value}: generic={generic_result}, trait={trait_result}"
            );
        }
    }

    #[test]
    fn test_nan_handling() {
        // Both implementations should handle NaN the same way
        assert_eq!(f64::NAN.to_int_32(), 0);
        assert_eq!(f64_to_int32_generic(f64::NAN), 0);

        #[cfg(target_arch = "aarch64")]
        {
            // SAFETY: This is a test and we're only testing NaN handling
            unsafe {
                assert_eq!(f64_to_int32_arm64(f64::NAN), 0);
            }
        }
    }
}
