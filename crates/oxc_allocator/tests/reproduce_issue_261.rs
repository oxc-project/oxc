#![cfg(feature = "collections")]
use bumpalo::{collections::string::String, Bump};
use std::panic::AssertUnwindSafe;

#[test]
fn issue_261_reproduction() {
    let bump = Bump::new();
    let mut s = String::new_in(&bump);

    s.push_str("_få_b");
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        s.retain(|c| {
            if c == 'b' {
                panic!();
            }
            c != '_'
        });
    }));

    // The string should still be valid UTF-8
    let s_slice = s.as_bytes();
    if let Err(e) = core::str::from_utf8(s_slice) {
        panic!("Invalid UTF-8: {:?}", e);
    }
}
