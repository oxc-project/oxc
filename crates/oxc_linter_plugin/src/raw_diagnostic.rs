use crate::errors::SpanStartOrEnd;

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
