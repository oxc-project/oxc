#![allow(clippy::cast_possible_truncation)]
// use std::sync::Arc;

use napi_derive::napi;

// use oxc_sourcemap::napi::SourceMap;
use self_cell::self_cell;
use string_wizard::MagicString as MS;

#[napi]
pub struct MagicString {
    cell: MagicStringImpl,
}

self_cell!(
    struct MagicStringImpl {
        owner: String,
        #[covariant]
        dependent: MS,
    }
);

impl MagicString {
    pub fn new(source_text: String) -> Self {
        Self { cell: MagicStringImpl::new(source_text, |s| string_wizard::MagicString::new(s)) }
    }
}

#[napi(object)]
pub struct OverwriteOptions {
    pub content_only: bool,
}

#[napi(object)]
pub struct SourceMapOptions {
    pub include_content: Option<bool>,
    pub source: Option<String>,
    pub hires: Option<bool>,
}

#[napi]
impl MagicString {
    #[napi]
    pub fn get_source_text(&self, start: u32, end: u32) -> &str {
        &self.cell.borrow_owner()[start as usize..end as usize]
    }

    #[napi]
    pub fn length(&self) -> u32 {
        self.cell.borrow_dependent().len() as u32
    }

    #[napi]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.cell.borrow_dependent().to_string()
    }

    // #[napi]
    // pub fn source_map(&self, options: Option<SourceMapOptions>) -> SourceMap {
    // let options = options.map(|o| string_wizard::SourceMapOptions {
    // include_content: o.include_content.unwrap_or_default(),
    // source: o.source.map(Arc::from).unwrap_or_default(),
    // hires: o.hires.unwrap_or_default(),
    // });
    // let map = self.cell.borrow_dependent().source_map(options.unwrap_or_default());
    // oxc_sourcemap::napi::SourceMap::from(map)
    // }

    #[napi]
    pub fn append(&mut self, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append(input);
        });
        self
    }

    #[napi]
    pub fn append_left(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append_left(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn append_right(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append_right(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn indent(&mut self) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.indent();
        });
        self
    }

    #[napi]
    pub fn prepend(&mut self, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend(input);
        });
        self
    }

    #[napi]
    pub fn prepend_left(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend_left(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn prepend_right(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend_right(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn relocate(&mut self, start: u32, end: u32, to: u32) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.relocate(start as usize, end as usize, to as usize);
        });
        self
    }

    #[napi]
    pub fn remove(&mut self, start: u32, end: u32) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.remove(start as usize, end as usize);
        });
        self
    }
}
