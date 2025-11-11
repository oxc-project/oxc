//! Error types for ESTree to oxc AST conversion.

/// Span representation (start, end) as byte offsets.
/// The actual `oxc_span::Span` type will be used in the calling code.
pub type Span = (u32, u32);

pub type ConversionResult<T> = Result<T, ConversionError>;

/// Errors that can occur during ESTree to oxc AST conversion.
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Unsupported ESTree node type encountered
    UnsupportedNodeType { node_type: String, span: Span },
    /// Invalid identifier context for conversion
    InvalidIdentifierContext { context: String, span: Span },
    /// Invalid span information
    InvalidSpan { expected: String, got: String, span: Span },
    /// Pattern conversion error
    PatternConversionError { message: String, span: Span },
    /// Literal conversion error
    LiteralConversionError { message: String, span: Span },
    /// JSON parsing error
    JsonParseError { message: String },
    /// Missing required field
    MissingField { field: String, node_type: String, span: Span },
    /// Invalid field type
    InvalidFieldType { field: String, expected: String, got: String, span: Span },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnsupportedNodeType { node_type, .. } => {
                write!(f, "Unsupported ESTree node type: {}", node_type)
            }
            ConversionError::InvalidIdentifierContext { context, .. } => {
                write!(f, "Invalid identifier context: {}", context)
            }
            ConversionError::InvalidSpan { expected, got, .. } => {
                write!(f, "Invalid span: expected {}, got {}", expected, got)
            }
            ConversionError::PatternConversionError { message, .. } => {
                write!(f, "Pattern conversion error: {}", message)
            }
            ConversionError::LiteralConversionError { message, .. } => {
                write!(f, "Literal conversion error: {}", message)
            }
            ConversionError::JsonParseError { message } => {
                write!(f, "JSON parse error: {}", message)
            }
            ConversionError::MissingField { field, node_type, .. } => {
                write!(f, "Missing required field '{}' in node type '{}'", field, node_type)
            }
            ConversionError::InvalidFieldType { field, expected, got, .. } => {
                write!(f, "Invalid field type for '{}': expected {}, got {}", field, expected, got)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl ConversionError {
    /// Get the span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            ConversionError::UnsupportedNodeType { span, .. }
            | ConversionError::InvalidIdentifierContext { span, .. }
            | ConversionError::InvalidSpan { span, .. }
            | ConversionError::PatternConversionError { span, .. }
            | ConversionError::LiteralConversionError { span, .. }
            | ConversionError::MissingField { span, .. }
            | ConversionError::InvalidFieldType { span, .. } => Some(*span),
            ConversionError::JsonParseError { .. } => None,
        }
    }
}
