use std::io::Write;

use cow_utils::CowUtils;
use serde::{
    ser::{SerializeMap, SerializeSeq, Serializer},
    Serialize,
};

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast_macros::ast_meta;

use crate::ast::*;

impl Program<'_> {
    /// Serialize AST to JSON.
    //
    // Should not panic if everything is working correctly.
    // Serializing into a `Vec<u8>` should be infallible.
    #[expect(clippy::missing_panics_doc)]
    pub fn to_json(&self) -> String {
        let buf = Vec::new();
        let ser = self.to_json_into_writer(buf).unwrap();
        let buf = ser.into_inner();
        // SAFETY: `serde_json` outputs valid UTF-8.
        // `serde_json::to_string` also uses `from_utf8_unchecked`.
        // https://github.com/serde-rs/json/blob/1174c5f57db44c26460951b525c6ede50984b655/src/ser.rs#L2209-L2219
        unsafe { String::from_utf8_unchecked(buf) }
    }

    /// Serialize AST into a "black hole" writer.
    ///
    /// Only useful for testing, to make sure serialization completes successfully.
    /// Should be faster than [`Program::to_json`], as does not actually produce any output.
    ///
    /// # Errors
    /// Returns `Err` if serialization fails.
    #[doc(hidden)]
    pub fn test_to_json(&self) -> Result<(), serde_json::Error> {
        struct BlackHole;

        #[expect(clippy::inline_always)]
        impl Write for BlackHole {
            #[inline(always)]
            fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
                Ok(buf.len())
            }

            #[inline(always)]
            fn flush(&mut self) -> Result<(), std::io::Error> {
                Ok(())
            }
        }

        self.to_json_into_writer(BlackHole).map(|_| ())
    }

    /// Serialize AST into the provided writer.
    fn to_json_into_writer<W: Write>(
        &self,
        writer: W,
    ) -> Result<serde_json::Serializer<W, EcmaFormatter>, serde_json::Error> {
        let mut ser = serde_json::Serializer::with_formatter(writer, EcmaFormatter);
        self.serialize(&mut ser)?;
        Ok(ser)
    }
}

/// `serde_json` formatter which uses `ryu_js` to serialize `f64`.
pub struct EcmaFormatter;

impl serde_json::ser::Formatter for EcmaFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        use oxc_syntax::number::ToJsString;
        writer.write_all(value.to_js_string().as_bytes())
    }
}

// --------------------
// Basic types
// --------------------

/// Serialized as `null`.
#[ast_meta]
#[estree(ts_type = "null")]
pub struct Null<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> Serialize for Null<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ().serialize(serializer)
    }
}

/// Serialized as `true`.
#[ast_meta]
#[estree(ts_type = "true")]
pub struct True<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> Serialize for True<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        true.serialize(serializer)
    }
}

/// Serialized as `false`.
#[ast_meta]
#[estree(ts_type = "false")]
pub struct False<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> Serialize for False<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        false.serialize(serializer)
    }
}

/// Serialized as `"in"`.
#[ast_meta]
#[estree(ts_type = "'in'")]
pub struct In<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> Serialize for In<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        "in".serialize(serializer)
    }
}

/// Serialized as `"init"`.
#[ast_meta]
#[estree(ts_type = "'init'")]
pub struct Init<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> Serialize for Init<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        "init".serialize(serializer)
    }
}

// --------------------
// Literals
// --------------------

/// Serializer for `raw` field of `BooleanLiteral`.
#[ast_meta]
#[estree(ts_type = "string | null")]
pub struct BooleanLiteralRaw<'b>(pub &'b BooleanLiteral);

impl Serialize for BooleanLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = if self.0.span.is_unspanned() {
            None
        } else if self.0.value {
            Some("true")
        } else {
            Some("false")
        };
        raw.serialize(serializer)
    }
}

/// Serializer for `raw` field of `NullLiteral`.
#[ast_meta]
#[estree(ts_type = "'null' | null")]
pub struct NullLiteralRaw<'b>(pub &'b NullLiteral);

impl Serialize for NullLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = if self.0.span.is_unspanned() { None } else { Some("null") };
        raw.serialize(serializer)
    }
}

/// Serializer for `bigint` field of `BigIntLiteral`.
#[ast_meta]
#[estree(ts_type = "string")]
pub struct BigIntLiteralBigint<'a, 'b>(pub &'b BigIntLiteral<'a>);

impl Serialize for BigIntLiteralBigint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let bigint = self.0.raw.strip_suffix('n').unwrap().cow_replace('_', "");
        bigint.serialize(serializer)
    }
}

/// Serializer for `value` field of `BigIntLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `BigInt`.
#[ast_meta]
#[estree(ts_type = "BigInt")]
pub struct BigIntLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b BigIntLiteral<'a>);

impl Serialize for BigIntLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

/// Serializer for `regex` field of `RegExpLiteral`.
pub struct RegExpLiteralRegex<'a, 'b>(pub &'b RegExpLiteral<'a>);

impl Serialize for RegExpLiteralRegex<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("pattern", &self.0.regex.pattern)?;

        // If `raw` field is present, flags must be in same order as in source to match Acorn.
        // Count number of set bits in `flags` to get number of flags
        // (cheaper than searching through `raw` for last `/`).
        let flags = self.0.regex.flags;
        if let Some(raw) = &self.0.raw {
            let flags_count = flags.bits().count_ones() as usize;
            let flags_index = raw.len() - flags_count;
            map.serialize_entry("flags", &raw[flags_index..])?;
        } else {
            map.serialize_entry("flags", &flags)?;
        }
        map.end()
    }
}

/// Serializer for `value` field of `RegExpLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `RegExp` if the regexp is valid.
#[ast_meta]
#[estree(ts_type = "RegExp | null")]
pub struct RegExpLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b RegExpLiteral<'a>);

impl Serialize for RegExpLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl Serialize for RegExpFlags {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RegExpPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

// --------------------
// Various
// --------------------

/// Serialize `ArrayExpressionElement::Elision` variant as `null`.
impl Serialize for Elision {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

/// Serialize `FormalParameters`, to be estree compatible, with `items` and `rest` fields combined
/// and `argument` field flattened.
impl Serialize for FormalParameters<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        for item in &self.items {
            seq.serialize_element(item)?;
        }

        if let Some(rest) = &self.rest {
            seq.serialize_element(&FormalParametersRest(rest))?;
        }

        seq.end()
    }
}

struct FormalParametersRest<'a, 'b>(&'b BindingRestElement<'a>);

impl Serialize for FormalParametersRest<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let rest = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        map.serialize_entry("start", &rest.span.start)?;
        map.serialize_entry("end", &rest.span.end)?;
        map.serialize_entry("argument", &rest.argument.kind)?;
        map.serialize_entry("type_annotation", &rest.argument.type_annotation)?;
        map.serialize_entry("optional", &rest.argument.optional)?;
        map.end()
    }
}

/// Wrap an `Option<Vec<T>>` so that it's serialized as an empty array (`[]`) if the `Option` is `None`.
pub struct OptionVecDefault<'a, 'b, T: Serialize>(pub &'b Option<ArenaVec<'a, T>>);

impl<T: Serialize> Serialize for OptionVecDefault<'_, '_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(vec) = &self.0 {
            vec.serialize(serializer)
        } else {
            [false; 0].serialize(serializer)
        }
    }
}

/// Serialize `ObjectProperty` with fields in same order as Acorn.
impl Serialize for ObjectProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Property")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("method", &self.method)?;
        map.serialize_entry("shorthand", &self.shorthand)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("key", &self.key)?;
        // Acorn has `kind` field before `value` for methods and shorthand properties
        if self.method || self.kind != PropertyKind::Init || self.shorthand {
            map.serialize_entry("kind", &self.kind)?;
            map.serialize_entry("value", &self.value)?;
        } else {
            map.serialize_entry("value", &self.value)?;
            map.serialize_entry("kind", &self.kind)?;
        }
        map.end()
    }
}

/// Serialize `BindingProperty` with fields in same order as Acorn.
impl Serialize for BindingProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Property")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("method", &crate::serialize::False(self))?;
        map.serialize_entry("shorthand", &self.shorthand)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("key", &self.key)?;
        // Acorn has `kind` field before `value` for shorthand properties
        if self.shorthand {
            map.serialize_entry("kind", &crate::serialize::Init(self))?;
            map.serialize_entry("value", &self.value)?;
        } else {
            map.serialize_entry("value", &self.value)?;
            map.serialize_entry("kind", &crate::serialize::Init(self))?;
        }
        map.end()
    }
}

/// Serializer for `ArrowFunctionExpression`'s `body` field.
///
/// Serializes as either an expression (if `expression` property is set),
/// or a `BlockStatement` (if it's not).
pub struct ArrowFunctionExpressionBody<'a>(pub &'a ArrowFunctionExpression<'a>);

impl Serialize for ArrowFunctionExpressionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(expression) = self.0.get_expression() {
            expression.serialize(serializer)
        } else {
            self.0.body.serialize(serializer)
        }
    }
}

/// Serializer for `AssignmentTargetPropertyIdentifier`'s `init` field
/// (which is renamed to `value` in ESTree AST).
pub struct AssignmentTargetPropertyIdentifierValue<'a>(
    pub &'a AssignmentTargetPropertyIdentifier<'a>,
);

impl Serialize for AssignmentTargetPropertyIdentifierValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(init) = &self.0.init {
            let mut map = serializer.serialize_map(None)?;
            map.serialize_entry("type", "AssignmentPattern")?;
            map.serialize_entry("start", &self.0.span.start)?;
            map.serialize_entry("end", &self.0.span.end)?;
            map.serialize_entry("left", &self.0.binding)?;
            map.serialize_entry("right", init)?;
            map.end()
        } else {
            self.0.binding.serialize(serializer)
        }
    }
}

/// Get `options` field of `ImportExpression` (from original `arguments` field).
pub fn import_expression_options<'a>(
    arguments: &'a [Expression<'a>],
) -> Option<&'a Expression<'a>> {
    arguments.first()
}

/// Serializer for `ImportDeclaration` and `ExportNamedDeclaration`'s `with_clause` field
/// (which is renamed to `attributes` in ESTree AST).
// https://github.com/estree/estree/blob/master/es2025.md#importdeclaration
// https://github.com/estree/estree/blob/master/es2025.md#exportnameddeclaration
pub struct ImportExportWithClause<'a>(pub &'a Option<ArenaBox<'a, WithClause<'a>>>);

impl Serialize for ImportExportWithClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(with_clause) = &self.0 {
            with_clause.with_entries.serialize(serializer)
        } else {
            [(); 0].serialize(serializer)
        }
    }
}

// --------------------
// JSX
// --------------------

impl Serialize for JSXElementName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Identifier(ident) => ident.serialize(serializer),
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer)
            }
            Self::NamespacedName(name) => name.serialize(serializer),
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}

impl Serialize for JSXMemberExpressionObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer)
            }
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}
