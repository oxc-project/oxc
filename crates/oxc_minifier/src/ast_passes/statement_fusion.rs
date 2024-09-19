use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{keep_var::KeepVar, node_util::NodeUtil, tri::Tri, CompressorPass};

/// Statement Fusion
///
/// Tries to fuse all the statements in a block into a one statement by using COMMAs or statements.
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/StatementFusion.java>
pub struct StatementFusion;

impl<'a> CompressorPass<'a> for StatementFusion {}

impl<'a> Traverse<'a> for StatementFusion {}

impl<'a> StatementFusion {
    pub fn new() -> Self {
        Self {}
    }
}
