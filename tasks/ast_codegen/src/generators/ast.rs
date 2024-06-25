use quote::ToTokens;

use crate::{CodegenCtx, Generator, GeneratorOutput};

pub struct AstGenerator;

impl Generator for AstGenerator {
    fn name(&self) -> &'static str {
        "AstGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let output =
            ctx.modules.iter().map(|it| (it.module.clone(), it.to_token_stream())).collect();
        GeneratorOutput::Many(output)
    }
}
