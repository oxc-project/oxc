pub use jolt::with_number_literal;
#[cfg(test)]
mod tests {
    use super::with_number_literal;

    #[test]
    fn small_integer_literals() {
        for integer in 0_u16..1000 {
            let expected = integer.to_string();
            with_number_literal(f64::from(integer), |literal| assert_eq!(literal, expected));
        }
    }

    #[test]
    fn small_integer_literal_boundaries() {
        for (value, expected) in [
            (-0.0, "0"),
            (0.0_f64.next_up(), "5e-324"),
            (0.5, ".5"),
            (1.0_f64.next_down(), ".9999999999999999"),
            (1.0_f64.next_up(), "1.0000000000000002"),
            (10.0_f64.next_down(), "9.999999999999998"),
            (10.0_f64.next_up(), "10.000000000000002"),
            (100.0_f64.next_down(), "99.99999999999999"),
            (100.0_f64.next_up(), "100.00000000000001"),
            (999.0_f64.next_down(), "998.9999999999999"),
            (999.0_f64.next_up(), "999.0000000000001"),
            (999.5, "999.5"),
            (1000.0_f64.next_down(), "999.9999999999999"),
            (1000.0, "1e3"),
            (1000.0_f64.next_up(), "1000.0000000000001"),
            (1001.0, "1001"),
        ] {
            with_number_literal(value, |literal| assert_eq!(literal, expected, "value: {value:?}"));
        }
    }

    #[test]
    fn shortest_number_literal() {
        for (value, expected) in [
            (0.0, "0"),
            (0.05, ".05"),
            (0.000_001, "1e-6"),
            (1000.0, "1e3"),
            (281_474_976_710_655.0, "0xffffffffffff"),
            (1.2e101, "12e100"),
            (f64::MAX, "17976931348623157e292"),
        ] {
            with_number_literal(value, |literal| assert_eq!(literal, expected));
        }
    }
}
