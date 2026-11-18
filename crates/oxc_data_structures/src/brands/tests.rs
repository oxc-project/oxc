use std::num::NonZero;

use crate::types::implements;

use super::*;

#[test]
fn brand_origin_cannot_be_copied() {
    assert!(implements!(BrandOrigin: !Clone));
    assert!(implements!(BrandOrigin: !Copy));
}

#[test]
fn brand_can_be_copied() {
    assert!(implements!(Brand: Clone));
    assert!(implements!(Brand: Copy));
}

/// `Default` would allow creating a `BrandOrigin` or `Brand` with any `'id`,
/// outside of the closure passed to [`BrandOrigin::scope`].
///
/// `Default` for `BrandScope` would allow [`BrandScope::extend_lifetime`]
/// to change any lifetime to any other.
#[test]
fn cannot_be_created_with_default() {
    assert!(implements!(BrandOrigin: !Default));
    assert!(implements!(Brand: !Default));
    assert!(implements!(BrandScope: !Default));
}

#[test]
fn send_and_sync() {
    assert!(implements!(BrandOrigin: Send));
    assert!(implements!(BrandOrigin: Sync));
    assert!(implements!(Brand: Send));
    assert!(implements!(Brand: Sync));
}

#[test]
fn unbranded() {
    assert!(implements!(bool: Unbranded));
    assert!(implements!(char: Unbranded));
    assert!(implements!(u8: Unbranded));
    assert!(implements!(usize: Unbranded));
    assert!(implements!(i128: Unbranded));
    assert!(implements!(f64: Unbranded));
    assert!(implements!(NonZero<u32>: Unbranded));
    assert!(implements!(NonZero<isize>: Unbranded));
    assert!(implements!((): Unbranded));
    assert!(implements!(String: Unbranded));

    assert!(implements!(&str: Unbranded));
    assert!(implements!(&mut str: Unbranded));
    assert!(implements!(&u32: Unbranded));
    assert!(implements!(&mut u32: Unbranded));
    assert!(implements!(&[u8]: Unbranded));
    assert!(implements!(&mut [u8]: Unbranded));
    assert!(implements!(&&str: Unbranded));
    assert!(implements!(&[&str]: Unbranded));
    assert!(implements!(&Option<u8>: Unbranded));

    assert!(implements!(Option<&str>: Unbranded));
    assert!(implements!(Option<Option<u8>>: Unbranded));
    assert!(implements!(Result<&str, ()>: Unbranded));
    assert!(implements!(Result<Option<&str>, NonZero<u32>>: Unbranded));
    assert!(implements!(Box<&str>: Unbranded));
    assert!(implements!(Vec<Option<&str>>: Unbranded));
    assert!(implements!([&str; 3]: Unbranded));
    assert!(implements!([[u8; 2]; 0]: Unbranded));
    assert!(implements!((&str,): Unbranded));
    assert!(implements!((&str, u32, Option<bool>): Unbranded));
    assert!(implements!((u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8): Unbranded));

    assert!(implements!(BrandOrigin: !Unbranded));
    assert!(implements!(Brand: !Unbranded));
    assert!(implements!(BrandScope: !Unbranded));
    assert!(implements!(Option<Brand>: !Unbranded));
    assert!(implements!(Result<Brand, ()>: !Unbranded));
    assert!(implements!(Result<(), BrandOrigin>: !Unbranded));
    assert!(implements!(Box<Brand>: !Unbranded));
    assert!(implements!(Vec<Brand>: !Unbranded));
    assert!(implements!([Brand; 2]: !Unbranded));
    assert!(implements!((u32, Brand): !Unbranded));
    assert!(implements!(&Brand: !Unbranded));
    assert!(implements!(&mut BrandOrigin: !Unbranded));
    assert!(implements!(&[Brand]: !Unbranded));
    assert!(implements!(&mut [Brand]: !Unbranded));
}

/// A reference to a type is `Unbranded` only if that type is
#[test]
fn unbranded_references() {
    /// Type with a lifetime, which is `Unbranded`
    #[expect(dead_code, reason = "only used as a type")]
    struct Word<'a>(&'a str);

    // SAFETY: `Word`'s only field is a `&'a str`.
    // Nothing relies on its lifetime to identify an owner.
    unsafe impl Unbranded for Word<'_> {
        type WithLifetime<'a> = Word<'a>;
    }

    /// Type with no lifetime, which isn't `Unbranded`
    struct NotUnbranded;

    assert!(implements!(&Word: Unbranded));
    assert!(implements!(&mut Word: Unbranded));
    assert!(implements!(&[Word]: Unbranded));
    assert!(implements!(&mut [Word]: Unbranded));
    assert!(implements!(Option<&[Word]>: Unbranded));

    assert!(implements!(&NotUnbranded: !Unbranded));
    assert!(implements!(&mut NotUnbranded: !Unbranded));
    assert!(implements!(&[NotUnbranded]: !Unbranded));
    assert!(implements!(&mut [NotUnbranded]: !Unbranded));
}

#[test]
fn debug() {
    /// Owner which derives `Debug`
    #[derive(Debug)]
    #[expect(dead_code, reason = "fields are only read by `Debug`")]
    struct Owner<'id> {
        num: u32,
        brand_origin: BrandOrigin<'id>,
    }

    /// Branded value which derives `Debug`
    #[derive(Debug)]
    #[expect(dead_code, reason = "fields are only read by `Debug`")]
    struct Value<'id> {
        num: u32,
        brand: Brand<'id>,
    }

    BrandOrigin::scope(|brand_origin| {
        let brand = brand_origin.brand();
        assert_eq!(format!("{brand_origin:?}"), "BrandOrigin");
        assert_eq!(format!("{brand:?}"), "Brand");
        assert_eq!(format!("{brand_origin:#?}"), "BrandOrigin");
        assert_eq!(format!("{brand:#?}"), "Brand");

        let value = Value { num: 2, brand };
        assert_eq!(format!("{value:?}"), "Value { num: 2, brand: Brand }");

        let owner = Owner { num: 1, brand_origin };
        assert_eq!(format!("{owner:?}"), "Owner { num: 1, brand_origin: BrandOrigin }");
    });
}

#[test]
fn zero_size() {
    assert_eq!(size_of::<BrandOrigin>(), 0);
    assert_eq!(align_of::<BrandOrigin>(), 1);
    assert_eq!(size_of::<Brand>(), 0);
    assert_eq!(align_of::<Brand>(), 1);
    assert_eq!(size_of::<BrandScope>(), 0);
    assert_eq!(align_of::<BrandScope>(), 1);
}
