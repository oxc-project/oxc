use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct SourceLocation {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct Comment {
    #[napi(ts_type = "'Line' | 'Block'")]
    pub r#type: String,
    pub value: String,
    pub start: u32,
    pub end: u32,
    pub loc: Option<SourceLocation>,
}
