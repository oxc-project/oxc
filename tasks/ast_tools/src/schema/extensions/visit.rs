use syn::Ident;

use crate::utils::create_safe_ident;

/// Details of visiting on a struct.
#[derive(Default, Debug)]
pub struct VisitStruct {
    /// Name of `visit_*` method and `walk_*` function.
    /// `None` if this struct is not visited.
    pub visitor_names: Option<VisitorNames>,
    pub visit_args: Vec<(String, String)>,
    pub scope: Option<Scope>,
    /// `true` if this type has a scope, or any of its fields contain a scope.
    pub contains_scope: bool,
}

impl VisitStruct {
    /// Returns `true` if this struct has a visitor.
    pub fn has_visitor(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get visitor method name for this struct as an [`Ident`], if it has a visitor.
    ///
    /// [`Ident`]: struct@Ident
    pub fn visitor_ident(&self) -> Option<Ident> {
        self.visitor_names.as_ref().map(VisitorNames::visitor_ident)
    }
}

/// Details of visiting on an enum.
#[derive(Default, Debug)]
pub struct VisitEnum {
    /// Name of `visit_*` method and `walk_*` function.
    /// `None` if this enum is not visited.
    pub visitor_names: Option<VisitorNames>,
    /// `true` if any variants contain a scope.
    pub contains_scope: bool,
}

impl VisitEnum {
    /// Returns `true` if this enum has a visitor.
    pub fn has_visitor(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get visitor method name for this enum as an [`Ident`], if it has a visitor.
    ///
    /// [`Ident`]: struct@Ident
    pub fn visitor_ident(&self) -> Option<Ident> {
        self.visitor_names.as_ref().map(VisitorNames::visitor_ident)
    }
}

/// Details of visiting on a `Vec`.
#[derive(Default, Debug)]
pub struct VisitVec {
    /// Name of `visit_*` method and `walk_*` function.
    /// `None` if this `Vec` does not have a visitor.
    pub visitor_names: Option<VisitorNames>,
}

impl VisitVec {
    /// Returns `true` if this `Vec` has a visitor.
    #[expect(dead_code)]
    pub fn has_visitor(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get visitor method name for this `Vec` as an [`Ident`], if it has a visitor.
    ///
    /// [`Ident`]: struct@Ident
    pub fn visitor_ident(&self) -> Option<Ident> {
        self.visitor_names.as_ref().map(VisitorNames::visitor_ident)
    }
}

/// Details of visiting on a struct field or enum variant.
#[derive(Default, Debug)]
pub struct VisitFieldOrVariant {
    pub visit_args: Vec<(String, String)>,
}

/// Names for visitor method and walk function.
#[derive(Debug)]
pub struct VisitorNames {
    pub visit: String,
    pub walk: String,
}

impl VisitorNames {
    pub fn from_snake_name(snake_name: &str) -> Self {
        Self { visit: format!("visit_{snake_name}"), walk: format!("walk_{snake_name}") }
    }

    /// Get name of visitor method as an [`Ident`].
    ///
    /// [`Ident`]: struct@Ident
    pub fn visitor_ident(&self) -> Ident {
        // Visitor method names cannot be reserved words, as they begin with `visit_`
        create_safe_ident(&self.visit)
    }

    /// Get name of walk function as an [`Ident`].
    ///
    /// [`Ident`]: struct@Ident
    pub fn walk_ident(&self) -> Ident {
        // Walk function names cannot be reserved words, as they begin with `walk_`
        create_safe_ident(&self.walk)
    }
}

/// Details of scope on a struct.
#[derive(Debug)]
pub struct Scope {
    /// Field index before which scope is entered
    pub enter_before_index: usize,
    /// Field index before which scope is exited.
    /// If scope is exited after last field, this is `struct_def.fields.len()`.
    pub exit_before_index: usize,
    /// Scope flags for the scope.
    /// Stored as a string which should be parsed as an expression.
    pub flags: String,
    /// Conditions in which scope is strict mode.
    /// Stored as a string which should be parsed as an expression.
    pub strict_if: Option<String>,
}
