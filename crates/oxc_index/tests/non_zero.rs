// #![allow(
//     clippy::assertions_on_constants,
//     clippy::eq_op,
//     clippy::uninlined_format_args,
//     clippy::should_panic_without_expect,
//     clippy::cast_possible_truncation
// )]
//
// use oxc_index::NonZeroIndexVec;
//
// oxc_index::define_index_type! {
//     #[non_zero]
//     pub struct NonZeroIdx8 = u8;
// }
//
// oxc_index::define_index_type! {
//     #[non_zero]
//     pub struct NonZeroIdx16 = u16;
// }
//
// oxc_index::define_index_type! {
//     #[non_zero]
//     pub struct NonZeroIdx32 = u32;
// }
//
// oxc_index::define_index_type! {
//     #[non_zero]
//     pub struct NonZeroIdxSz = usize;
// }
//
// #[test]
// #[should_panic]
// fn test_nz_idx_new_0_panics() {
//     NonZeroIdx8::new(0);
//     NonZeroIdx16::new(0);
//     NonZeroIdx32::new(0);
//     NonZeroIdxSz::new(0);
// }
//
// #[test]
// fn test_nz_idx_default_max() {
//     assert_eq!(NonZeroIdx32::MAX_INDEX, u32::max_value() as usize);
//     assert_eq!(NonZeroIdxSz::MAX_INDEX, usize::max_value());
//     assert_eq!(NonZeroIdx16::MAX_INDEX, u16::max_value() as usize);
//     assert_eq!(NonZeroIdx8::MAX_INDEX, u8::max_value() as usize);
//
//     assert!(NonZeroIdx32::CHECKS_MAX_INDEX);
//     assert!(NonZeroIdxSz::CHECKS_MAX_INDEX);
//     assert!(NonZeroIdx16::CHECKS_MAX_INDEX);
//     assert!(NonZeroIdx8::CHECKS_MAX_INDEX);
// }
//
// #[test]
// fn test_nz_idx_arith() {
//     assert_eq!(NonZeroIdx32::new(1), 1usize);
//     assert_eq!(NonZeroIdx32::new(1) + 1, 2usize);
//     assert_eq!(1 + NonZeroIdx32::new(1), 2usize);
//
//     assert_eq!(NonZeroIdx32::new(2) - 1, 1usize);
//     assert_eq!(NonZeroIdx32::new(5) % 4, 1usize);
//
//     let mut m = NonZeroIdx32::new(5);
//     m += 1;
//     assert_eq!(m, 6);
//
//     assert!(NonZeroIdx32::new(5) < NonZeroIdx32::new(6));
//     assert!(NonZeroIdx32::new(5) < 6usize);
//
//     assert!(NonZeroIdx32::new(5) < NonZeroIdx32::new(6));
//     assert!(NonZeroIdx32::new(5) < 6usize);
//     assert!(5usize < NonZeroIdx32::new(6));
// }
//
// #[test]
// fn test_non_zero_index_vec() {
//     let mut vec: NonZeroIndexVec<NonZeroIdx8, u8> = NonZeroIndexVec::new();
//     let idx = vec.push(1);
// }
