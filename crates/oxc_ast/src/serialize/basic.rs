use oxc_ast_macros::ast_meta;
use oxc_estree::{ESTree, JsonSafeString, Serializer};

/// Serialized as `null`.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null", raw_deser_inline)]
pub struct Null<T>(pub T);

impl<T> ESTree for Null<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

/// Serialized as `null`. Field only present in TS-ESTree AST.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null", raw_deser_inline)]
#[ts]
pub struct TsNull<T>(pub T);

impl<T> ESTree for TsNull<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

/// Serialized as `true`.
#[ast_meta]
#[estree(ts_type = "true", raw_deser = "true", raw_deser_inline)]
pub struct True<T>(pub T);

impl<T> ESTree for True<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        true.serialize(serializer);
    }
}

/// Serialized as `false`.
#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false", raw_deser_inline)]
pub struct False<T>(pub T);

impl<T> ESTree for False<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `false`. Field only present in JS ESTree AST (not TS-ESTree).
#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false", raw_deser_inline)]
#[js_only]
pub struct JsFalse<T>(pub T);

impl<T> ESTree for JsFalse<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `false`. Field only present in TS-ESTree AST.
#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false", raw_deser_inline)]
#[ts]
pub struct TsFalse<T>(pub T);

impl<T> ESTree for TsFalse<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `"value"`.
#[ast_meta]
#[estree(ts_type = "'value'", raw_deser = "'value'", raw_deser_inline)]
#[ts]
pub struct TsValue<T>(pub T);

impl<T> ESTree for TsValue<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("value").serialize(serializer);
    }
}

/// Serialized as `"in"`.
#[ast_meta]
#[estree(ts_type = "'in'", raw_deser = "'in'", raw_deser_inline)]
pub struct In<T>(pub T);

impl<T> ESTree for In<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("in").serialize(serializer);
    }
}

/// Serialized as `"init"`.
#[ast_meta]
#[estree(ts_type = "'init'", raw_deser = "'init'", raw_deser_inline)]
pub struct Init<T>(pub T);

impl<T> ESTree for Init<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("init").serialize(serializer);
    }
}

/// Serialized as `"this"`.
#[ast_meta]
#[estree(ts_type = "'this'", raw_deser = "'this'", raw_deser_inline)]
pub struct This<T>(pub T);

impl<T> ESTree for This<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("this").serialize(serializer);
    }
}

/// Serialized as `"global"`.
#[ast_meta]
#[estree(ts_type = "'global'", raw_deser = "'global'", raw_deser_inline)]
pub struct Global<T>(pub T);

impl<T> ESTree for Global<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("global").serialize(serializer);
    }
}

/// Serialized as `[]`.
#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]", raw_deser_inline)]
pub struct EmptyArray<T>(pub T);

impl<T> ESTree for EmptyArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        [(); 0].serialize(serializer);
    }
}

/// Serialized as `[]`. Field only present in JS ESTree AST (not TS-ESTree).
#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]", raw_deser_inline)]
#[js_only]
pub struct JsEmptyArray<T>(pub T);

impl<T> ESTree for JsEmptyArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        EmptyArray(()).serialize(serializer);
    }
}

/// Serialized as `[]`. Field only present in TS-ESTree AST.
#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]", raw_deser_inline)]
#[ts]
pub struct TsEmptyArray<T>(pub T);

impl<T> ESTree for TsEmptyArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        EmptyArray(()).serialize(serializer);
    }
}
