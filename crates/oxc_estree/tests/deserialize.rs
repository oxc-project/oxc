//! Tests for ESTree to oxc AST deserialization.

#[cfg(feature = "deserialize")]
mod identifier {
    use oxc_estree::deserialize::{
        ConversionContext, EstreeIdentifier, IdentifierKind, convert_identifier,
        get_identifier_span,
    };

    #[test]
    fn test_identifier_kind_binding() {
        let estree_id = EstreeIdentifier {
            name: "x".to_string(),
            range: Some([0, 1]),
            _oxc_identifierKind: None,
        };

        let context = ConversionContext::new().with_parent("VariableDeclarator", "id");

        let kind = convert_identifier(&estree_id, &context, "").unwrap();
        assert_eq!(kind, IdentifierKind::Binding);
    }

    #[test]
    fn test_identifier_kind_reference() {
        let estree_id = EstreeIdentifier {
            name: "x".to_string(),
            range: Some([0, 1]),
            _oxc_identifierKind: None,
        };

        let context = ConversionContext::new().with_parent("CallExpression", "callee");

        let kind = convert_identifier(&estree_id, &context, "").unwrap();
        assert_eq!(kind, IdentifierKind::Reference);
    }

    #[test]
    fn test_identifier_kind_name() {
        let estree_id = EstreeIdentifier {
            name: "prop".to_string(),
            range: Some([0, 4]),
            _oxc_identifierKind: None,
        };

        let mut context = ConversionContext::new().with_parent("MemberExpression", "property");
        context.is_computed = false;

        let kind = convert_identifier(&estree_id, &context, "").unwrap();
        assert_eq!(kind, IdentifierKind::Name);
    }

    #[test]
    fn test_identifier_kind_label() {
        let estree_id = EstreeIdentifier {
            name: "loop".to_string(),
            range: Some([0, 4]),
            _oxc_identifierKind: None,
        };

        let context = ConversionContext::new().with_parent("LabeledStatement", "label");

        let kind = convert_identifier(&estree_id, &context, "").unwrap();
        assert_eq!(kind, IdentifierKind::Label);
    }

    #[test]
    fn test_identifier_hint_priority() {
        // Hint should take priority over context
        let estree_id = EstreeIdentifier {
            name: "x".to_string(),
            range: Some([0, 1]),
            _oxc_identifierKind: Some("name".to_string()),
        };

        let context = ConversionContext::new().with_parent("VariableDeclarator", "id");

        let kind = convert_identifier(&estree_id, &context, "").unwrap();
        assert_eq!(kind, IdentifierKind::Name); // Hint takes priority
    }

    #[test]
    fn test_identifier_span() {
        let estree_id = EstreeIdentifier {
            name: "x".to_string(),
            range: Some([10, 15]),
            _oxc_identifierKind: None,
        };

        let span = get_identifier_span(&estree_id);
        assert_eq!(span, (10, 15));
    }

    #[test]
    fn test_identifier_span_no_range() {
        let estree_id =
            EstreeIdentifier { name: "x".to_string(), range: None, _oxc_identifierKind: None };

        let span = get_identifier_span(&estree_id);
        assert_eq!(span, (0, 0));
    }
}

#[cfg(feature = "deserialize")]
mod literals {
    use oxc_estree::deserialize::{
        EstreeLiteral, LiteralKind, convert_literal, get_boolean_value, get_literal_span,
        get_numeric_value, get_string_value,
    };
    use serde_json::{Number, Value};

    #[test]
    fn test_literal_kind_boolean() {
        let estree_literal =
            EstreeLiteral { value: Value::Bool(true), raw: None, range: Some([0, 4]) };

        let kind = convert_literal(&estree_literal).unwrap();
        assert_eq!(kind, LiteralKind::Boolean);
    }

    #[test]
    fn test_literal_kind_numeric() {
        let estree_literal = EstreeLiteral {
            value: Value::Number(Number::from(42)),
            raw: Some("42".to_string()),
            range: Some([0, 2]),
        };

        let kind = convert_literal(&estree_literal).unwrap();
        assert_eq!(kind, LiteralKind::Numeric);
    }

    #[test]
    fn test_literal_kind_string() {
        let estree_literal = EstreeLiteral {
            value: Value::String("hello".to_string()),
            raw: Some("\"hello\"".to_string()),
            range: Some([0, 7]),
        };

        let kind = convert_literal(&estree_literal).unwrap();
        assert_eq!(kind, LiteralKind::String);
    }

    #[test]
    fn test_literal_kind_null() {
        let estree_literal = EstreeLiteral {
            value: Value::Null,
            raw: Some("null".to_string()),
            range: Some([0, 4]),
        };

        let kind = convert_literal(&estree_literal).unwrap();
        assert_eq!(kind, LiteralKind::Null);
    }

    #[test]
    fn test_get_boolean_value() {
        let estree_literal =
            EstreeLiteral { value: Value::Bool(false), raw: None, range: Some([0, 5]) };

        let value = get_boolean_value(&estree_literal).unwrap();
        assert_eq!(value, false);
    }

    #[test]
    fn test_get_numeric_value() {
        let estree_literal = EstreeLiteral {
            value: Value::Number(Number::from_f64(123.45).unwrap()),
            raw: Some("123.45".to_string()),
            range: Some([0, 6]),
        };

        let value = get_numeric_value(&estree_literal).unwrap();
        assert_eq!(value, 123.45);
    }

    #[test]
    fn test_get_string_value() {
        let estree_literal = EstreeLiteral {
            value: Value::String("test".to_string()),
            raw: Some("\"test\"".to_string()),
            range: Some([0, 6]),
        };

        let value = get_string_value(&estree_literal).unwrap();
        assert_eq!(value, "test");
    }

    #[test]
    fn test_literal_span() {
        let estree_literal = EstreeLiteral {
            value: Value::Number(Number::from(42)),
            raw: Some("42".to_string()),
            range: Some([10, 12]),
        };

        let span = get_literal_span(&estree_literal);
        assert_eq!(span, (10, 12));
    }
}

#[cfg(feature = "deserialize")]
mod context {
    use oxc_estree::deserialize::ConversionContext;

    #[test]
    fn test_assignment_context() {
        let context = ConversionContext::new().with_parent("AssignmentExpression", "left");

        assert!(context.is_assignment_context());
    }

    #[test]
    fn test_binding_context() {
        let context = ConversionContext::new().with_parent("VariableDeclarator", "id");

        assert!(context.is_binding_context());
    }

    #[test]
    fn test_property_context() {
        let context = ConversionContext::new().with_parent("MemberExpression", "property");

        assert!(context.is_property_context());
    }

    #[test]
    fn test_label_context() {
        let context = ConversionContext::new().with_parent("LabeledStatement", "label");

        assert!(context.is_label_context());
    }

    #[test]
    fn test_context_with_parent_stack() {
        let context = ConversionContext::new()
            .with_parent("Program", "body")
            .with_parent("VariableDeclaration", "declarations")
            .with_parent("VariableDeclarator", "id");

        assert!(context.is_binding_context());
        assert_eq!(context.parent_stack.len(), 3);
    }
}

#[cfg(feature = "deserialize")]
mod patterns {
    use oxc_estree::deserialize::{ConversionContext, PatternTargetKind, determine_pattern_kind};
    use serde_json::json;

    #[test]
    fn test_pattern_assignment_context() {
        let context = ConversionContext::new().with_parent("AssignmentExpression", "left");

        let estree_node = json!({"type": "Identifier", "name": "x"});

        let kind = determine_pattern_kind(&estree_node, &context).unwrap();
        assert_eq!(kind, PatternTargetKind::AssignmentTarget);
    }

    #[test]
    fn test_pattern_binding_context() {
        let context = ConversionContext::new().with_parent("VariableDeclarator", "id");

        let estree_node = json!({"type": "Identifier", "name": "x"});

        let kind = determine_pattern_kind(&estree_node, &context).unwrap();
        assert_eq!(kind, PatternTargetKind::Pattern);
    }

    #[test]
    fn test_pattern_default() {
        let context = ConversionContext::new();

        let estree_node = json!({"type": "Identifier", "name": "x"});

        let kind = determine_pattern_kind(&estree_node, &context).unwrap();
        assert_eq!(kind, PatternTargetKind::Pattern); // Default to pattern
    }
}

#[cfg(feature = "deserialize")]
mod converter {
    use oxc_estree::deserialize::{ConversionError, EstreeConverter};
    use serde_json::json;

    #[test]
    fn test_validate_program() {
        let converter = EstreeConverter::new("");
        let estree = json!({
            "type": "Program",
            "body": []
        });

        let result = converter.validate_program(&estree);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_program_invalid_type() {
        let converter = EstreeConverter::new("");
        let estree = json!({
            "type": "ExpressionStatement",
            "expression": {}
        });

        let result = converter.validate_program(&estree);
        assert!(matches!(result, Err(ConversionError::UnsupportedNodeType { .. })));
    }

    #[test]
    fn test_validate_program_missing_type() {
        let converter = EstreeConverter::new("");
        let estree = json!({
            "body": []
        });

        let result = converter.validate_program(&estree);
        assert!(matches!(result, Err(ConversionError::MissingField { .. })));
    }
}
