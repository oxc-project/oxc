//! Conversion context tracking for ESTree to oxc AST conversion.

/// Context information used during AST node conversion to determine
/// the correct oxc AST node type (e.g., BindingIdentifier vs IdentifierReference).
#[derive(Debug, Clone)]
pub struct ConversionContext {
    /// Type of the parent node (e.g., "VariableDeclarator", "MemberExpression")
    pub parent_type: Option<String>,
    /// Name of the field this node is in (e.g., "id", "property", "callee")
    pub field_name: Option<String>,
    /// Stack of parent types and field names for nested contexts
    pub parent_stack: Vec<(String, String)>,
    /// Whether this is in a shorthand property context
    pub is_shorthand: bool,
    /// Whether this is in a computed property context
    pub is_computed: bool,
    /// Whether this is in a binding context (declaration, not assignment)
    pub is_binding_context: bool,
}

impl ConversionContext {
    /// Create a new empty conversion context.
    pub fn new() -> Self {
        Self {
            parent_type: None,
            field_name: None,
            parent_stack: Vec::new(),
            is_shorthand: false,
            is_computed: false,
            is_binding_context: false,
        }
    }

    /// Create a new context with a parent node.
    pub fn with_parent(mut self, parent_type: &str, field_name: &str) -> Self {
        let mut path = self.parent_stack.clone();
        path.push((parent_type.to_string(), field_name.to_string()));
        Self {
            parent_type: Some(parent_type.to_string()),
            field_name: Some(field_name.to_string()),
            parent_stack: path,
            is_shorthand: self.is_shorthand,
            is_computed: self.is_computed,
            is_binding_context: self.is_binding_context,
        }
    }

    /// Check if the current context is an assignment context.
    pub fn is_assignment_context(&self) -> bool {
        self.parent_type.as_deref() == Some("AssignmentExpression")
            && self.field_name.as_deref() == Some("left")
    }

    /// Check if the current context is a binding context.
    pub fn is_binding_context(&self) -> bool {
        self.is_binding_context
            || matches!(
                (self.parent_type.as_deref(), self.field_name.as_deref()),
                (Some("VariableDeclarator"), Some("id"))
                    | (Some("FunctionDeclaration"), Some("id"))
                    | (Some("FunctionExpression"), Some("id"))
                    | (Some("ClassDeclaration"), Some("id"))
                    | (Some("ClassExpression"), Some("id"))
                    | (Some("CatchClause"), Some("param"))
                    | (Some("ForInStatement"), Some("left"))
                    | (Some("ForOfStatement"), Some("left"))
                    | (Some("ObjectPattern"), Some("properties"))
                    | (Some("ArrayPattern"), Some("elements"))
                    | (Some("RestElement"), Some("argument"))
                    | (Some("AssignmentPattern"), Some("left"))
            )
    }

    /// Check if the current context is a property context.
    pub fn is_property_context(&self) -> bool {
        matches!(
            (self.parent_type.as_deref(), self.field_name.as_deref()),
            (Some("Property"), Some("key"))
                | (Some("MemberExpression"), Some("property"))
                | (Some("MethodDefinition"), Some("key"))
                | (Some("PropertyDefinition"), Some("key"))
                | (Some("ExportSpecifier"), Some("exported"))
                | (Some("ImportSpecifier"), Some("imported"))
                | (Some("ImportDefaultSpecifier"), Some("local"))
                | (Some("ImportNamespaceSpecifier"), Some("local"))
        )
    }

    /// Check if the current context is a label context.
    pub fn is_label_context(&self) -> bool {
        matches!(
            (self.parent_type.as_deref(), self.field_name.as_deref()),
            (Some("LabeledStatement"), Some("label"))
                | (Some("BreakStatement"), Some("label"))
                | (Some("ContinueStatement"), Some("label"))
        )
    }
}

impl Default for ConversionContext {
    fn default() -> Self {
        Self::new()
    }
}

