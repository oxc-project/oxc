use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct Comment {
    #[napi(ts_type = "'Line' | 'Block'")]
    pub r#type: String,
    pub value: String,
    pub start: u32,
    pub end: u32,
}
