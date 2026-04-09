#![cfg(all(feature = "collections", feature = "serde"))]

use super::{assert_eq_json, Mixed, Test};

use bumpalo::{vec, Bump};

macro_rules! compare_std_vec {
    (in $bump:ident; $($x:expr),+) => {{
        let vec = vec![in &$bump; $($x),+];
        let std_vec = std::vec![$($x),+];
        (vec, std_vec)
    }}
}

#[test]
fn test_vec_serializes_str() {
    let bump = Bump::new();
    let (vec, std_vec) = compare_std_vec![in bump; "hello", "world"];
    assert_eq_json!(vec, std_vec);
    let de: std::vec::Vec<std::string::String> =
        serde_json::from_str(&serde_json::to_string(&vec).unwrap()).unwrap();
    assert_eq!(de, std_vec);
}

#[test]
fn test_vec_serializes_f32() {
    let bump = Bump::new();
    let (vec, std_vec) = compare_std_vec![in bump; 1.5707964, 3.1415927];
    assert_eq_json!(vec, std_vec);
    let de: std::vec::Vec<f32> =
        serde_json::from_str(&serde_json::to_string(&vec).unwrap()).unwrap();
    assert_eq!(de, std_vec);
}

#[test]
fn test_vec_serializes_complex() {
    let bump = Bump::new();
    let (vec, std_vec) = compare_std_vec![
        in bump;
        Mixed {
            i: 8,
            s: "a".into(),
            o: None,
            e: Test::Second,
        },
        Mixed {
            i: 8,
            s: "b".into(),
            o: Some("some".into()),
            e: Test::First,
        }
    ];
    assert_eq_json!(vec, std_vec);
    let de: std::vec::Vec<Mixed> =
        serde_json::from_str(&serde_json::to_string(&vec).unwrap()).unwrap();
    assert_eq!(de, std_vec);
}
