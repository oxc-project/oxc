use super::*;

#[test]
fn construct() {
    let zero = NonMaxU32::new(0).unwrap();
    assert_eq!(zero.get(), 0);

    let some = NonMaxU32::new(19).unwrap();
    assert_eq!(some.get(), 19);

    let max_minus_one = NonMaxU32::new(u32::MAX - 1).unwrap();
    assert_eq!(max_minus_one.get(), u32::MAX - 1);

    let max = NonMaxU32::new(u32::MAX);
    assert_eq!(max, None);
}

#[test]
fn construct_unchecked() {
    // SAFETY: 0 is a valid value
    let zero = unsafe { NonMaxU32::new_unchecked(0) };
    assert_eq!(zero.get(), 0);

    // SAFETY: 19 is a valid value
    let some = unsafe { NonMaxU32::new_unchecked(19) };
    assert_eq!(some.get(), 19);

    // SAFETY: `u32::MAX - 1` is a valid value
    let max_minus_one = unsafe { NonMaxU32::new_unchecked(u32::MAX - 1) };
    assert_eq!(max_minus_one.get(), u32::MAX - 1);
}

#[test]
fn convert() {
    let zero = NonMaxU32::try_from(0u32).unwrap();
    let zero = u32::from(zero);
    assert_eq!(zero, 0);

    NonMaxU32::try_from(u32::MAX).unwrap_err();
}

#[test]
fn eq() {
    let zero = NonMaxU32::new(0).unwrap();
    let one = NonMaxU32::new(1).unwrap();
    let two = NonMaxU32::new(2).unwrap();
    assert_eq!(zero, zero);
    assert_eq!(one, one);
    assert_eq!(two, two);

    assert_ne!(zero, one);
    assert_ne!(zero, two);
    assert_ne!(one, two);
    assert_ne!(one, zero);
    assert_ne!(two, zero);
    assert_ne!(two, one);
}

#[test]
fn cmp() {
    let zero = NonMaxU32::new(0).unwrap();
    let one = NonMaxU32::new(1).unwrap();
    let two = NonMaxU32::new(2).unwrap();
    assert!(zero < one);
    assert!(one < two);
    assert!(two > one);
    assert!(one > zero);
}

#[test]
fn constants() {
    let zero = NonMaxU32::ZERO;
    let max = NonMaxU32::MAX;
    assert_eq!(zero.get(), 0);
    assert_eq!(max.get(), u32::MAX - 1);
}

#[test]
fn fmt() {
    let zero = NonMaxU32::new(0).unwrap();
    let some = NonMaxU32::new(19).unwrap();
    let max_minus_one = NonMaxU32::new(u32::MAX - 1).unwrap();
    for value in [zero, some, max_minus_one].iter().copied() {
        assert_eq!(format!("{}", value.get()), format!("{}", value)); // Display
        assert_eq!(format!("{:?}", value.get()), format!("{:?}", value)); // Debug
        assert_eq!(format!("{:b}", value.get()), format!("{:b}", value)); // Binary
        assert_eq!(format!("{:o}", value.get()), format!("{:o}", value)); // Octal
        assert_eq!(format!("{:x}", value.get()), format!("{:x}", value)); // LowerHex
        assert_eq!(format!("{:X}", value.get()), format!("{:X}", value)); // UpperHex
    }
}
