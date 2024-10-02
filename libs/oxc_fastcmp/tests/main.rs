extern crate oxc_fastcmp;
use oxc_fastcmp::Compare;

extern crate rand;
use rand::random;

#[test]
fn memcmp_success() {
    let mut vec = vec![];
    for _ in 1..1001 {
        vec.push(random::<u8>());
        assert!(vec.feq(&vec));
    }
}

#[test]
fn memcmp_failure() {
    let mut vec_1 = vec![];
    let mut vec_2 = vec![];
    for _ in 1..1000 {
        vec_1.push(random::<u8>());
        vec_2.push(0);
        assert!(!vec_1.feq(&vec_2));
    }
}
