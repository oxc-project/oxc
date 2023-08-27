use oxc_span::Span;

use crate::errors::SpanStartOrEnd;

/// Represents an oxc span with more TryFrom implementations catered
/// towards span_start and span_end.
pub struct RawPluginDiagnostic {
    pub start: u32,
    pub end: u32,
}

impl TryFrom<(u64, u64)> for RawPluginDiagnostic {
    type Error = SpanStartOrEnd;

    fn try_from(value: (u64, u64)) -> Result<Self, Self::Error> {
        Ok(Self {
            start: u32::try_from(value.0).map_err(|_| SpanStartOrEnd::Start)?,
            end: u32::try_from(value.1).map_err(|_| SpanStartOrEnd::End)?,
        })
    }
}

impl TryFrom<(i64, i64)> for RawPluginDiagnostic {
    type Error = SpanStartOrEnd;

    fn try_from(value: (i64, i64)) -> Result<Self, Self::Error> {
        Ok(Self {
            start: u32::try_from(value.0).map_err(|_| SpanStartOrEnd::Start)?,
            end: u32::try_from(value.1).map_err(|_| SpanStartOrEnd::End)?,
        })
    }
}

impl From<RawPluginDiagnostic> for Span {
    fn from(val: RawPluginDiagnostic) -> Self {
        Self::new(val.start, val.end)
    }
}
