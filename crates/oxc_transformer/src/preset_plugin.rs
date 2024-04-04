use oxc_ast::ast::Program;

pub trait Transformation {
    fn transform<'a>(&mut self, _program: &mut Program<'a>) {}
}

pub type BoxedTransformation = Box<dyn Transformation>;
