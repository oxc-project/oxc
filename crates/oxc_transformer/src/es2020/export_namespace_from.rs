use oxc_ast::VisitMut;

pub struct ExportNamespaceFrom;

impl VisitMut<'_> for ExportNamespaceFrom {}
