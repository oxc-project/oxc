use oxc_ast::VisitMut;

use crate::impl_plugin_transformation;

pub struct ExportNamespaceFrom;

impl_plugin_transformation!(ExportNamespaceFrom);

impl VisitMut<'_> for ExportNamespaceFrom {}
