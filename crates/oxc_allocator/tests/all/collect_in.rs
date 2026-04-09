#![cfg(feature = "collections")]

use crate::quickcheck;
use bumpalo::collections::{CollectIn, String, Vec};
use bumpalo::Bump;
use std::string::String as StdString;
use std::vec::Vec as StdVec;

quickcheck! {
  fn test_string_collect(input: StdString) -> bool {
    let bump = Bump::new();
    let bump_str = input.chars().collect_in::<String>(&bump);

    bump_str == input
  }

  fn test_vec_collect(input: StdVec<i32>) -> bool {
    let bump = Bump::new();
    let bump_vec = input.clone().into_iter().collect_in::<Vec<_>>(&bump);

    bump_vec.as_slice() == input.as_slice()
  }
}
