use oxc_ast::VisitMut;

use crate::impl_plugin_transformation;

pub struct DynamicImport;

impl_plugin_transformation!(DynamicImport);

impl VisitMut<'_> for DynamicImport {}
