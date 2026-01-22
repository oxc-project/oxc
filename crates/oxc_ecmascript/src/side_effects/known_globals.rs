//! Known global identifiers and side-effect-free member expressions.
//!
//! This module contains lists of known globals and their properties that can be accessed
//! without side effects. This is used by bundlers for tree-shaking.

/// Known global identifiers that can be accessed without side effects.
///
/// This includes:
/// - Built-in JavaScript objects (Array, Object, Math, etc.)
/// - Browser APIs (Document, Window, etc.)
/// - Node.js globals (when applicable)
static KNOWN_GLOBAL_IDENTS: phf::Set<&str> = phf::phf_set![
    // Core JavaScript globals
    "Infinity",
    "undefined",
    "NaN",
    "Array",
    "Boolean",
    "Function",
    "Math",
    "Number",
    "Object",
    "RegExp",
    "String",
    // Common globals in browser and node
    "AbortController",
    "AbortSignal",
    "AggregateError",
    "ArrayBuffer",
    "BigInt",
    "DataView",
    "Date",
    "Error",
    "EvalError",
    "Event",
    "EventTarget",
    "Float32Array",
    "Float64Array",
    "Int16Array",
    "Int32Array",
    "Int8Array",
    "Intl",
    "JSON",
    "Map",
    "MessageChannel",
    "MessageEvent",
    "MessagePort",
    "Promise",
    "Proxy",
    "RangeError",
    "ReferenceError",
    "Reflect",
    "Set",
    "Symbol",
    "SyntaxError",
    "TextDecoder",
    "TextEncoder",
    "TypeError",
    "URIError",
    "URL",
    "URLSearchParams",
    "Uint16Array",
    "Uint32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "WeakMap",
    "WeakSet",
    "WebAssembly",
    "clearInterval",
    "clearTimeout",
    "console",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "escape",
    "globalThis",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "queueMicrotask",
    "setInterval",
    "setTimeout",
    "unescape",
];

/// Math: Static properties and methods
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math>
static MATH_PROPERTIES: phf::Set<&str> = phf::phf_set![
    // Properties
    "E", "LN10", "LN2", "LOG10E", "LOG2E", "PI", "SQRT1_2", "SQRT2", // Methods
    "abs", "acos", "acosh", "asin", "asinh", "atan", "atan2", "atanh", "cbrt", "ceil", "clz32",
    "cos", "cosh", "exp", "expm1", "floor", "fround", "hypot", "imul", "log", "log10", "log1p",
    "log2", "max", "min", "pow", "random", "round", "sign", "sin", "sinh", "sqrt", "tan", "tanh",
    "trunc",
];

/// Console methods (assumed side-effect-free for property access, not calls)
/// <https://developer.mozilla.org/en-US/docs/Web/API/console>
static CONSOLE_PROPERTIES: phf::Set<&str> = phf::phf_set![
    "assert",
    "clear",
    "count",
    "countReset",
    "debug",
    "dir",
    "dirxml",
    "error",
    "group",
    "groupCollapsed",
    "groupEnd",
    "info",
    "log",
    "table",
    "time",
    "timeEnd",
    "timeLog",
    "trace",
    "warn",
];

/// Reflect: Static methods
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect>
static REFLECT_PROPERTIES: phf::Set<&str> = phf::phf_set![
    "apply",
    "construct",
    "defineProperty",
    "deleteProperty",
    "get",
    "getOwnPropertyDescriptor",
    "getPrototypeOf",
    "has",
    "isExtensible",
    "ownKeys",
    "preventExtensions",
    "set",
    "setPrototypeOf",
];

/// Object: Static methods
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object>
static OBJECT_PROPERTIES: phf::Set<&str> = phf::phf_set![
    "assign",
    "create",
    "defineProperties",
    "defineProperty",
    "entries",
    "freeze",
    "fromEntries",
    "getOwnPropertyDescriptor",
    "getOwnPropertyDescriptors",
    "getOwnPropertyNames",
    "getOwnPropertySymbols",
    "getPrototypeOf",
    "is",
    "isExtensible",
    "isFrozen",
    "isSealed",
    "keys",
    "preventExtensions",
    "seal",
    "setPrototypeOf",
    "values",
];

/// Symbol: Static properties
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol>
static SYMBOL_PROPERTIES: phf::Set<&str> = phf::phf_set![
    "asyncDispose",
    "asyncIterator",
    "dispose",
    "hasInstance",
    "isConcatSpreadable",
    "iterator",
    "match",
    "matchAll",
    "replace",
    "search",
    "species",
    "split",
    "toPrimitive",
    "toStringTag",
    "unscopables",
];

/// Object.prototype: Instance methods
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object>
static OBJECT_PROTOTYPE_PROPERTIES: phf::Set<&str> = phf::phf_set![
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "unwatch",
    "valueOf",
    "watch",
];

/// Check if an identifier is a known global that can be accessed without side effects.
#[inline]
pub fn is_known_global_ident(name: &str) -> bool {
    KNOWN_GLOBAL_IDENTS.contains(name)
}

/// Check if a two-part member expression (like `Math.abs`, `JSON.stringify`) can be
/// accessed without side effects.
///
/// # Arguments
/// * `object` - The object part (e.g., "Math", "JSON")
/// * `property` - The property part (e.g., "abs", "stringify")
#[inline]
pub fn is_side_effect_free_member_access(object: &str, property: &str) -> bool {
    match object {
        "Math" => MATH_PROPERTIES.contains(property),
        "console" => CONSOLE_PROPERTIES.contains(property),
        "Reflect" => REFLECT_PROPERTIES.contains(property),
        "Object" => OBJECT_PROPERTIES.contains(property),
        "Symbol" => SYMBOL_PROPERTIES.contains(property),
        "JSON" => property == "stringify" || property == "parse",
        _ => false,
    }
}

/// Check if a three-part member expression (like `Object.prototype.hasOwnProperty`) can be
/// accessed without side effects.
///
/// # Arguments
/// * `object` - The object part (e.g., "Object")
/// * `middle` - The middle part (e.g., "prototype")
/// * `property` - The property part (e.g., "hasOwnProperty")
#[inline]
pub fn is_side_effect_free_nested_member_access(
    object: &str,
    middle: &str,
    property: &str,
) -> bool {
    object == "Object" && middle == "prototype" && OBJECT_PROTOTYPE_PROPERTIES.contains(property)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_globals() {
        assert!(is_known_global_ident("Math"));
        assert!(is_known_global_ident("JSON"));
        assert!(is_known_global_ident("Object"));
        assert!(is_known_global_ident("console"));
        assert!(is_known_global_ident("undefined"));
        assert!(!is_known_global_ident("foo"));
        assert!(!is_known_global_ident("myVariable"));
    }

    #[test]
    fn test_member_access() {
        assert!(is_side_effect_free_member_access("Math", "abs"));
        assert!(is_side_effect_free_member_access("Math", "PI"));
        assert!(is_side_effect_free_member_access("JSON", "stringify"));
        assert!(is_side_effect_free_member_access("JSON", "parse"));
        assert!(is_side_effect_free_member_access("Object", "assign"));
        assert!(is_side_effect_free_member_access("console", "log"));
        assert!(is_side_effect_free_member_access("Reflect", "apply"));
        assert!(is_side_effect_free_member_access("Symbol", "iterator"));

        assert!(!is_side_effect_free_member_access("Math", "unknown"));
        assert!(!is_side_effect_free_member_access("foo", "bar"));
        assert!(!is_side_effect_free_member_access("Object", "test"));
    }

    #[test]
    fn test_nested_member_access() {
        assert!(is_side_effect_free_nested_member_access("Object", "prototype", "hasOwnProperty"));
        assert!(is_side_effect_free_nested_member_access("Object", "prototype", "toString"));
        assert!(!is_side_effect_free_nested_member_access("Object", "prototype", "unknown"));
        assert!(!is_side_effect_free_nested_member_access("Array", "prototype", "map"));
    }
}
