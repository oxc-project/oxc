use serde::Serialize;

/// https://github.com/microsoft/TypeScript/blob/61200368bb440ba8a40641be87c44d875ca31f69/src/server/protocol.ts#L1715
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRequest<'a> {
    pub file: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_content: Option<&'a str>,
}

/// https://github.com/microsoft/TypeScript/blob/61200368bb440ba8a40641be87c44d875ca31f69/src/server/protocol.ts#L292
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRequest<'a> {
    pub file: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub pos: u32,
    pub end: u32,
}

impl From<&oxc_span::Span> for Span {
    fn from(value: &oxc_span::Span) -> Self {
        Self { pos: value.start, end: value.end }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRequest<'a> {
    pub file: &'a str,
    pub span: Span,
}
