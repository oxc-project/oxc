//! Perform the code path analysis

use crate::AstNode;

pub struct CodePathAnalyzer {}

impl<'a> CodePathAnalyzer {
    pub fn new() -> CodePathAnalyzer {
        CodePathAnalyzer {}
    }

    pub fn get_code_path(&self, node: &'a AstNode<'a>) -> &CodePath {
        todo!()
    }
}

pub struct CodePath<'a> {
    segments: Vec<&'a CodeSegment>,
}

impl<'a> CodePath<'a> {
    fn new() -> CodePath<'a> {
        CodePath { segments: todo!() }
    }
    fn get_segments(&self) -> &Vec<&CodeSegment> {
        &self.segments
    }
}

pub struct CodeSegment {
    id: String,
}
