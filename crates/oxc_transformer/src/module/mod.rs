use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod amd;
mod cjs;
mod esm;
mod options;
mod umd;

use amd::AMD;
use cjs::CJS;
use esm::ESM;
use umd::UMD;

pub use options::ModuleOptions;

pub struct Module<'a, 'ctx> {
    options: ModuleOptions,
    // Plugins
    cjs: Option<CJS<'a, 'ctx>>,
    amd: Option<AMD<'a, 'ctx>>,
    umd: Option<UMD<'a, 'ctx>>,
    esm: Option<ESM<'a, 'ctx>>,
}

impl<'a, 'ctx> Module<'a, 'ctx> {
    pub fn new(options: ModuleOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            options,
            cjs: if let options::ModuleTarget::CJS = options.target {
                Some(CJS::new(ctx))
            } else {
                None
            },
            amd: if let options::ModuleTarget::AMD = options.target {
                Some(AMD::new(ctx))
            } else {
                None
            },
            umd: if let options::ModuleTarget::UMD = options.target {
                Some(UMD::new(ctx))
            } else {
                None
            },
            esm: if let options::ModuleTarget::Preserve = options.target {
                Some(ESM::new(ctx))
            } else {
                None
            },
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for Module<'a, 'ctx> {}
