use napi_derive::napi;

#[napi]
pub struct MagicString; // (string_wizard::MagicString<'static>);

impl MagicString {
    pub fn new(_s: String) -> Self {
        Self //(string_wizard::MagicString::new(s))
    }
}

#[napi(object)]
pub struct OverwriteOptions {
    pub content_only: bool,
}

#[napi]
impl MagicString {
    #[napi]
    pub fn length(&self) -> u32 {
        // self.0.len() as u32
        unimplemented!()
    }

    #[napi]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        // self.0.to_string()
        unimplemented!()
    }

    #[napi]
    pub fn append(&mut self, _input: String) -> &Self {
        // self.0.append(input);
        // self
        unimplemented!()
    }

    #[napi]
    pub fn prepend(&mut self, _input: String) -> &Self {
        // self.0.prepend(input);
        // self
        unimplemented!()
    }

    #[napi]
    pub fn append_left(&mut self, _index: u32, _input: String) -> &Self {
        // self.0.append_left(index as usize, input);
        // self
        unimplemented!()
    }

    #[napi]
    pub fn append_right(&mut self, _index: u32, _input: String) -> &Self {
        // self.0.append_right(index as usize, input);
        // self
        unimplemented!()
    }

    #[napi]
    pub fn prepend_left(&mut self, _index: u32, _input: String) -> &Self {
        // self.0.prepend_left(index as usize, input);
        // self
        unimplemented!()
    }

    #[napi]
    pub fn prepend_right(&mut self, _index: u32, _input: String) -> &Self {
        // self.0.prepend_right(index as usize, input);
        // self
        unimplemented!()
    }

    // #[napi]
    // pub fn overwrite(
    // &mut self,
    // start: i64,
    // end: i64,
    // content: String,
    // options: OverwriteOptions,
    // ) -> &Self {
    // self.0.overwrite(start, end, content, options);
    // self
    // }

    // #[napi]
    // pub fn trim(&mut self, pattern: Option<String>) -> &Self {
    // self.0.trim(pattern.as_deref());
    // self
    // }

    // #[napi]
    // pub fn trim_start(&mut self, pattern: Option<String>) -> &Self {
    // self.0.trim_start(pattern.as_deref());
    // self
    // }

    // #[napi]
    // pub fn trim_end(&mut self, pattern: Option<String>) -> &Self {
    // self.0.trim_end(pattern.as_deref());
    // self
    // }

    // #[napi]
    // pub fn trim_lines(&mut self) -> &Self {
    // self.0.trim_lines();
    // self
    // }
}
