use napi::{
    Env, Error, Status, Unknown,
    bindgen_prelude::{
        FnArgs, FromNapiValue, Function, JsObjectValue, JsValue, Object, ToNapiValue, TypeName,
        ValidateNapiValue,
    },
    sys,
};

/// Owned data copied from a JavaScript `RegExp`.
#[derive(Debug, Default, Clone)]
pub struct JsRegExp {
    /// Pattern source without delimiters.
    pub source: String,
    /// JavaScript regular expression flags.
    pub flags: String,
    /// Current match position used by global and sticky regular expressions.
    pub last_index: u32,
}

impl JsRegExp {
    /// Creates a regular expression with `lastIndex` set to zero.
    pub fn new(source: String, flags: String) -> Self {
        Self { source, flags, last_index: 0 }
    }
}

impl TypeName for JsRegExp {
    fn type_name() -> &'static str {
        "RegExp"
    }

    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}

impl ValidateNapiValue for JsRegExp {}

impl FromNapiValue for JsRegExp {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        let object = Object::from_raw(env, napi_val);
        let env = Env::from(env);
        let global = env.get_global()?;
        let constructor = global.get_named_property::<Function<Unknown, ()>>("RegExp")?;

        if !object.instanceof(constructor)? {
            return Err(Error::new(Status::ObjectExpected, "Expected a RegExp object"));
        }

        Ok(Self {
            source: object.get_named_property::<String>("source")?,
            flags: object.get_named_property::<String>("flags")?,
            last_index: object.get_named_property::<u32>("lastIndex").unwrap_or_default(),
        })
    }
}

impl ToNapiValue for JsRegExp {
    unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> napi::Result<sys::napi_value> {
        let global = Env::from(env).get_global()?;
        let constructor =
            global.get_named_property::<Function<FnArgs<(String, String)>, Unknown>>("RegExp")?;
        let object = constructor.new_instance(FnArgs::from((value.source, value.flags)))?;
        let mut object = Object::from_raw(env, object.raw());
        if value.last_index != 0 {
            object.set_named_property("lastIndex", value.last_index)?;
        }
        Ok(object.raw())
    }
}
