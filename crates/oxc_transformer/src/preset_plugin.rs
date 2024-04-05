use oxc_ast::ast::Program;

pub trait Transformation {
    fn transform<'a>(&mut self, _program: &mut Program<'a>) {}
}

pub type BoxedTransformation = Box<dyn Transformation>;

#[macro_export]
macro_rules! impl_preset_transformation {
    ($preset:ident) => {
        impl crate::preset_plugin::Transformation for $preset {
            fn transform<'a>(&mut self, program: &mut oxc_ast::ast::Program<'a>) {
                for plugin in &mut self.plugins {
                    plugin.transform(program);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_plugin_transformation {
    ($preset:ident) => {
        impl crate::preset_plugin::Transformation for $preset {
            fn transform<'a>(&mut self, program: &mut oxc_ast::ast::Program<'a>) {
                self.visit_program(program);
            }
        }
    };
}
