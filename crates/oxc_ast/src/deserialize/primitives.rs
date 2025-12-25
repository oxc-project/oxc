//! FromESTree implementations for primitive types.

use oxc_allocator::{Allocator, Box as ABox, Vec as AVec};
use oxc_span::{Atom, CompactStr, Span};

use super::{DeserError, DeserResult, FromESTree};

// Primitive types

impl<'a> FromESTree<'a> for bool {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_bool().ok_or(DeserError::ExpectedBool)
    }
}

impl<'a> FromESTree<'a> for u8 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_u64().and_then(|n| u8::try_from(n).ok()).ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for u32 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for u64 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_u64().ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for i32 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_i64().and_then(|n| i32::try_from(n).ok()).ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for i64 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_i64().ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for f64 {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_f64().ok_or(DeserError::ExpectedNumber)
    }
}

impl<'a> FromESTree<'a> for String {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        value.as_str().map(String::from).ok_or(DeserError::ExpectedString)
    }
}

impl<'a> FromESTree<'a> for &'a str {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let s = value.as_str().ok_or(DeserError::ExpectedString)?;
        Ok(allocator.alloc_str(s))
    }
}

// Container types

impl<'a, T: FromESTree<'a>> FromESTree<'a> for Option<T> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        if value.is_null() { Ok(None) } else { Ok(Some(T::from_estree(value, allocator)?)) }
    }
}

impl<'a, T: FromESTree<'a>> FromESTree<'a> for ABox<'a, T> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let inner = T::from_estree(value, allocator)?;
        Ok(ABox::new_in(inner, allocator))
    }
}

impl<'a, T: FromESTree<'a>> FromESTree<'a> for AVec<'a, T> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let arr = value.as_array().ok_or(DeserError::ExpectedArray)?;
        let mut vec = AVec::with_capacity_in(arr.len(), allocator);
        for item in arr {
            vec.push(T::from_estree(item, allocator)?);
        }
        Ok(vec)
    }
}

// Span type
impl<'a> FromESTree<'a> for Span {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        super::parse_span(value)
    }
}

// Atom type
impl<'a> FromESTree<'a> for Atom<'a> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let s = value.as_str().ok_or(DeserError::ExpectedString)?;
        Ok(Atom::from(allocator.alloc_str(s)))
    }
}

// CompactStr type
impl<'a> FromESTree<'a> for CompactStr {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = value.as_str().ok_or(DeserError::ExpectedString)?;
        Ok(CompactStr::new(s))
    }
}

// Unit type - used for empty fields
impl<'a> FromESTree<'a> for () {
    fn from_estree(_value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        Ok(())
    }
}
