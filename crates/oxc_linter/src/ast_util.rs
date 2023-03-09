use phf::{phf_set, Set};

pub const STRICT_MODE_NAMES: Set<&'static str> = phf_set! {
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
};
