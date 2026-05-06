use oxc_str::CompactStr;

/// A compile-time constant value from a TypeScript enum member.
/// Per the TypeScript spec, enum members can only evaluate to numbers or strings.
///
/// Note: Only `PartialEq` is derived — `Eq` is intentionally omitted because
/// `Number(NaN) != Number(NaN)` per IEEE 754. Do not use as a map key.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Number(f64),
    String(CompactStr),
}
