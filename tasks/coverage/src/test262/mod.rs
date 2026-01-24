mod meta;

pub use self::meta::{MetaData, Phase, TestFlag};

/// Parse Test262 metadata from code comments
pub fn parse_meta(code: &str) -> MetaData {
    let Some(start) = code.find("/*---") else { return MetaData::default() };
    let Some(end) = code.find("---*/") else { return MetaData::default() };
    let s = &code[start + 5..end].replace('\r', "\n");
    MetaData::from_str(s)
}
