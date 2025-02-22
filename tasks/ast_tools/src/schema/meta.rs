use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, parse_str};

use crate::utils::create_ident;

use super::{File, FileId, MetaId, Schema, extensions::estree::ESTreeMeta};

/// Definition for a meta type.
///
/// Meta types are types which are not part of the AST, but are associated with the AST in some way,
/// and used by `oxc_ast_tools` as "helpers", or used in generated code.
#[derive(Debug)]
pub struct MetaType {
    #[expect(dead_code)]
    pub id: MetaId,
    pub name: String,
    pub file_id: FileId,
    pub estree: ESTreeMeta,
}

impl MetaType {
    /// Create new [`MetaType`].
    pub fn new(id: MetaId, name: String, file_id: FileId) -> Self {
        Self { id, name, file_id, estree: ESTreeMeta::default() }
    }

    /// Get meta type name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get meta type name as an [`Ident`].
    ///
    /// [`Ident`]: struct@Ident
    pub fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get the [`File`] which this meta type is defined in.
    pub fn file<'s>(&self, schema: &'s Schema) -> &'s File {
        &schema.files[self.file_id]
    }

    /// Get the import path for this meta type from specified crate.
    ///
    /// e.g. `crate::serialize::Null` or `oxc_ast::serialize::Null`.
    pub fn import_path_from_crate(&self, from_krate: &str, schema: &Schema) -> TokenStream {
        let file = self.file(schema);

        let mut path = if file.krate() == from_krate {
            quote!(crate)
        } else {
            let crate_ident = create_ident(file.krate());
            quote!(#crate_ident)
        };

        if !file.import_path().is_empty() {
            let import_path: Path = parse_str(file.import_path()).unwrap();
            path.extend(quote!( #import_path ));
        }

        let ident = self.ident();
        path.extend(quote!( ::#ident ));

        path
    }
}
