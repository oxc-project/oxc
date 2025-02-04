/// Details of visiting on a struct.
#[derive(Default, Debug)]
pub struct VisitStruct {
    /// Name of `visit_*` method and `walk_*` function.
    /// `None` if this struct is not visited.
    pub visitor_names: Option<VisitorNames>,
    pub visit_args: Option<Vec<(String, String)>>,
    pub scope: Option<Scope>,
}

impl VisitStruct {
    /// Returns `true` if this struct is visited.
    pub fn is_visited(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get name of visitor method for this struct, if it is visited.
    pub fn visitor_name(&self) -> Option<&str> {
        self.visitor_names.as_ref().map(|names| names.visit.as_str())
    }
}

/// Details of visiting on an enum.
#[derive(Default, Debug)]
pub struct VisitEnum {
    /// Name of `visit_*` method and `walk_*` function.
    /// `None` if this enum is not visited.
    pub visitor_names: Option<VisitorNames>,
}

impl VisitEnum {
    /// Returns `true` if this enum is visited.
    pub fn is_visited(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get name of visitor method for this enum, if it is visited.
    pub fn visitor_name(&self) -> Option<&str> {
        self.visitor_names.as_ref().map(|names| names.visit.as_str())
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
    /// Returns `true` if this `Vec` is visited.
    #[expect(dead_code)]
    pub fn is_visited(&self) -> bool {
        self.visitor_names.is_some()
    }

    /// Get name of visitor method for this `Vec`, if it is visited.
    pub fn visitor_name(&self) -> Option<&str> {
        self.visitor_names.as_ref().map(|names| names.visit.as_str())
    }
}

/// Details of visiting on a struct field or enum variant.
#[derive(Default, Debug)]
pub struct VisitFieldOrVariant {
    pub visit_args: Option<Vec<(String, String)>>,
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
