#![cfg(all(feature = "boxed", feature = "serde"))]

use super::{assert_eq_json, Mixed, Test};

use bumpalo::{boxed::Box, Bump};

macro_rules! compare_std_box {
    (in $bump:ident; $x:expr) => {
        (Box::new_in($x, &$bump), std::boxed::Box::new($x))
    };
}

#[test]
fn test_box_serializes() {
    let bump = Bump::new();
    let (box_int, std_box_int) = compare_std_box!(in bump; 1);
    assert_eq_json!(box_int, std_box_int);
    let (box_str, std_box_str) = compare_std_box!(in bump; 1);
    assert_eq_json!(box_str, std_box_str);
    let (box_vec, std_box_vec) = compare_std_box!(in bump; std::vec!["hello", "world"]);
    assert_eq_json!(box_vec, std_box_vec);
}

#[test]
fn test_box_serializes_complex() {
    let bump = Bump::new();
    let (vec, std_vec) = compare_std_box![
        in bump;
        Mixed {
            i: 8,
            s: "a".into(),
            o: None,
            e: Test::Second,
        }
    ];
    assert_eq_json!(vec, std_vec);
    let de: std::boxed::Box<Mixed> =
        serde_json::from_str(&serde_json::to_string(&vec).unwrap()).unwrap();
    assert_eq!(de, std_vec);
}
