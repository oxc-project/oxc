//! ESTree type definitions for deserialization.

use serde_json::Value;

/// ESTree node type identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstreeNodeType {
    Program,
    Identifier,
    Literal,
    VariableDeclarator,
    VariableDeclaration,
    FunctionDeclaration,
    FunctionExpression,
    ClassDeclaration,
    ClassExpression,
    MemberExpression,
    CallExpression,
    Property,
    LabeledStatement,
    BreakStatement,
    ContinueStatement,
    AssignmentExpression,
    CatchClause,
    // Export/Import related
    ImportDeclaration,
    ExportNamedDeclaration,
    ExportDefaultDeclaration,
    ExportAllDeclaration,
    ExportSpecifier,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    // Method/Property related
    MethodDefinition,
    PropertyDefinition,
    // Object/Array patterns
    ObjectPattern,
    ArrayPattern,
    RestElement,
    // Assignment patterns
    AssignmentPattern,
    // Expression patterns
    SpreadElement,
    // Other statement types
    BlockStatement,
    ExpressionStatement,
    ReturnStatement,
    IfStatement,
    WhileStatement,
    DoWhileStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    ThrowStatement,
    TryStatement,
    SwitchStatement,
    SwitchCase,
    EmptyStatement,
    DebuggerStatement,
    WithStatement,
    // Expression types
    ArrayExpression,
    ObjectExpression,
    BinaryExpression,
    UnaryExpression,
    UpdateExpression,
    LogicalExpression,
    ConditionalExpression,
    NewExpression,
    SequenceExpression,
    ThisExpression,
    Super,
    YieldExpression,
    AwaitExpression,
    TemplateLiteral,
    TaggedTemplateExpression,
    TemplateElement,
    ArrowFunctionExpression,
    ChainExpression,
    ParenthesizedExpression,
    ImportExpression,
    MetaProperty,
    PrivateIdentifier,
    AccessorProperty,
    StaticBlock,
    // TypeScript declarations
    TSInterfaceDeclaration,
    TSEnumDeclaration,
    TSTypeAliasDeclaration,
    TSModuleDeclaration,
    // TypeScript expressions
    TSAsExpression,
    TSSatisfiesExpression,
    TSNonNullExpression,
    TSInstantiationExpression,
    TSTypeAssertion,
    // TypeScript module declarations
    TSImportEqualsDeclaration,
    TSExportAssignment,
    TSNamespaceExportDeclaration,
    // Other
    Unknown(String),
}

impl From<&str> for EstreeNodeType {
    fn from(s: &str) -> Self {
        match s {
            "Program" => EstreeNodeType::Program,
            "Identifier" => EstreeNodeType::Identifier,
            "Literal" => EstreeNodeType::Literal,
            "VariableDeclarator" => EstreeNodeType::VariableDeclarator,
            "VariableDeclaration" => EstreeNodeType::VariableDeclaration,
            "FunctionDeclaration" => EstreeNodeType::FunctionDeclaration,
            "FunctionExpression" => EstreeNodeType::FunctionExpression,
            "ClassDeclaration" => EstreeNodeType::ClassDeclaration,
            "ClassExpression" => EstreeNodeType::ClassExpression,
            "MemberExpression" => EstreeNodeType::MemberExpression,
            "CallExpression" => EstreeNodeType::CallExpression,
            "ArrayExpression" => EstreeNodeType::ArrayExpression,
            "ObjectExpression" => EstreeNodeType::ObjectExpression,
            "Property" => EstreeNodeType::Property,
            "LabeledStatement" => EstreeNodeType::LabeledStatement,
            "BlockStatement" => EstreeNodeType::BlockStatement,
            "BreakStatement" => EstreeNodeType::BreakStatement,
            "ContinueStatement" => EstreeNodeType::ContinueStatement,
            "ExpressionStatement" => EstreeNodeType::ExpressionStatement,
            "ReturnStatement" => EstreeNodeType::ReturnStatement,
            "IfStatement" => EstreeNodeType::IfStatement,
            "WhileStatement" => EstreeNodeType::WhileStatement,
            "DoWhileStatement" => EstreeNodeType::DoWhileStatement,
            "ForStatement" => EstreeNodeType::ForStatement,
            "ForInStatement" => EstreeNodeType::ForInStatement,
            "ForOfStatement" => EstreeNodeType::ForOfStatement,
            "EmptyStatement" => EstreeNodeType::EmptyStatement,
            "ThisExpression" => EstreeNodeType::ThisExpression,
            "NewExpression" => EstreeNodeType::NewExpression,
            "AssignmentExpression" => EstreeNodeType::AssignmentExpression,
            "CatchClause" => EstreeNodeType::CatchClause,
            "ImportDeclaration" => EstreeNodeType::ImportDeclaration,
            "ExportNamedDeclaration" => EstreeNodeType::ExportNamedDeclaration,
            "ExportDefaultDeclaration" => EstreeNodeType::ExportDefaultDeclaration,
            "ExportAllDeclaration" => EstreeNodeType::ExportAllDeclaration,
            "ExportSpecifier" => EstreeNodeType::ExportSpecifier,
            "ImportSpecifier" => EstreeNodeType::ImportSpecifier,
            "ImportDefaultSpecifier" => EstreeNodeType::ImportDefaultSpecifier,
            "ImportNamespaceSpecifier" => EstreeNodeType::ImportNamespaceSpecifier,
            "MethodDefinition" => EstreeNodeType::MethodDefinition,
            "PropertyDefinition" => EstreeNodeType::PropertyDefinition,
            "ObjectPattern" => EstreeNodeType::ObjectPattern,
            "ArrayPattern" => EstreeNodeType::ArrayPattern,
            "RestElement" => EstreeNodeType::RestElement,
            "AssignmentPattern" => EstreeNodeType::AssignmentPattern,
            "SpreadElement" => EstreeNodeType::SpreadElement,
            "ThrowStatement" => EstreeNodeType::ThrowStatement,
            "TryStatement" => EstreeNodeType::TryStatement,
            "SwitchStatement" => EstreeNodeType::SwitchStatement,
            "SwitchCase" => EstreeNodeType::SwitchCase,
            "DebuggerStatement" => EstreeNodeType::DebuggerStatement,
            "WithStatement" => EstreeNodeType::WithStatement,
            "BinaryExpression" => EstreeNodeType::BinaryExpression,
            "UnaryExpression" => EstreeNodeType::UnaryExpression,
            "UpdateExpression" => EstreeNodeType::UpdateExpression,
            "LogicalExpression" => EstreeNodeType::LogicalExpression,
            "ConditionalExpression" => EstreeNodeType::ConditionalExpression,
            "SequenceExpression" => EstreeNodeType::SequenceExpression,
            "Super" => EstreeNodeType::Super,
            "YieldExpression" => EstreeNodeType::YieldExpression,
            "AwaitExpression" => EstreeNodeType::AwaitExpression,
            "TemplateLiteral" => EstreeNodeType::TemplateLiteral,
            "TaggedTemplateExpression" => EstreeNodeType::TaggedTemplateExpression,
            "TemplateElement" => EstreeNodeType::TemplateElement,
            "ArrowFunctionExpression" => EstreeNodeType::ArrowFunctionExpression,
            "ChainExpression" => EstreeNodeType::ChainExpression,
            "ParenthesizedExpression" => EstreeNodeType::ParenthesizedExpression,
            "ImportExpression" => EstreeNodeType::ImportExpression,
            "MetaProperty" => EstreeNodeType::MetaProperty,
            "AccessorProperty" => EstreeNodeType::AccessorProperty,
            "StaticBlock" => EstreeNodeType::StaticBlock,
            "TSInterfaceDeclaration" => EstreeNodeType::TSInterfaceDeclaration,
            "TSEnumDeclaration" => EstreeNodeType::TSEnumDeclaration,
            "TSTypeAliasDeclaration" => EstreeNodeType::TSTypeAliasDeclaration,
            "TSModuleDeclaration" => EstreeNodeType::TSModuleDeclaration,
            "TSAsExpression" => EstreeNodeType::TSAsExpression,
            "TSSatisfiesExpression" => EstreeNodeType::TSSatisfiesExpression,
            "TSNonNullExpression" => EstreeNodeType::TSNonNullExpression,
            "TSInstantiationExpression" => EstreeNodeType::TSInstantiationExpression,
            "TSTypeAssertion" => EstreeNodeType::TSTypeAssertion,
            "TSImportEqualsDeclaration" => EstreeNodeType::TSImportEqualsDeclaration,
            "TSExportAssignment" => EstreeNodeType::TSExportAssignment,
            "TSNamespaceExportDeclaration" => EstreeNodeType::TSNamespaceExportDeclaration,
            _ => EstreeNodeType::Unknown(s.to_string()),
        }
    }
}

/// Helper trait for extracting ESTree node information.
pub trait EstreeNode {
    /// Get the node type from a JSON value.
    fn get_type(value: &Value) -> Option<EstreeNodeType> {
        value.get("type")?.as_str().map(EstreeNodeType::from)
    }

    /// Get the range from a JSON value.
    fn get_range(value: &Value) -> Option<[usize; 2]> {
        let range = value.get("range")?.as_array()?;
        if range.len() >= 2 {
            Some([
                range[0].as_u64()? as usize,
                range[1].as_u64()? as usize,
            ])
        } else {
            None
        }
    }

    /// Get a string field from a JSON value.
    fn get_string(value: &Value, field: &str) -> Option<String> {
        value.get(field)?.as_str().map(String::from)
    }

    /// Get a boolean field from a JSON value.
    fn get_bool(value: &Value, field: &str) -> Option<bool> {
        value.get(field)?.as_bool()
    }

    /// Get an optional string field (for hints like `_oxc_identifierKind`).
    #[allow(non_snake_case)]
    fn get_optional_string(value: &Value, field: &str) -> Option<String> {
        value.get(field)?.as_str().map(String::from)
    }
}

impl EstreeNode for Value {}

/// ESTree Identifier node representation.
#[derive(Debug, Clone)]
#[allow(non_snake_case, clippy::pub_underscore_fields)] // _oxc_identifierKind uses camelCase for JavaScript compatibility
pub struct EstreeIdentifier {
    pub name: String,
    pub range: Option<[usize; 2]>,
    /// Optional hint for identifier kind (from JavaScript preprocessing)
    /// Format: `_oxc_identifierKind` (designed for potential future standardization)
    pub _oxc_identifierKind: Option<String>,
}

impl EstreeIdentifier {
    /// Parse an Identifier node from JSON.
    pub fn from_json(value: &Value) -> Option<Self> {
        Some(Self {
            name: <Value as EstreeNode>::get_string(value, "name")?,
            range: <Value as EstreeNode>::get_range(value),
            _oxc_identifierKind: <Value as EstreeNode>::get_optional_string(value, "_oxc_identifierKind"),
        })
    }
}

/// ESTree Literal node representation.
#[derive(Debug, Clone)]
pub struct EstreeLiteral {
    pub value: Value,
    pub raw: Option<String>,
    pub range: Option<[usize; 2]>,
}

impl EstreeLiteral {
    /// Parse a Literal node from JSON.
    pub fn from_json(value: &Value) -> Option<Self> {
        Some(Self {
            value: value.get("value")?.clone(),
            raw: <Value as EstreeNode>::get_string(value, "raw"),
            range: <Value as EstreeNode>::get_range(value),
        })
    }
}

