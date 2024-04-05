use oxc_ast::VisitMut;

pub struct DynamicImport;

impl VisitMut<'_> for DynamicImport {}
