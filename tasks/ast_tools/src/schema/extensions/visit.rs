/// Details of visiting on a struct.
#[derive(Default, Debug)]
pub struct VisitStruct {
    pub is_visited: bool,
    pub scope: Option<Scope>,
}

/// Details of visiting on an enum.
#[derive(Default, Debug)]
pub struct VisitEnum {
    pub is_visited: bool,
}

/// Details of visiting on a struct field or enum variant.
#[derive(Default, Debug)]
pub struct VisitFieldOrVariant {
    pub visit_args: Option<Vec<(String, String)>>,
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
