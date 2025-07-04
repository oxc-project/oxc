use oxc_traverse::Traverse;
use serde::Deserialize;

use crate::{context::TransformCtx, state::TransformState};

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct StyledComponentsOptions {}

impl Default for StyledComponentsOptions {
    fn default() -> Self {
        Self {}
    }
}

pub struct StyledComponents<'a, 'ctx> {
    pub options: StyledComponentsOptions,
    pub ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> StyledComponents<'a, 'ctx> {
    pub fn new(options: StyledComponentsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { options, ctx }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for StyledComponents<'a, '_> {}
