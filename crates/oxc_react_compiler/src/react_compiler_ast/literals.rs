use crate::react_compiler_diagnostics::JsString;

use crate::react_compiler_ast::common::BaseNode;

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub base: BaseNode,
    /// JS string values may contain unpaired surrogates; see [`JsString`].
    pub value: JsString,
}

#[derive(Debug, Clone)]
pub struct NumericLiteral {
    pub base: BaseNode,
    pub value: f64,
    /// Babel's extra field containing the raw source text.
    /// Used to recover exact f64 values that serde_json may parse imprecisely.
    pub extra: Option<NumericLiteralExtra>,
}

impl NumericLiteral {
    /// Get the f64 value, preferring re-parsing from `extra.raw` when available
    /// to avoid serde_json float parsing precision issues.
    pub fn precise_value(&self) -> f64 {
        if let Some(extra) = &self.extra {
            if let Ok(v) = extra.raw.parse::<f64>() {
                return v;
            }
        }
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct NumericLiteralExtra {
    pub raw: String,
    pub raw_value: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    pub base: BaseNode,
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct NullLiteral {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct BigIntLiteral {
    pub base: BaseNode,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct RegExpLiteral {
    pub base: BaseNode,
    pub pattern: String,
    pub flags: String,
}

#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub base: BaseNode,
    pub value: TemplateElementValue,
    pub tail: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateElementValue {
    pub raw: String,
    pub cooked: Option<String>,
}
