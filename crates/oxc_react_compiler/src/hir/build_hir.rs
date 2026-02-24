use oxc_ast::ast;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    compiler_error::{
        CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
        GENERATED_SOURCE, SourceLocation,
    },
    hir::{
        ArrayExpressionElement, BlockKind, BranchTerminal, Case, DeclareContext, DeclareLocal,
        Destructure, DoWhileTerminal, ForInTerminal, ForOfTerminal, ForTerminal,
        FunctionExpressionType, FunctionExpressionValue, GetIterator, GotoTerminal, GotoVariant,
        HIRFunction, IfTerminal, Instruction, InstructionId, InstructionKind, InstructionValue,
        IteratorNext, LValue, LValuePattern, LabelTerminal, LoadContext, LoadLocal,
        LogicalTerminal, LoweredFunction, NextPropertyOf, NonLocalBinding, ObjectPatternProperty,
        ObjectProperty, ObjectPropertyKey, ObjectPropertyType, OptionalTerminal, PrimitiveValue,
        PrimitiveValueKind, ReactFunctionType, ReturnVariant, SequenceTerminal, SpreadPattern,
        StoreContext, StoreGlobal, StoreLocal, SwitchTerminal, TemplateLiteralQuasi, Terminal,
        TernaryTerminal, TryTerminal, WhileTerminal,
        environment::Environment,
        find_context_identifiers::{find_context_identifiers, find_context_identifiers_arrow},
        hir_builder::{BindingKind, HirBuilder, VariableBinding, create_temporary_place},
        hir_types::{
            ArrayPattern as HirArrayPattern, ArrayPatternElement, IdentifierName,
            ObjectPattern as HirObjectPattern, Pattern,
        },
    },
};

use super::lower_ast::{convert_expression, convert_statement};

/// A function that can be lowered to HIR.
pub enum LowerableFunction<'a> {
    Function(&'a ast::Function<'a>),
    ArrowFunction(&'a ast::ArrowFunctionExpression<'a>),
}

/// Lower the parameters of a function to HIR reactive params.
///
/// This registers each parameter name in the builder's bindings map so that
/// later identifier references can resolve to the correct local binding.
///
/// For destructured parameters (ObjectPattern, ArrayPattern, AssignmentPattern),
/// this also emits Destructure/StoreLocal instructions into the function body
/// to extract the individual bindings from the parameter place. This matches
/// the TS reference which calls `lowerAssignment()` for destructured params
/// (BuildHIR.ts lines 130-151).
///
/// # Errors
/// Returns a `CompilerError` if destructuring lowering fails.
fn lower_params(
    builder: &mut HirBuilder,
    func: &LowerableFunction<'_>,
) -> Result<Vec<crate::hir::ReactiveParam>, CompilerError> {
    let params = match func {
        LowerableFunction::Function(f) => &f.params,
        LowerableFunction::ArrowFunction(a) => &a.params,
    };

    let mut result = Vec::new();
    for param in &params.items {
        let loc = span_to_loc(param.span);
        match &param.pattern {
            ast::BindingPattern::BindingIdentifier(ident) => {
                let place = builder.declare_binding(&ident.name, BindingKind::Param, loc);
                result.push(crate::hir::ReactiveParam::Place(place));
            }
            // Destructured parameters: create a promoted temporary for the overall param,
            // then emit Destructure instructions to extract individual bindings.
            // This matches the TS reference (BuildHIR.ts lines 130-151).
            ast::BindingPattern::ObjectPattern(_)
            | ast::BindingPattern::ArrayPattern(_)
            | ast::BindingPattern::AssignmentPattern(_) => {
                let place = create_promoted_temporary(builder, loc);
                result.push(crate::hir::ReactiveParam::Place(place.clone()));
                lower_destructuring_declaration(
                    builder,
                    &param.pattern,
                    place,
                    InstructionKind::Let,
                    BindingKind::Let,
                    loc,
                )?;
            }
        }
    }
    if let Some(rest) = &params.rest {
        let loc = span_to_loc(rest.span);
        match &rest.rest.argument {
            ast::BindingPattern::BindingIdentifier(ident) => {
                let place = builder.declare_binding(&ident.name, BindingKind::Param, loc);
                result.push(crate::hir::ReactiveParam::Spread(SpreadPattern { place }));
            }
            // Destructured rest parameter
            _ => {
                let place = create_promoted_temporary(builder, loc);
                result.push(crate::hir::ReactiveParam::Spread(SpreadPattern {
                    place: place.clone(),
                }));
                lower_destructuring_declaration(
                    builder,
                    &rest.rest.argument,
                    place,
                    InstructionKind::Let,
                    BindingKind::Let,
                    loc,
                )?;
            }
        }
    }
    Ok(result)
}

/// Declare all leaf bindings in a pattern, emitting DeclareLocal/DeclareContext
/// for each leaf identifier. Used for variable declarations without initializers.
fn declare_all_bindings_in_pattern(
    builder: &mut HirBuilder,
    pattern: &ast::BindingPattern<'_>,
    binding_kind: BindingKind,
    instruction_kind: InstructionKind,
    decl_loc: SourceLocation,
    loc: SourceLocation,
) {
    match pattern {
        ast::BindingPattern::BindingIdentifier(ident) => {
            let decl_place = builder.declare_binding(&ident.name, binding_kind, decl_loc);
            if builder.is_context_identifier(&ident.name) {
                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue,
                    value: InstructionValue::DeclareContext(DeclareContext {
                        lvalue_kind: InstructionKind::Let,
                        lvalue_place: decl_place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
            } else {
                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue,
                    value: InstructionValue::DeclareLocal(DeclareLocal {
                        lvalue: LValue { place: decl_place, kind: instruction_kind },
                        loc,
                    }),
                    effects: None,
                    loc,
                });
            }
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                declare_all_bindings_in_pattern(
                    builder,
                    &prop.value,
                    binding_kind,
                    instruction_kind,
                    decl_loc,
                    loc,
                );
            }
            if let Some(rest) = &obj.rest {
                declare_all_bindings_in_pattern(
                    builder,
                    &rest.argument,
                    binding_kind,
                    instruction_kind,
                    decl_loc,
                    loc,
                );
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                declare_all_bindings_in_pattern(
                    builder,
                    elem,
                    binding_kind,
                    instruction_kind,
                    decl_loc,
                    loc,
                );
            }
            if let Some(rest) = &arr.rest {
                declare_all_bindings_in_pattern(
                    builder,
                    &rest.argument,
                    binding_kind,
                    instruction_kind,
                    decl_loc,
                    loc,
                );
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            declare_all_bindings_in_pattern(
                builder,
                &assign.left,
                binding_kind,
                instruction_kind,
                decl_loc,
                loc,
            );
        }
    }
}

/// Create a promoted temporary place. Promoted temporaries have a name like `#t0`
/// so they appear as named variables in the HIR, which is needed for destructuring
/// pattern elements that are nested patterns (not simple identifiers).
fn create_promoted_temporary(builder: &mut HirBuilder, loc: SourceLocation) -> crate::hir::Place {
    let id = builder.environment_mut().next_identifier_id();
    let identifier = crate::hir::Identifier {
        id,
        declaration_id: crate::hir::DeclarationId(id.0),
        name: Some(IdentifierName::Promoted(format!("#t{}", id.0))),
        mutable_range: crate::hir::MutableRange::default(),
        scope: None,
        type_: crate::hir::types::make_type(),
        loc,
    };
    crate::hir::Place { identifier, effect: crate::hir::Effect::Unknown, reactive: false, loc }
}

/// Lower a destructuring declaration (ObjectPattern or ArrayPattern) to HIR.
///
/// This is the core of destructuring support. For each pattern, it:
/// 1. Walks the pattern to build an HIR Pattern (ObjectPattern or ArrayPattern)
/// 2. For leaf identifiers, declares them in the builder's bindings map
/// 3. For nested patterns or patterns with defaults, creates promoted temporaries
/// 4. Emits a Destructure instruction
/// 5. Recursively processes followups (nested patterns assigned to temporaries)
///
/// Port of the `ArrayPattern` and `ObjectPattern` cases in `lowerAssignment()` from
/// `HIR/BuildHIR.ts`.
fn lower_destructuring_declaration(
    builder: &mut HirBuilder,
    pattern: &ast::BindingPattern<'_>,
    value: crate::hir::Place,
    kind: InstructionKind,
    binding_kind: BindingKind,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    match pattern {
        ast::BindingPattern::ObjectPattern(obj_pat) => {
            lower_object_destructuring(builder, obj_pat, value, kind, binding_kind, loc)
        }
        ast::BindingPattern::ArrayPattern(arr_pat) => {
            lower_array_destructuring(builder, arr_pat, value, kind, binding_kind, loc)
        }
        ast::BindingPattern::AssignmentPattern(assign_pat) => {
            // AssignmentPattern: `const { a = defaultVal } = obj`
            // Lower the default value conditionally, then recursively lower the left side
            lower_assignment_pattern_declaration(
                builder,
                assign_pat,
                value,
                kind,
                binding_kind,
                loc,
            )
        }
        ast::BindingPattern::BindingIdentifier(ident) => {
            // Base case: simple identifier — declare and emit store
            let ident_loc = span_to_loc(ident.span);
            let decl_place = builder.declare_binding(&ident.name, binding_kind, ident_loc);

            if builder.is_context_identifier(&ident.name) {
                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue,
                    value: InstructionValue::StoreContext(StoreContext {
                        lvalue_kind: kind,
                        lvalue_place: decl_place,
                        value,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
            } else {
                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: decl_place, kind },
                        value,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
            }
            Ok(())
        }
    }
}

/// Lower an ObjectPattern destructuring declaration.
///
/// Port of the `ObjectPattern` case in `lowerAssignment()` from BuildHIR.ts.
fn lower_object_destructuring(
    builder: &mut HirBuilder,
    obj_pat: &ast::ObjectPattern<'_>,
    value: crate::hir::Place,
    kind: InstructionKind,
    binding_kind: BindingKind,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    let pat_loc = span_to_loc(obj_pat.span);
    let mut properties = Vec::new();
    let mut followups: Vec<(crate::hir::Place, FollowupPattern)> = Vec::new();

    for prop in &obj_pat.properties {
        let prop_loc = span_to_loc(prop.span);
        let key = lower_binding_property_key(builder, &prop.key)?;

        // Get the value pattern
        match &prop.value {
            ast::BindingPattern::BindingIdentifier(ident) => {
                // Simple identifier: declare it directly and use its place in the pattern
                let ident_loc = span_to_loc(ident.span);
                let place = builder.declare_binding(&ident.name, binding_kind, ident_loc);
                properties.push(ObjectPatternProperty::Property(ObjectProperty {
                    key,
                    property_type: ObjectPropertyType::Property,
                    place,
                }));
            }
            ast::BindingPattern::AssignmentPattern(assign) => {
                // Property with default: `{ a = defaultVal }` or `{ key: val = defaultVal }`
                // Create a promoted temporary and handle the default in followups
                let temp = create_promoted_temporary(builder, prop_loc);
                properties.push(ObjectPatternProperty::Property(ObjectProperty {
                    key,
                    property_type: ObjectPropertyType::Property,
                    place: temp.clone(),
                }));
                followups.push((temp, FollowupPattern::AssignmentPattern(assign)));
            }
            // Nested pattern (object or array): create a temporary and recurse
            nested @ (ast::BindingPattern::ObjectPattern(_)
            | ast::BindingPattern::ArrayPattern(_)) => {
                let temp = create_promoted_temporary(builder, prop_loc);
                properties.push(ObjectPatternProperty::Property(ObjectProperty {
                    key,
                    property_type: ObjectPropertyType::Property,
                    place: temp.clone(),
                }));
                followups.push((temp, FollowupPattern::Binding(nested)));
            }
        }
    }

    // Handle rest element: `const { a, ...rest } = obj`
    if let Some(rest) = &obj_pat.rest {
        let rest_loc = span_to_loc(rest.span);
        match &rest.argument {
            ast::BindingPattern::BindingIdentifier(ident) => {
                let ident_loc = span_to_loc(ident.span);
                let place = builder.declare_binding(&ident.name, binding_kind, ident_loc);
                properties.push(ObjectPatternProperty::Spread(SpreadPattern { place }));
            }
            nested => {
                let temp = create_promoted_temporary(builder, rest_loc);
                properties
                    .push(ObjectPatternProperty::Spread(SpreadPattern { place: temp.clone() }));
                followups.push((temp, FollowupPattern::Binding(nested)));
            }
        }
    }

    // Emit the Destructure instruction
    lower_value_to_temporary(
        builder,
        InstructionValue::Destructure(Destructure {
            lvalue: LValuePattern {
                kind,
                pattern: Pattern::Object(HirObjectPattern { properties, loc: pat_loc }),
            },
            value,
            loc,
        }),
        loc,
    )?;

    // Process followups: recursively lower nested patterns
    for (temp_place, followup) in followups {
        match followup {
            FollowupPattern::Binding(nested_pat) => {
                lower_destructuring_declaration(
                    builder,
                    nested_pat,
                    temp_place,
                    kind,
                    binding_kind,
                    loc,
                )?;
            }
            FollowupPattern::AssignmentPattern(assign_pat) => {
                lower_assignment_pattern_declaration(
                    builder,
                    assign_pat,
                    temp_place,
                    kind,
                    binding_kind,
                    loc,
                )?;
            }
        }
    }

    Ok(())
}

/// Lower an ArrayPattern destructuring declaration.
///
/// Port of the `ArrayPattern` case in `lowerAssignment()` from BuildHIR.ts.
fn lower_array_destructuring(
    builder: &mut HirBuilder,
    arr_pat: &ast::ArrayPattern<'_>,
    value: crate::hir::Place,
    kind: InstructionKind,
    binding_kind: BindingKind,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    let pat_loc = span_to_loc(arr_pat.span);
    let mut items = Vec::new();
    let mut followups: Vec<(crate::hir::Place, FollowupPattern)> = Vec::new();

    for element in &arr_pat.elements {
        match element {
            None => {
                // Hole: `const [, x] = arr`
                items.push(ArrayPatternElement::Hole);
            }
            Some(binding) => {
                let elem_loc = span_to_loc(binding.span());
                match binding {
                    ast::BindingPattern::BindingIdentifier(ident) => {
                        // Simple identifier element
                        let ident_loc = span_to_loc(ident.span);
                        let place = builder.declare_binding(&ident.name, binding_kind, ident_loc);
                        items.push(ArrayPatternElement::Place(place));
                    }
                    ast::BindingPattern::AssignmentPattern(assign) => {
                        // Element with default: `const [a = 1] = arr`
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, FollowupPattern::AssignmentPattern(assign)));
                    }
                    // Nested pattern
                    nested @ (ast::BindingPattern::ObjectPattern(_)
                    | ast::BindingPattern::ArrayPattern(_)) => {
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, FollowupPattern::Binding(nested)));
                    }
                }
            }
        }
    }

    // Handle rest element: `const [first, ...rest] = arr`
    if let Some(rest) = &arr_pat.rest {
        let rest_loc = span_to_loc(rest.span);
        match &rest.argument {
            ast::BindingPattern::BindingIdentifier(ident) => {
                let ident_loc = span_to_loc(ident.span);
                let place = builder.declare_binding(&ident.name, binding_kind, ident_loc);
                items.push(ArrayPatternElement::Spread(SpreadPattern { place }));
            }
            nested => {
                let temp = create_promoted_temporary(builder, rest_loc);
                items.push(ArrayPatternElement::Spread(SpreadPattern { place: temp.clone() }));
                followups.push((temp, FollowupPattern::Binding(nested)));
            }
        }
    }

    // Emit the Destructure instruction
    lower_value_to_temporary(
        builder,
        InstructionValue::Destructure(Destructure {
            lvalue: LValuePattern {
                kind,
                pattern: Pattern::Array(HirArrayPattern { items, loc: pat_loc }),
            },
            value,
            loc,
        }),
        loc,
    )?;

    // Process followups: recursively lower nested patterns
    for (temp_place, followup) in followups {
        match followup {
            FollowupPattern::Binding(nested_pat) => {
                lower_destructuring_declaration(
                    builder,
                    nested_pat,
                    temp_place,
                    kind,
                    binding_kind,
                    loc,
                )?;
            }
            FollowupPattern::AssignmentPattern(assign_pat) => {
                lower_assignment_pattern_declaration(
                    builder,
                    assign_pat,
                    temp_place,
                    kind,
                    binding_kind,
                    loc,
                )?;
            }
        }
    }

    Ok(())
}

/// A followup pattern to process after the initial Destructure instruction.
/// This handles nested destructuring and default values.
enum FollowupPattern<'a> {
    /// A nested binding pattern to destructure further.
    Binding(&'a ast::BindingPattern<'a>),
    /// An assignment pattern (default value) to handle conditionally.
    AssignmentPattern(&'a ast::AssignmentPattern<'a>),
}

/// Lower an AssignmentPattern (default value in destructuring).
///
/// Port of the `AssignmentPattern` case in `lowerAssignment()` from BuildHIR.ts.
///
/// `const { a = defaultVal } = obj` or `const [x = defaultVal] = arr`
///
/// Creates a conditional: `value === undefined ? defaultVal : value`,
/// then recursively lowers the left side of the pattern.
fn lower_assignment_pattern_declaration(
    builder: &mut HirBuilder,
    assign_pat: &ast::AssignmentPattern<'_>,
    value: crate::hir::Place,
    kind: InstructionKind,
    binding_kind: BindingKind,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    let pat_loc = span_to_loc(assign_pat.span);

    // Create a temporary to hold the resolved value (either the provided value or the default)
    let temp = create_promoted_temporary(builder, pat_loc);

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());

    let continuation_id = continuation_block.id;

    // Consequent: use the default value (when value === undefined)
    let default_expr = convert_expression(&assign_pat.right);
    let consequent_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        if let Ok(result) = lower_expression(builder, &default_expr) {
            let lvalue = create_temporary_place(builder.environment_mut(), pat_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                    value: result.place,
                    loc: pat_loc,
                }),
                effects: None,
                loc: pat_loc,
            });
        } else {
            // If default lowering fails, store undefined
            let undef_place = create_temporary_place(builder.environment_mut(), pat_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: undef_place.clone(),
                value: lower_undefined(pat_loc),
                effects: None,
                loc: pat_loc,
            });
            let lvalue = create_temporary_place(builder.environment_mut(), pat_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                    value: undef_place,
                    loc: pat_loc,
                }),
                effects: None,
                loc: pat_loc,
            });
        }
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc: pat_loc,
        })
    });

    // Alternate: use the provided value (when value !== undefined)
    let alternate_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let lvalue = create_temporary_place(builder.environment_mut(), pat_loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue,
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                value: value.clone(),
                loc: pat_loc,
            }),
            effects: None,
            loc: pat_loc,
        });
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc: pat_loc,
        })
    });

    // Emit the ternary terminal
    builder.terminate_with_continuation(
        Terminal::Ternary(TernaryTerminal {
            id: InstructionId(0),
            test: test_block.id,
            fallthrough: continuation_id,
            loc: pat_loc,
        }),
        test_block,
    );

    // In the test block: compare value === undefined
    let undef = lower_value_to_temporary(builder, lower_undefined(pat_loc), pat_loc)?;
    let test_result = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
            operator: oxc_syntax::operator::BinaryOperator::StrictEquality,
            left: value,
            right: undef.place,
            loc: pat_loc,
        }),
        pat_loc,
    )?;

    builder.terminate_with_continuation(
        Terminal::Branch(BranchTerminal {
            id: InstructionId(0),
            test: test_result.place,
            consequent: consequent_block,
            alternate: alternate_block,
            fallthrough: continuation_id,
            loc: pat_loc,
        }),
        continuation_block,
    );

    // Now recursively lower the left side of the assignment pattern with the resolved temp value
    lower_destructuring_declaration(builder, &assign_pat.left, temp, kind, binding_kind, loc)
}

/// Lower a property key from a binding property (used in object destructuring).
///
/// Port of `lowerObjectPropertyKey()` from `HIR/BuildHIR.ts` (lines 1508-1543).
/// Handles static identifiers, string literals, numeric literals, and computed keys.
fn lower_binding_property_key(
    builder: &mut HirBuilder,
    key: &ast::PropertyKey<'_>,
) -> Result<ObjectPropertyKey, CompilerError> {
    match key {
        ast::PropertyKey::StaticIdentifier(ident) => {
            Ok(ObjectPropertyKey::Identifier(ident.name.to_string()))
        }
        ast::PropertyKey::StringLiteral(lit) => {
            Ok(ObjectPropertyKey::String(lit.value.to_string()))
        }
        ast::PropertyKey::NumericLiteral(lit) => Ok(ObjectPropertyKey::Number(lit.value)),
        _ => {
            // Computed key: lower the expression to get a Place
            let expr = key.to_expression();
            let lowerable = convert_expression(expr);
            let result = lower_expression(builder, &lowerable)?;
            Ok(ObjectPropertyKey::Computed(result.place))
        }
    }
}

/// Lower an object destructuring assignment target: `({ a, b } = expr)`.
///
/// This emits a Destructure instruction with `InstructionKind::Reassign`.
fn lower_object_assignment_target(
    builder: &mut HirBuilder,
    target: &ast::ObjectAssignmentTarget<'_>,
    value: crate::hir::Place,
    target_loc: SourceLocation,
    loc: SourceLocation,
) -> Result<ExpressionResult, CompilerError> {
    let mut properties = Vec::new();
    let mut followups: Vec<(crate::hir::Place, AssignmentFollowup)> = Vec::new();

    for prop in &target.properties {
        match prop {
            ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident_prop) => {
                // Shorthand: `{ a }` or `{ a = defaultVal }`
                let name = &ident_prop.binding.name;
                let ident_loc = span_to_loc(ident_prop.binding.span);
                let key = ObjectPropertyKey::Identifier(name.to_string());

                if ident_prop.init.is_some() {
                    // Has default value: create a promoted temporary and defer
                    let temp = create_promoted_temporary(builder, ident_loc);
                    properties.push(ObjectPatternProperty::Property(ObjectProperty {
                        key,
                        property_type: ObjectPropertyType::Property,
                        place: temp.clone(),
                    }));
                    followups.push((
                        temp,
                        AssignmentFollowup::IdentifierWithDefault {
                            name: name.to_string(),
                            default_expr: ident_prop.init.as_ref(),
                            loc: ident_loc,
                        },
                    ));
                } else {
                    // No default: resolve identifier directly
                    let place = resolve_identifier_for_reassignment(builder, name, ident_loc);
                    properties.push(ObjectPatternProperty::Property(ObjectProperty {
                        key,
                        property_type: ObjectPropertyType::Property,
                        place,
                    }));
                }
            }
            ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop_prop) => {
                let key = lower_binding_property_key(builder, &prop_prop.name)?;
                let prop_loc = span_to_loc(prop_prop.span);

                match &prop_prop.binding {
                    ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                        let place = resolve_identifier_for_reassignment(
                            builder,
                            &ident.name,
                            span_to_loc(ident.span),
                        );
                        properties.push(ObjectPatternProperty::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place,
                        }));
                    }
                    ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
                        with_default,
                    ) => {
                        let temp = create_promoted_temporary(builder, prop_loc);
                        properties.push(ObjectPatternProperty::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: temp.clone(),
                        }));
                        followups.push((
                            temp,
                            AssignmentFollowup::TargetWithDefault {
                                target: &with_default.binding,
                                default_expr: Some(&with_default.init),
                                loc: prop_loc,
                            },
                        ));
                    }
                    // Nested destructuring targets
                    ast::AssignmentTargetMaybeDefault::ObjectAssignmentTarget(nested_obj) => {
                        let temp = create_promoted_temporary(builder, prop_loc);
                        properties.push(ObjectPatternProperty::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: temp.clone(),
                        }));
                        followups.push((temp, AssignmentFollowup::NestedObject(nested_obj)));
                    }
                    ast::AssignmentTargetMaybeDefault::ArrayAssignmentTarget(nested_arr) => {
                        let temp = create_promoted_temporary(builder, prop_loc);
                        properties.push(ObjectPatternProperty::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: temp.clone(),
                        }));
                        followups.push((temp, AssignmentFollowup::NestedArray(nested_arr)));
                    }
                    // Simple member expressions or other targets
                    _ => {
                        let temp = create_promoted_temporary(builder, prop_loc);
                        properties.push(ObjectPatternProperty::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: temp.clone(),
                        }));
                        followups
                            .push((temp, AssignmentFollowup::SimpleTarget(&prop_prop.binding)));
                    }
                }
            }
        }
    }

    // Handle rest element
    if let Some(rest) = &target.rest {
        let rest_loc = span_to_loc(rest.span);
        if let ast::AssignmentTarget::AssignmentTargetIdentifier(ident) = &rest.target {
            let place =
                resolve_identifier_for_reassignment(builder, &ident.name, span_to_loc(ident.span));
            properties.push(ObjectPatternProperty::Spread(SpreadPattern { place }));
        } else {
            let temp = create_promoted_temporary(builder, rest_loc);
            properties.push(ObjectPatternProperty::Spread(SpreadPattern { place: temp.clone() }));
            followups.push((temp, AssignmentFollowup::AssignmentTarget(&rest.target)));
        }
    }

    // Emit the Destructure instruction
    let result = lower_value_to_temporary(
        builder,
        InstructionValue::Destructure(Destructure {
            lvalue: LValuePattern {
                kind: InstructionKind::Reassign,
                pattern: Pattern::Object(HirObjectPattern { properties, loc: target_loc }),
            },
            value,
            loc,
        }),
        loc,
    )?;

    // Process followups
    process_assignment_followups(builder, followups, loc)?;

    Ok(result)
}

/// Lower an array destructuring assignment target: `([a, b] = expr)`.
///
/// This emits a Destructure instruction with `InstructionKind::Reassign`.
fn lower_array_assignment_target(
    builder: &mut HirBuilder,
    target: &ast::ArrayAssignmentTarget<'_>,
    value: crate::hir::Place,
    target_loc: SourceLocation,
    loc: SourceLocation,
) -> Result<ExpressionResult, CompilerError> {
    let mut items = Vec::new();
    let mut followups: Vec<(crate::hir::Place, AssignmentFollowup)> = Vec::new();

    for element in &target.elements {
        match element {
            None => {
                items.push(ArrayPatternElement::Hole);
            }
            Some(target_maybe_default) => {
                let elem_loc = span_to_loc(target_maybe_default.span());
                match target_maybe_default {
                    ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                        let place = resolve_identifier_for_reassignment(
                            builder,
                            &ident.name,
                            span_to_loc(ident.span),
                        );
                        items.push(ArrayPatternElement::Place(place));
                    }
                    ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
                        with_default,
                    ) => {
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((
                            temp,
                            AssignmentFollowup::TargetWithDefault {
                                target: &with_default.binding,
                                default_expr: Some(&with_default.init),
                                loc: elem_loc,
                            },
                        ));
                    }
                    ast::AssignmentTargetMaybeDefault::ObjectAssignmentTarget(nested_obj) => {
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, AssignmentFollowup::NestedObject(nested_obj)));
                    }
                    ast::AssignmentTargetMaybeDefault::ArrayAssignmentTarget(nested_arr) => {
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, AssignmentFollowup::NestedArray(nested_arr)));
                    }
                    _ => {
                        let temp = create_promoted_temporary(builder, elem_loc);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups
                            .push((temp, AssignmentFollowup::SimpleTarget(target_maybe_default)));
                    }
                }
            }
        }
    }

    // Handle rest element
    if let Some(rest) = &target.rest {
        let rest_loc = span_to_loc(rest.span);
        if let ast::AssignmentTarget::AssignmentTargetIdentifier(ident) = &rest.target {
            let place =
                resolve_identifier_for_reassignment(builder, &ident.name, span_to_loc(ident.span));
            items.push(ArrayPatternElement::Spread(SpreadPattern { place }));
        } else {
            let temp = create_promoted_temporary(builder, rest_loc);
            items.push(ArrayPatternElement::Spread(SpreadPattern { place: temp.clone() }));
            followups.push((temp, AssignmentFollowup::AssignmentTarget(&rest.target)));
        }
    }

    // Emit the Destructure instruction
    let result = lower_value_to_temporary(
        builder,
        InstructionValue::Destructure(Destructure {
            lvalue: LValuePattern {
                kind: InstructionKind::Reassign,
                pattern: Pattern::Array(HirArrayPattern { items, loc: target_loc }),
            },
            value,
            loc,
        }),
        loc,
    )?;

    // Process followups
    process_assignment_followups(builder, followups, loc)?;

    Ok(result)
}

/// An assignment followup to process after the initial Destructure instruction.
enum AssignmentFollowup<'a> {
    /// An identifier with a default value (e.g., `{ a = 1 }` in `{ a = 1 } = obj`).
    IdentifierWithDefault {
        name: String,
        default_expr: Option<&'a ast::Expression<'a>>,
        loc: SourceLocation,
    },
    /// An assignment target with an optional default value.
    TargetWithDefault {
        target: &'a ast::AssignmentTarget<'a>,
        default_expr: Option<&'a ast::Expression<'a>>,
        loc: SourceLocation,
    },
    /// A nested object destructuring target.
    NestedObject(&'a ast::ObjectAssignmentTarget<'a>),
    /// A nested array destructuring target.
    NestedArray(&'a ast::ArrayAssignmentTarget<'a>),
    /// A simple assignment target (member expression etc.).
    SimpleTarget(&'a ast::AssignmentTargetMaybeDefault<'a>),
    /// A nested assignment target (for rest elements etc.).
    AssignmentTarget(&'a ast::AssignmentTarget<'a>),
}

/// Process a list of assignment followups after a Destructure instruction.
fn process_assignment_followups(
    builder: &mut HirBuilder,
    followups: Vec<(crate::hir::Place, AssignmentFollowup<'_>)>,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    for (temp_place, followup) in followups {
        match followup {
            AssignmentFollowup::IdentifierWithDefault { name, default_expr, loc: ident_loc } => {
                // Resolve the value through a conditional default
                let resolved = if let Some(default) = default_expr {
                    lower_conditional_default_assignment(builder, temp_place, default, ident_loc)?
                } else {
                    temp_place
                };
                // Assign to the identifier
                let lowerable = LowerableExpression::Identifier(name, Span::default());
                lower_assignment(builder, &lowerable, resolved, loc)?;
            }
            AssignmentFollowup::TargetWithDefault { target, default_expr, loc: target_loc } => {
                let resolved = if let Some(default) = default_expr {
                    lower_conditional_default_assignment(builder, temp_place, default, target_loc)?
                } else {
                    temp_place
                };
                let lowerable = convert_assignment_target_to_lowerable(target);
                lower_assignment(builder, &lowerable, resolved, loc)?;
            }
            AssignmentFollowup::NestedObject(nested) => {
                let nested_loc = span_to_loc(nested.span);
                lower_object_assignment_target(builder, nested, temp_place, nested_loc, loc)?;
            }
            AssignmentFollowup::NestedArray(nested) => {
                let nested_loc = span_to_loc(nested.span);
                lower_array_assignment_target(builder, nested, temp_place, nested_loc, loc)?;
            }
            AssignmentFollowup::SimpleTarget(target_maybe_default) => {
                if let ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) =
                    target_maybe_default
                {
                    let lowerable =
                        LowerableExpression::Identifier(ident.name.to_string(), ident.span);
                    lower_assignment(builder, &lowerable, temp_place, loc)?;
                } else {
                    // For other simple targets (member expressions), convert and assign
                    let lowerable = LowerableExpression::Undefined(Span::default());
                    lower_assignment(builder, &lowerable, temp_place, loc)?;
                }
            }
            AssignmentFollowup::AssignmentTarget(target) => {
                let lowerable = convert_assignment_target_to_lowerable(target);
                lower_assignment(builder, &lowerable, temp_place, loc)?;
            }
        }
    }
    Ok(())
}

/// Lower a conditional default value for assignment destructuring.
///
/// `value === undefined ? defaultVal : value`
fn lower_conditional_default_assignment(
    builder: &mut HirBuilder,
    value: crate::hir::Place,
    default_expr: &ast::Expression<'_>,
    loc: SourceLocation,
) -> Result<crate::hir::Place, CompilerError> {
    let temp = create_promoted_temporary(builder, loc);

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    let default_lowerable = convert_expression(default_expr);

    // Consequent: use the default value
    let consequent_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        if let Ok(result) = lower_expression(builder, &default_lowerable) {
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                    value: result.place,
                    loc,
                }),
                effects: None,
                loc,
            });
        }
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc,
        })
    });

    // Alternate: use the provided value
    let alternate_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let lvalue = create_temporary_place(builder.environment_mut(), loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue,
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                value: value.clone(),
                loc,
            }),
            effects: None,
            loc,
        });
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc,
        })
    });

    // Emit the ternary terminal
    builder.terminate_with_continuation(
        Terminal::Ternary(TernaryTerminal {
            id: InstructionId(0),
            test: test_block.id,
            fallthrough: continuation_id,
            loc,
        }),
        test_block,
    );

    // In the test block: compare value === undefined
    let undef = lower_value_to_temporary(builder, lower_undefined(loc), loc)?;
    let test_result = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
            operator: oxc_syntax::operator::BinaryOperator::StrictEquality,
            left: value,
            right: undef.place,
            loc,
        }),
        loc,
    )?;

    builder.terminate_with_continuation(
        Terminal::Branch(BranchTerminal {
            id: InstructionId(0),
            test: test_result.place,
            consequent: consequent_block,
            alternate: alternate_block,
            fallthrough: continuation_id,
            loc,
        }),
        continuation_block,
    );

    Ok(temp)
}

/// Convert an `AssignmentTarget` to a `LowerableExpression` for use in `lower_assignment`.
fn convert_assignment_target_to_lowerable<'a>(
    target: &'a ast::AssignmentTarget<'a>,
) -> LowerableExpression<'a> {
    use super::lower_ast::convert_expression;

    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            LowerableExpression::Identifier(ident.name.to_string(), ident.span)
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            LowerableExpression::PropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: member.property.name.to_string(),
                span: member.span,
            }
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            LowerableExpression::ComputedPropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: Box::new(convert_expression(&member.expression)),
                span: member.span,
            }
        }
        ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
            LowerableExpression::ObjectAssignmentTarget { target: obj, span: obj.span }
        }
        ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
            LowerableExpression::ArrayAssignmentTarget { target: arr, span: arr.span }
        }
        _ => LowerableExpression::Undefined(target.span()),
    }
}

/// Resolve an identifier for reassignment in a destructuring assignment.
/// Returns a Place for the identifier, declaring it if needed.
fn resolve_identifier_for_reassignment(
    builder: &mut HirBuilder,
    name: &str,
    loc: SourceLocation,
) -> crate::hir::Place {
    match builder.resolve_identifier(name) {
        VariableBinding::Identifier { identifier, .. } => crate::hir::Place {
            identifier,
            effect: crate::hir::Effect::Unknown,
            reactive: false,
            loc,
        },
        VariableBinding::NonLocal(_) => {
            // Global: create a temporary (the actual StoreGlobal will be
            // handled in the followup via lower_assignment)
            create_temporary_place(builder.environment_mut(), loc)
        }
    }
}

/// Lower the body of a function to HIR, returning extracted directives.
///
/// For arrow functions with `expression: true`, the single expression body is
/// lowered as an implicit return. For block bodies, each statement is lowered
/// and directives are extracted.
///
/// # Errors
/// Returns a `CompilerError` if any statement or expression cannot be lowered.
fn lower_body(
    builder: &mut HirBuilder,
    func: &LowerableFunction<'_>,
) -> Result<Vec<String>, CompilerError> {
    match func {
        LowerableFunction::ArrowFunction(arrow) if arrow.expression => {
            // Arrow expression body: `() => expr`
            // In oxc_ast, the body has one ExpressionStatement wrapping the return expression.
            if let Some(ast::Statement::ExpressionStatement(expr_stmt)) =
                arrow.body.statements.first()
            {
                let lowerable = convert_expression(&expr_stmt.expression);
                let expr_result = lower_expression(builder, &lowerable)?;
                let fallthrough = builder.reserve(BlockKind::Block);
                builder.terminate_with_continuation(
                    Terminal::Return(crate::hir::ReturnTerminal {
                        id: InstructionId(0),
                        value: expr_result.place,
                        return_variant: ReturnVariant::Implicit,
                        loc: GENERATED_SOURCE,
                    }),
                    fallthrough,
                );
            }
            Ok(Vec::new())
        }
        LowerableFunction::Function(f) => {
            if let Some(body) = &f.body {
                let stmts: Vec<_> = body.statements.iter().map(convert_statement).collect();
                lower_block_statement(builder, &stmts)?;
                let directives = body.directives.iter().map(|d| d.directive.to_string()).collect();
                Ok(directives)
            } else {
                Ok(Vec::new())
            }
        }
        LowerableFunction::ArrowFunction(arrow) => {
            // Block body arrow: `() => { ... }`
            let stmts: Vec<_> = arrow.body.statements.iter().map(convert_statement).collect();
            lower_block_statement(builder, &stmts)?;
            let directives =
                arrow.body.directives.iter().map(|d| d.directive.to_string()).collect();
            Ok(directives)
        }
    }
}

/// Extract function metadata (id, generator, is_async) from an AST function node.
fn extract_function_metadata(func: &LowerableFunction<'_>) -> (Option<String>, bool, bool) {
    match func {
        LowerableFunction::Function(f) => {
            (f.id.as_ref().map(|id| id.name.to_string()), f.generator, f.r#async)
        }
        LowerableFunction::ArrowFunction(a) => (None, false, a.r#async),
    }
}

/// Extract the source location span from a function node.
fn extract_function_loc(func: &LowerableFunction<'_>) -> SourceLocation {
    match func {
        LowerableFunction::Function(f) => span_to_loc(f.span),
        LowerableFunction::ArrowFunction(a) => span_to_loc(a.span),
    }
}

/// Lower an oxc AST function into HIR.
///
/// Port of `lower()` from `HIR/BuildHIR.ts` (lines 72-264).
///
/// # Errors
/// Returns a `CompilerError` if lowering fails due to unsupported syntax.
pub fn lower(
    env: &Environment,
    fn_type: ReactFunctionType,
    func: &LowerableFunction<'_>,
) -> Result<HIRFunction, CompilerError> {
    // Find context identifiers (variables captured by inner closures)
    let context_identifiers = match func {
        LowerableFunction::Function(f) => find_context_identifiers(f),
        LowerableFunction::ArrowFunction(a) => find_context_identifiers_arrow(a),
    };

    let mut builder = HirBuilder::new(env.clone(), None, context_identifiers);

    // Extract function metadata
    let (id, generator, is_async) = extract_function_metadata(func);
    let func_loc = extract_function_loc(func);

    // Walk parameters
    let params = lower_params(&mut builder, func)?;

    // Walk body statements and extract directives
    let directives = lower_body(&mut builder, func)?;

    // Emit final void return (matches TS: builder.terminate({ kind: 'return', returnVariant: 'Void', ... }))
    let void_value = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
    builder.push(Instruction {
        id: InstructionId(0),
        lvalue: void_value.clone(),
        value: lower_undefined(GENERATED_SOURCE),
        effects: None,
        loc: GENERATED_SOURCE,
    });
    builder.terminate(
        Terminal::Return(crate::hir::ReturnTerminal {
            id: InstructionId(0),
            value: void_value,
            return_variant: ReturnVariant::Void,
            loc: GENERATED_SOURCE,
        }),
        None,
    );

    let (body, mut built_env) = builder.build_with_env()?;
    let returns = create_temporary_place(&mut built_env, func_loc);

    Ok(HIRFunction {
        loc: func_loc,
        id,
        name_hint: None,
        fn_type,
        env: built_env,
        params,
        returns,
        context: Vec::new(),
        body,
        generator,
        is_async,
        directives,
        aliasing_effects: None,
    })
}

// =====================================================================================
// Function expression lowering (recursive lowering for inner functions)
// =====================================================================================

/// Gather the captured context of an inner function.
///
/// This corresponds to `gatherCapturedContext()` in the TS reference.
/// For each identifier referenced by the inner function, check if it exists
/// as a binding in the outer builder. If so, it is a captured context variable.
///
/// Returns a map from variable name to source location for each captured variable.
fn gather_captured_context(
    func: &LowerableFunction<'_>,
    outer_builder: &HirBuilder,
) -> rustc_hash::FxHashMap<String, SourceLocation> {
    // Collect all identifiers referenced by the inner function
    // We use a simple approach: walk the inner function body collecting all
    // Identifier references, then intersect with outer builder's bindings.
    let mut captured = rustc_hash::FxHashMap::default();

    // Get all binding names from the outer builder
    // Walk the inner function body to find free variable references
    let inner_refs = collect_inner_function_references(func);

    for (name, loc) in inner_refs {
        // Check if the name is bound in the outer function's scope
        if let VariableBinding::Identifier { .. } = outer_builder.resolve_identifier(&name) {
            captured.insert(name, loc);
        }
    }

    captured
}

/// Collect all identifier references from an inner function that could be
/// captured from an outer scope.
///
/// This does a simple walk of the function body, returning all identifiers
/// that are not declared within the function itself.
fn collect_inner_function_references(
    func: &LowerableFunction<'_>,
) -> Vec<(String, SourceLocation)> {
    use rustc_hash::FxHashSet;

    let mut refs = Vec::new();
    let mut declared = FxHashSet::default();

    // Declare the function's own parameters
    let params = match func {
        LowerableFunction::Function(f) => &f.params,
        LowerableFunction::ArrowFunction(a) => &a.params,
    };
    for param in &params.items {
        collect_binding_pattern_names(&param.pattern, &mut declared);
    }
    if let Some(rest) = &params.rest {
        collect_binding_pattern_names(&rest.rest.argument, &mut declared);
    }

    // For function expressions, the function's own name is also declared
    if let LowerableFunction::Function(f) = func
        && let Some(id) = &f.id
    {
        declared.insert(id.name.to_string());
    }

    // Walk the body collecting references
    match func {
        LowerableFunction::Function(f) => {
            if let Some(body) = &f.body {
                for stmt in &body.statements {
                    collect_statement_refs(stmt, &mut refs, &mut declared);
                }
            }
        }
        LowerableFunction::ArrowFunction(a) => {
            for stmt in &a.body.statements {
                collect_statement_refs(stmt, &mut refs, &mut declared);
            }
        }
    }

    // Filter out names that were declared within the inner function
    refs.into_iter().filter(|(name, _)| !declared.contains(name)).collect()
}

/// Collect binding names from a binding pattern.
fn collect_binding_pattern_names(
    pattern: &ast::BindingPattern<'_>,
    names: &mut rustc_hash::FxHashSet<String>,
) {
    match pattern {
        ast::BindingPattern::BindingIdentifier(ident) => {
            names.insert(ident.name.to_string());
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_pattern_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_pattern_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_pattern_names(&assign.left, names);
        }
    }
}

/// Collect identifier references from a statement, also tracking local declarations.
fn collect_statement_refs(
    stmt: &ast::Statement<'_>,
    refs: &mut Vec<(String, SourceLocation)>,
    declared: &mut rustc_hash::FxHashSet<String>,
) {
    match stmt {
        ast::Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    collect_expression_refs(init, refs);
                }
                collect_binding_pattern_names(&declarator.id, declared);
            }
        }
        ast::Statement::ExpressionStatement(expr) => {
            collect_expression_refs(&expr.expression, refs);
        }
        ast::Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                collect_expression_refs(arg, refs);
            }
        }
        ast::Statement::IfStatement(if_stmt) => {
            collect_expression_refs(&if_stmt.test, refs);
            collect_statement_refs(&if_stmt.consequent, refs, declared);
            if let Some(alt) = &if_stmt.alternate {
                collect_statement_refs(alt, refs, declared);
            }
        }
        ast::Statement::WhileStatement(while_stmt) => {
            collect_expression_refs(&while_stmt.test, refs);
            collect_statement_refs(&while_stmt.body, refs, declared);
        }
        ast::Statement::DoWhileStatement(do_while) => {
            collect_statement_refs(&do_while.body, refs, declared);
            collect_expression_refs(&do_while.test, refs);
        }
        ast::Statement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                match init {
                    ast::ForStatementInit::VariableDeclaration(decl) => {
                        for declarator in &decl.declarations {
                            if let Some(init_expr) = &declarator.init {
                                collect_expression_refs(init_expr, refs);
                            }
                            collect_binding_pattern_names(&declarator.id, declared);
                        }
                    }
                    _ => collect_expression_refs(init.to_expression(), refs),
                }
            }
            if let Some(test) = &for_stmt.test {
                collect_expression_refs(test, refs);
            }
            if let Some(update) = &for_stmt.update {
                collect_expression_refs(update, refs);
            }
            collect_statement_refs(&for_stmt.body, refs, declared);
        }
        ast::Statement::ForOfStatement(for_of) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, declared);
                }
            }
            collect_expression_refs(&for_of.right, refs);
            collect_statement_refs(&for_of.body, refs, declared);
        }
        ast::Statement::ForInStatement(for_in) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_in.left {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, declared);
                }
            }
            collect_expression_refs(&for_in.right, refs);
            collect_statement_refs(&for_in.body, refs, declared);
        }
        ast::Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_statement_refs(s, refs, declared);
            }
        }
        ast::Statement::ThrowStatement(throw) => {
            collect_expression_refs(&throw.argument, refs);
        }
        ast::Statement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                collect_statement_refs(s, refs, declared);
            }
            if let Some(handler) = &try_stmt.handler {
                if let Some(param) = &handler.param {
                    collect_binding_pattern_names(&param.pattern, declared);
                }
                for s in &handler.body.body {
                    collect_statement_refs(s, refs, declared);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for s in &finalizer.body {
                    collect_statement_refs(s, refs, declared);
                }
            }
        }
        ast::Statement::SwitchStatement(switch) => {
            collect_expression_refs(&switch.discriminant, refs);
            for case in &switch.cases {
                if let Some(test) = &case.test {
                    collect_expression_refs(test, refs);
                }
                for s in &case.consequent {
                    collect_statement_refs(s, refs, declared);
                }
            }
        }
        ast::Statement::LabeledStatement(labeled) => {
            collect_statement_refs(&labeled.body, refs, declared);
        }
        ast::Statement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                declared.insert(id.name.to_string());
            }
            // Do not recurse into inner function bodies for captured context
            // (their own references are their own captures)
        }
        _ => {}
    }
}

/// Collect identifier references from an expression.
fn collect_expression_refs(expr: &ast::Expression<'_>, refs: &mut Vec<(String, SourceLocation)>) {
    match expr {
        ast::Expression::Identifier(ident) => {
            if ident.name != "undefined" {
                refs.push((ident.name.to_string(), span_to_loc(ident.span)));
            }
        }
        ast::Expression::BinaryExpression(bin) => {
            collect_expression_refs(&bin.left, refs);
            collect_expression_refs(&bin.right, refs);
        }
        ast::Expression::LogicalExpression(logical) => {
            collect_expression_refs(&logical.left, refs);
            collect_expression_refs(&logical.right, refs);
        }
        ast::Expression::UnaryExpression(unary) => {
            collect_expression_refs(&unary.argument, refs);
        }
        ast::Expression::ConditionalExpression(cond) => {
            collect_expression_refs(&cond.test, refs);
            collect_expression_refs(&cond.consequent, refs);
            collect_expression_refs(&cond.alternate, refs);
        }
        ast::Expression::CallExpression(call) => {
            collect_expression_refs(&call.callee, refs);
            for arg in &call.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        collect_expression_refs(&spread.argument, refs);
                    }
                    _ => collect_expression_refs(arg.to_expression(), refs),
                }
            }
        }
        ast::Expression::NewExpression(new_expr) => {
            collect_expression_refs(&new_expr.callee, refs);
            for arg in &new_expr.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        collect_expression_refs(&spread.argument, refs);
                    }
                    _ => collect_expression_refs(arg.to_expression(), refs),
                }
            }
        }
        ast::Expression::StaticMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
        }
        ast::Expression::ComputedMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
            collect_expression_refs(&member.expression, refs);
        }
        ast::Expression::AssignmentExpression(assign) => {
            collect_expression_refs(&assign.right, refs);
            collect_assignment_target_refs(&assign.left, refs);
        }
        ast::Expression::UpdateExpression(update) => {
            if let ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &update.argument
            {
                refs.push((ident.name.to_string(), span_to_loc(ident.span)));
            }
        }
        ast::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    ast::ArrayExpressionElement::SpreadElement(spread) => {
                        collect_expression_refs(&spread.argument, refs);
                    }
                    ast::ArrayExpressionElement::Elision(_) => {}
                    _ => collect_expression_refs(elem.to_expression(), refs),
                }
            }
        }
        ast::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_expression_refs(&spread.argument, refs);
                    }
                    ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        if prop.computed {
                            collect_expression_refs(prop.key.to_expression(), refs);
                        }
                        collect_expression_refs(&prop.value, refs);
                    }
                }
            }
        }
        ast::Expression::TemplateLiteral(tpl) => {
            for sub_expr in &tpl.expressions {
                collect_expression_refs(sub_expr, refs);
            }
        }
        ast::Expression::TaggedTemplateExpression(tagged) => {
            collect_expression_refs(&tagged.tag, refs);
            for sub_expr in &tagged.quasi.expressions {
                collect_expression_refs(sub_expr, refs);
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            for sub_expr in &seq.expressions {
                collect_expression_refs(sub_expr, refs);
            }
        }
        ast::Expression::AwaitExpression(await_expr) => {
            collect_expression_refs(&await_expr.argument, refs);
        }
        ast::Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                collect_expression_refs(arg, refs);
            }
        }
        ast::Expression::ChainExpression(chain) => {
            collect_chain_element_refs(&chain.expression, refs);
        }
        ast::Expression::ParenthesizedExpression(paren) => {
            collect_expression_refs(&paren.expression, refs);
        }
        ast::Expression::TSAsExpression(ts_as) => {
            collect_expression_refs(&ts_as.expression, refs);
        }
        ast::Expression::TSSatisfiesExpression(ts_sat) => {
            collect_expression_refs(&ts_sat.expression, refs);
        }
        ast::Expression::TSNonNullExpression(ts_nn) => {
            collect_expression_refs(&ts_nn.expression, refs);
        }
        ast::Expression::TSTypeAssertion(ts_ta) => {
            collect_expression_refs(&ts_ta.expression, refs);
        }
        ast::Expression::JSXElement(jsx) => {
            collect_jsx_element_refs(jsx, refs);
        }
        ast::Expression::JSXFragment(frag) => {
            for child in &frag.children {
                collect_jsx_child_refs(child, refs);
            }
        }
        // Literals, function expressions, and other expression types
        // that don't reference identifiers (or have their own scope)
        _ => {}
    }
}

/// Collect identifier references from an assignment target.
fn collect_assignment_target_refs(
    target: &ast::AssignmentTarget<'_>,
    refs: &mut Vec<(String, SourceLocation)>,
) {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            refs.push((ident.name.to_string(), span_to_loc(ident.span)));
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
            collect_expression_refs(&member.expression, refs);
        }
        _ => {}
    }
}

/// Collect identifier references from a chain element.
fn collect_chain_element_refs(
    element: &ast::ChainElement<'_>,
    refs: &mut Vec<(String, SourceLocation)>,
) {
    match element {
        ast::ChainElement::CallExpression(call) => {
            collect_expression_refs(&call.callee, refs);
            for arg in &call.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        collect_expression_refs(&spread.argument, refs);
                    }
                    _ => collect_expression_refs(arg.to_expression(), refs),
                }
            }
        }
        ast::ChainElement::StaticMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
        }
        ast::ChainElement::ComputedMemberExpression(member) => {
            collect_expression_refs(&member.object, refs);
            collect_expression_refs(&member.expression, refs);
        }
        ast::ChainElement::TSNonNullExpression(ts_nn) => {
            collect_expression_refs(&ts_nn.expression, refs);
        }
        ast::ChainElement::PrivateFieldExpression(pf) => {
            collect_expression_refs(&pf.object, refs);
        }
    }
}

/// Collect identifier references from a JSX element.
fn collect_jsx_element_refs(
    element: &ast::JSXElement<'_>,
    refs: &mut Vec<(String, SourceLocation)>,
) {
    // Tag name
    if let ast::JSXElementName::IdentifierReference(ident) = &element.opening_element.name
        && ident.name.starts_with(|c: char| c.is_ascii_uppercase())
    {
        refs.push((ident.name.to_string(), span_to_loc(ident.span)));
    }
    // Attributes
    for attr in &element.opening_element.attributes {
        match attr {
            ast::JSXAttributeItem::SpreadAttribute(spread) => {
                collect_expression_refs(&spread.argument, refs);
            }
            ast::JSXAttributeItem::Attribute(attr) => {
                if let Some(value) = &attr.value {
                    match value {
                        ast::JSXAttributeValue::ExpressionContainer(container) => {
                            if !matches!(
                                &container.expression,
                                ast::JSXExpression::EmptyExpression(_)
                            ) {
                                collect_expression_refs(container.expression.to_expression(), refs);
                            }
                        }
                        ast::JSXAttributeValue::Element(elem) => {
                            collect_jsx_element_refs(elem, refs);
                        }
                        ast::JSXAttributeValue::Fragment(frag) => {
                            for child in &frag.children {
                                collect_jsx_child_refs(child, refs);
                            }
                        }
                        ast::JSXAttributeValue::StringLiteral(_) => {}
                    }
                }
            }
        }
    }
    // Children
    for child in &element.children {
        collect_jsx_child_refs(child, refs);
    }
}

/// Collect identifier references from a JSX child.
fn collect_jsx_child_refs(child: &ast::JSXChild<'_>, refs: &mut Vec<(String, SourceLocation)>) {
    match child {
        ast::JSXChild::ExpressionContainer(container) => {
            if !matches!(&container.expression, ast::JSXExpression::EmptyExpression(_)) {
                collect_expression_refs(container.expression.to_expression(), refs);
            }
        }
        ast::JSXChild::Element(element) => {
            collect_jsx_element_refs(element, refs);
        }
        ast::JSXChild::Fragment(frag) => {
            for c in &frag.children {
                collect_jsx_child_refs(c, refs);
            }
        }
        ast::JSXChild::Spread(spread) => {
            collect_expression_refs(&spread.expression, refs);
        }
        ast::JSXChild::Text(_) => {}
    }
}

/// Lower an inner function to a `FunctionExpressionValue`.
///
/// Port of `lowerFunctionToValue()` from `HIR/BuildHIR.ts` (lines 3462-3520).
///
/// This function:
/// 1. Gathers the captured context (free variables of the inner function)
/// 2. Creates a new HIR builder for the inner function
/// 3. Recursively lowers the inner function body
/// 4. Returns the `FunctionExpressionValue`
///
/// # Errors
/// Returns a `CompilerError` if recursive lowering fails.
fn lower_function_to_value(
    outer_builder: &mut HirBuilder,
    func: &LowerableFunction<'_>,
    expression_type: FunctionExpressionType,
    loc: SourceLocation,
) -> Result<InstructionValue, CompilerError> {
    // 1. Gather captured context from the inner function
    let captured_context = gather_captured_context(func, outer_builder);

    // 2. Recursively lower the inner function with ReactFunctionType::Other
    let env = outer_builder.environment();

    // Find context identifiers for the inner function
    let context_identifiers = match func {
        LowerableFunction::Function(f) => find_context_identifiers(f),
        LowerableFunction::ArrowFunction(a) => find_context_identifiers_arrow(a),
    };

    // Create a new builder with the captured context merged into context_identifiers
    let mut merged_context = context_identifiers;
    for name in captured_context.keys() {
        merged_context.insert(name.clone());
    }

    let mut inner_builder = HirBuilder::new(env.clone(), None, merged_context);

    // Resolve captured context variables in the inner builder to build the context Vec<Place>
    let mut context_places = Vec::new();
    for (name, ctx_loc) in &captured_context {
        let place = inner_builder.declare_binding(name, BindingKind::Let, *ctx_loc);
        context_places.push(place);
    }

    // Extract function metadata
    let (id, generator, is_async) = extract_function_metadata(func);
    let func_loc = extract_function_loc(func);

    // Walk parameters
    let params = lower_params(&mut inner_builder, func)?;

    // Walk body statements and extract directives
    let directives = lower_body(&mut inner_builder, func)?;

    // Emit final void return
    let void_value = create_temporary_place(inner_builder.environment_mut(), GENERATED_SOURCE);
    inner_builder.push(Instruction {
        id: InstructionId(0),
        lvalue: void_value.clone(),
        value: lower_undefined(GENERATED_SOURCE),
        effects: None,
        loc: GENERATED_SOURCE,
    });
    inner_builder.terminate(
        Terminal::Return(crate::hir::ReturnTerminal {
            id: InstructionId(0),
            value: void_value,
            return_variant: ReturnVariant::Void,
            loc: GENERATED_SOURCE,
        }),
        None,
    );

    let (body, mut built_env) = inner_builder.build_with_env()?;
    let returns = create_temporary_place(&mut built_env, func_loc);

    // Advance the outer builder's environment counters past the inner function's
    // allocations. This simulates the TS behavior where the inner function shares
    // the same Environment object by reference, so the outer function's counters
    // automatically stay ahead of all inner function IDs.
    outer_builder.environment_mut().advance_counters_past(&built_env);

    let hir_function = HIRFunction {
        loc: func_loc,
        id: id.clone(),
        name_hint: None,
        fn_type: ReactFunctionType::Other,
        env: built_env,
        params,
        returns,
        context: context_places,
        body,
        generator,
        is_async,
        directives,
        aliasing_effects: None,
    };

    Ok(InstructionValue::FunctionExpression(FunctionExpressionValue {
        name: id,
        name_hint: None,
        lowered_func: LoweredFunction { func: Box::new(hir_function) },
        expression_type,
        loc,
    }))
}

// =====================================================================================
// Statement lowering helpers
// =====================================================================================

/// Lower a block statement into the HIR builder.
///
/// # Errors
/// Returns a `CompilerError` if any statement in the block cannot be lowered.
pub fn lower_block_statement(
    builder: &mut HirBuilder,
    stmts: &[LowerableStatement<'_>],
) -> Result<(), CompilerError> {
    for stmt in stmts {
        lower_statement(builder, stmt)?;
    }
    Ok(())
}

/// A statement that can be lowered to HIR.
/// This is an abstraction over oxc_ast statement types.
#[derive(Debug)]
pub enum LowerableStatement<'a> {
    VariableDeclaration(&'a ast::VariableDeclaration<'a>),
    ExpressionStatement(&'a ast::ExpressionStatement<'a>),
    ReturnStatement(&'a ast::ReturnStatement<'a>),
    IfStatement(&'a ast::IfStatement<'a>),
    WhileStatement(&'a ast::WhileStatement<'a>),
    ForStatement(&'a ast::ForStatement<'a>),
    ForOfStatement(&'a ast::ForOfStatement<'a>),
    ForInStatement(&'a ast::ForInStatement<'a>),
    DoWhileStatement(&'a ast::DoWhileStatement<'a>),
    BlockStatement(&'a ast::BlockStatement<'a>),
    ThrowStatement(&'a ast::ThrowStatement<'a>),
    TryStatement(&'a ast::TryStatement<'a>),
    SwitchStatement(&'a ast::SwitchStatement<'a>),
    LabeledStatement(&'a ast::LabeledStatement<'a>),
    FunctionDeclaration(&'a ast::Function<'a>),
    BreakStatement,
    ContinueStatement,
    DebuggerStatement,
    EmptyStatement,
}

/// Lower a statement, with an optional label for loop statements.
///
/// # Errors
/// Returns a `CompilerError` if the statement cannot be lowered.
fn lower_statement_with_label(
    builder: &mut HirBuilder,
    stmt: &LowerableStatement<'_>,
    label: Option<&str>,
) -> Result<(), CompilerError> {
    match stmt {
        // =====================================================================
        // ThrowStatement
        // =====================================================================
        LowerableStatement::ThrowStatement(throw) => {
            let loc = span_to_loc(throw.span);
            let lowerable = convert_expression(&throw.argument);
            let value = lower_expression(builder, &lowerable)?.place;
            builder.terminate(
                Terminal::Throw(crate::hir::ThrowTerminal { id: InstructionId(0), value, loc }),
                Some(BlockKind::Block),
            );
        }

        // =====================================================================
        // ReturnStatement
        // =====================================================================
        LowerableStatement::ReturnStatement(ret) => {
            let loc = span_to_loc(ret.span);
            let value = if let Some(argument) = &ret.argument {
                let lowerable = convert_expression(argument);
                lower_expression(builder, &lowerable)?.place
            } else {
                let lvalue = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: lvalue.clone(),
                    value: lower_undefined(GENERATED_SOURCE),
                    effects: None,
                    loc: GENERATED_SOURCE,
                });
                lvalue
            };
            builder.terminate(
                Terminal::Return(crate::hir::ReturnTerminal {
                    id: InstructionId(0),
                    value,
                    return_variant: ReturnVariant::Explicit,
                    loc,
                }),
                Some(BlockKind::Block),
            );
        }

        // =====================================================================
        // IfStatement
        // =====================================================================
        LowerableStatement::IfStatement(if_stmt) => {
            let loc = span_to_loc(if_stmt.span);

            // Block for code following the if
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Block for the consequent (if the test is truthy)
            let consequent_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                let consequent = convert_statement(&if_stmt.consequent);
                lower_statement_with_label(builder, &consequent, None).ok(); // errors accumulated in builder
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc: span_to_loc(if_stmt.consequent.span()),
                })
            });

            // Block for the alternate (if the test is not truthy)
            let alternate_block = if let Some(alternate) = &if_stmt.alternate {
                builder.enter(BlockKind::Block, |builder, _block_id| {
                    let alt = convert_statement(alternate);
                    lower_statement_with_label(builder, &alt, None).ok();
                    Terminal::Goto(GotoTerminal {
                        id: InstructionId(0),
                        block: continuation_id,
                        variant: GotoVariant::Break,
                        loc: span_to_loc(alternate.span()),
                    })
                })
            } else {
                // If there is no else clause, use the continuation directly
                continuation_id
            };

            let test_lowerable = convert_expression(&if_stmt.test);
            let test = lower_expression(builder, &test_lowerable)?.place;

            builder.terminate_with_continuation(
                Terminal::If(IfTerminal {
                    id: InstructionId(0),
                    test,
                    consequent: consequent_block,
                    alternate: alternate_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // BlockStatement
        // =====================================================================
        LowerableStatement::BlockStatement(block) => {
            let stmts: Vec<_> = block.body.iter().map(convert_statement).collect();
            lower_block_statement(builder, &stmts)?;
        }

        // =====================================================================
        // BreakStatement
        // =====================================================================
        LowerableStatement::BreakStatement => {
            let target = builder.lookup_break(None)?;
            builder.terminate(
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: target,
                    variant: GotoVariant::Break,
                    loc: GENERATED_SOURCE,
                }),
                Some(BlockKind::Block),
            );
        }

        // =====================================================================
        // ContinueStatement
        // =====================================================================
        LowerableStatement::ContinueStatement => {
            let target = builder.lookup_continue(None)?;
            builder.terminate(
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: target,
                    variant: GotoVariant::Continue,
                    loc: GENERATED_SOURCE,
                }),
                Some(BlockKind::Block),
            );
        }

        // =====================================================================
        // ForStatement
        // =====================================================================
        LowerableStatement::ForStatement(for_stmt) => {
            let loc = span_to_loc(for_stmt.span);

            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Init block
            let init_block = builder.enter(BlockKind::Loop, |builder, _block_id| {
                if let Some(init) = &for_stmt.init {
                    match init {
                        ast::ForStatementInit::VariableDeclaration(decl) => {
                            let init_stmt = LowerableStatement::VariableDeclaration(decl);
                            lower_statement_with_label(builder, &init_stmt, None).ok();
                        }
                        _ => {
                            // Non-variable init (expression): lower as expression placeholder
                            let lvalue =
                                create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
                            builder.push(Instruction {
                                id: InstructionId(0),
                                lvalue,
                                value: lower_undefined(GENERATED_SOURCE),
                                effects: None,
                                loc: GENERATED_SOURCE,
                            });
                        }
                    }
                }
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // Update block (optional)
            let update_block = if let Some(update) = &for_stmt.update {
                let update_expr = convert_expression(update);
                Some(builder.enter(BlockKind::Loop, |builder, _block_id| {
                    lower_expression(builder, &update_expr).ok();
                    Terminal::Goto(GotoTerminal {
                        id: InstructionId(0),
                        block: test_block_id,
                        variant: GotoVariant::Break,
                        loc: span_to_loc(update.span()),
                    })
                }))
            } else {
                None
            };

            let continue_target = update_block.unwrap_or(test_block_id);

            // Body block
            let body_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    continue_target,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&for_stmt.body);
                        lower_statement_with_label(builder, &body, None).ok();
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: continue_target,
                                variant: GotoVariant::Continue,
                                loc: span_to_loc(for_stmt.body.span()),
                            }),
                            None,
                        );
                    },
                );
                // The loop body terminate already handled, emit a dummy terminal
                // that won't be reached
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continue_target,
                    variant: GotoVariant::Continue,
                    loc,
                })
            });

            builder.terminate_with_continuation(
                Terminal::For(ForTerminal {
                    id: InstructionId(0),
                    init: init_block,
                    test: test_block_id,
                    update: update_block,
                    r#loop: body_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                test_block,
            );

            // Lower the test expression in the test block
            if let Some(test) = &for_stmt.test {
                let test_expr = convert_expression(test);
                let test_place = lower_expression(builder, &test_expr)?.place;
                builder.terminate_with_continuation(
                    Terminal::Branch(BranchTerminal {
                        id: InstructionId(0),
                        test: test_place,
                        consequent: body_block,
                        alternate: continuation_id,
                        fallthrough: continuation_id,
                        loc,
                    }),
                    continuation_block,
                );
            } else {
                // No test: unconditional loop (infinite loop)
                builder.terminate_with_continuation(
                    Terminal::Goto(GotoTerminal {
                        id: InstructionId(0),
                        block: body_block,
                        variant: GotoVariant::Break,
                        loc,
                    }),
                    continuation_block,
                );
            }
        }

        // =====================================================================
        // WhileStatement
        // =====================================================================
        LowerableStatement::WhileStatement(while_stmt) => {
            let loc = span_to_loc(while_stmt.span);

            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    conditional_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&while_stmt.body);
                        lower_statement_with_label(builder, &body, None).ok();
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: conditional_id,
                                variant: GotoVariant::Continue,
                                loc: span_to_loc(while_stmt.body.span()),
                            }),
                            None,
                        );
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: conditional_id,
                    variant: GotoVariant::Continue,
                    loc,
                })
            });

            // Terminate current block with WhileTerminal, continue into conditional block
            builder.terminate_with_continuation(
                Terminal::While(WhileTerminal {
                    id: InstructionId(0),
                    test: conditional_id,
                    r#loop: loop_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                conditional_block,
            );

            // Lower the test expression in the conditional block
            let test_expr = convert_expression(&while_stmt.test);
            let test = lower_expression(builder, &test_expr)?.place;
            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: conditional_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // DoWhileStatement
        // =====================================================================
        LowerableStatement::DoWhileStatement(do_while) => {
            let loc = span_to_loc(do_while.span);

            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body, executed at least once unconditionally prior to exit
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    conditional_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&do_while.body);
                        lower_statement_with_label(builder, &body, None).ok();
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: conditional_id,
                                variant: GotoVariant::Continue,
                                loc: span_to_loc(do_while.body.span()),
                            }),
                            None,
                        );
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: conditional_id,
                    variant: GotoVariant::Continue,
                    loc,
                })
            });

            // Jump to the conditional block
            builder.terminate_with_continuation(
                Terminal::DoWhile(DoWhileTerminal {
                    id: InstructionId(0),
                    r#loop: loop_block,
                    test: conditional_id,
                    fallthrough: continuation_id,
                    loc,
                }),
                conditional_block,
            );

            // Lower the test expression in the conditional block
            let test_expr = convert_expression(&do_while.test);
            let test = lower_expression(builder, &test_expr)?.place;
            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: conditional_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // ForOfStatement
        // =====================================================================
        LowerableStatement::ForOfStatement(for_of) => {
            let loc = span_to_loc(for_of.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;
            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;

            // Loop body
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&for_of.body);
                        lower_statement_with_label(builder, &body, None).ok();
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: init_block_id,
                                variant: GotoVariant::Continue,
                                loc: span_to_loc(for_of.body.span()),
                            }),
                            None,
                        );
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: init_block_id,
                    variant: GotoVariant::Continue,
                    loc,
                })
            });

            // Lower the right-hand side expression (collection)
            let right_expr = convert_expression(&for_of.right);
            let value = lower_expression(builder, &right_expr)?.place;

            builder.terminate_with_continuation(
                Terminal::ForOf(ForOfTerminal {
                    id: InstructionId(0),
                    init: init_block_id,
                    test: test_block_id,
                    r#loop: loop_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                init_block,
            );

            // Init block: GetIterator
            let iterator = create_temporary_place(builder.environment_mut(), value.loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: iterator.clone(),
                value: InstructionValue::GetIterator(GetIterator {
                    collection: value.clone(),
                    loc: value.loc,
                }),
                effects: None,
                loc: value.loc,
            });
            builder.terminate_with_continuation(
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    loc,
                }),
                test_block,
            );

            // Test block: IteratorNext + assignment
            let left_loc = span_to_loc(for_of.left.span());
            let advance_iterator = create_temporary_place(builder.environment_mut(), left_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: advance_iterator.clone(),
                value: InstructionValue::IteratorNext(IteratorNext {
                    iterator: iterator.clone(),
                    collection: value.clone(),
                    loc: left_loc,
                }),
                effects: None,
                loc: left_loc,
            });

            // Assign the iterator value to the loop variable via StoreLocal
            let test_place =
                lower_for_loop_left_assignment(builder, &for_of.left, &advance_iterator, left_loc);

            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test: test_place,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // ForInStatement
        // =====================================================================
        LowerableStatement::ForInStatement(for_in) => {
            let loc = span_to_loc(for_in.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;

            // Loop body
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&for_in.body);
                        lower_statement_with_label(builder, &body, None).ok();
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: init_block_id,
                                variant: GotoVariant::Continue,
                                loc: span_to_loc(for_in.body.span()),
                            }),
                            None,
                        );
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: init_block_id,
                    variant: GotoVariant::Continue,
                    loc,
                })
            });

            // Lower the right-hand side expression
            let right_expr = convert_expression(&for_in.right);
            let value = lower_expression(builder, &right_expr)?.place;

            builder.terminate_with_continuation(
                Terminal::ForIn(ForInTerminal {
                    id: InstructionId(0),
                    init: init_block_id,
                    r#loop: loop_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                init_block,
            );

            // Init block: NextPropertyOf + assignment
            let left_loc = span_to_loc(for_in.left.span());
            let next_property = create_temporary_place(builder.environment_mut(), left_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: next_property.clone(),
                value: InstructionValue::NextPropertyOf(NextPropertyOf {
                    value: value.clone(),
                    loc: left_loc,
                }),
                effects: None,
                loc: left_loc,
            });

            // Assign to the loop variable
            let test_place =
                lower_for_loop_left_assignment(builder, &for_in.left, &next_property, left_loc);

            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test: test_place,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // LabeledStatement
        // =====================================================================
        LowerableStatement::LabeledStatement(labeled) => {
            let loc = span_to_loc(labeled.span);
            let label_name = labeled.label.name.as_str();

            // For loop statements, push the label down
            match &labeled.body {
                ast::Statement::ForStatement(_)
                | ast::Statement::ForOfStatement(_)
                | ast::Statement::ForInStatement(_)
                | ast::Statement::WhileStatement(_)
                | ast::Statement::DoWhileStatement(_) => {
                    let body = convert_statement(&labeled.body);
                    lower_statement_with_label(builder, &body, Some(label_name))?;
                }
                _ => {
                    // All other statements create a continuation block to allow `break`
                    let continuation_block = builder.reserve(BlockKind::Block);
                    let continuation_id = continuation_block.id;

                    let block = builder.enter(BlockKind::Block, |builder, _block_id| {
                        builder.label(label_name.to_string(), continuation_id, |builder| {
                            let body = convert_statement(&labeled.body);
                            lower_statement_with_label(builder, &body, None).ok();
                        });
                        Terminal::Goto(GotoTerminal {
                            id: InstructionId(0),
                            block: continuation_id,
                            variant: GotoVariant::Break,
                            loc: span_to_loc(labeled.body.span()),
                        })
                    });

                    builder.terminate_with_continuation(
                        Terminal::Label(LabelTerminal {
                            id: InstructionId(0),
                            block,
                            fallthrough: continuation_id,
                            loc,
                        }),
                        continuation_block,
                    );
                }
            }
        }

        // =====================================================================
        // SwitchStatement
        // =====================================================================
        LowerableStatement::SwitchStatement(switch_stmt) => {
            let loc = span_to_loc(switch_stmt.span);

            // Block following the switch
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // The goto target for any cases that fallthrough
            let mut fallthrough = continuation_id;

            // Iterate through cases in reverse order
            let mut cases: Vec<Case> = Vec::new();
            let mut has_default = false;

            for ii in (0..switch_stmt.cases.len()).rev() {
                let case = &switch_stmt.cases[ii];

                if case.test.is_none() {
                    has_default = true;
                }

                let case_fallthrough = fallthrough;
                let block = builder.enter(BlockKind::Block, |builder, _block_id| {
                    builder.switch(label.map(String::from), continuation_id, |builder| {
                        for consequent in &case.consequent {
                            let stmt = convert_statement(consequent);
                            lower_statement_with_label(builder, &stmt, None).ok();
                        }
                        builder.terminate(
                            Terminal::Goto(GotoTerminal {
                                id: InstructionId(0),
                                block: case_fallthrough,
                                variant: GotoVariant::Break,
                                loc: span_to_loc(case.span),
                            }),
                            None,
                        );
                    });
                    Terminal::Goto(GotoTerminal {
                        id: InstructionId(0),
                        block: case_fallthrough,
                        variant: GotoVariant::Break,
                        loc: span_to_loc(case.span),
                    })
                });

                let test = if let Some(test_expr) = &case.test {
                    let lowerable = convert_expression(test_expr);
                    Some(lower_expression(builder, &lowerable)?.place)
                } else {
                    None
                };

                cases.push(Case { test, block });
                fallthrough = block;
            }

            // Reverse back to original order
            cases.reverse();

            // If there wasn't an explicit default case, generate one
            if !has_default {
                cases.push(Case { test: None, block: continuation_id });
            }

            // Lower the discriminant
            let discriminant_expr = convert_expression(&switch_stmt.discriminant);
            let test = lower_expression(builder, &discriminant_expr)?.place;

            builder.terminate_with_continuation(
                Terminal::Switch(SwitchTerminal {
                    id: InstructionId(0),
                    test,
                    cases,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // VariableDeclaration
        // =====================================================================
        LowerableStatement::VariableDeclaration(var_decl) => {
            let loc = span_to_loc(var_decl.span);
            let kind = match var_decl.kind {
                ast::VariableDeclarationKind::Let => InstructionKind::Let,
                ast::VariableDeclarationKind::Const => InstructionKind::Const,
                ast::VariableDeclarationKind::Var => InstructionKind::Let, // treat var as let
                ast::VariableDeclarationKind::Using | ast::VariableDeclarationKind::AwaitUsing => {
                    InstructionKind::Const
                }
            };
            let binding_kind = match var_decl.kind {
                ast::VariableDeclarationKind::Const
                | ast::VariableDeclarationKind::Using
                | ast::VariableDeclarationKind::AwaitUsing => BindingKind::Const,
                ast::VariableDeclarationKind::Let => BindingKind::Let,
                ast::VariableDeclarationKind::Var => BindingKind::Var,
            };

            for declaration in &var_decl.declarations {
                if let Some(init) = &declaration.init {
                    // Lower the initializer
                    let init_expr = convert_expression(init);
                    let value = lower_expression(builder, &init_expr)?.place;

                    match &declaration.id {
                        // Destructuring pattern: emit Destructure instruction
                        ast::BindingPattern::ObjectPattern(_)
                        | ast::BindingPattern::ArrayPattern(_) => {
                            lower_destructuring_declaration(
                                builder,
                                &declaration.id,
                                value,
                                kind,
                                binding_kind,
                                loc,
                            )?;
                        }
                        // Simple identifier binding
                        ast::BindingPattern::BindingIdentifier(ident) => {
                            let decl_loc = span_to_loc(declaration.span);
                            let decl_place =
                                builder.declare_binding(&ident.name, binding_kind, decl_loc);

                            if builder.is_context_identifier(&ident.name) {
                                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                                builder.push(Instruction {
                                    id: InstructionId(0),
                                    lvalue,
                                    value: InstructionValue::StoreContext(StoreContext {
                                        lvalue_kind: InstructionKind::Let,
                                        lvalue_place: decl_place,
                                        value,
                                        loc,
                                    }),
                                    effects: None,
                                    loc,
                                });
                            } else {
                                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                                builder.push(Instruction {
                                    id: InstructionId(0),
                                    lvalue,
                                    value: InstructionValue::StoreLocal(StoreLocal {
                                        lvalue: LValue { place: decl_place, kind },
                                        value,
                                        loc,
                                    }),
                                    effects: None,
                                    loc,
                                });
                            }
                        }
                        // AssignmentPattern at top level (e.g., `let x = 1 = ...` is invalid,
                        // but handle gracefully)
                        ast::BindingPattern::AssignmentPattern(assign) => {
                            lower_destructuring_declaration(
                                builder,
                                &assign.left,
                                value,
                                kind,
                                binding_kind,
                                loc,
                            )?;
                        }
                    }
                } else {
                    // No initializer: emit DeclareLocal or DeclareContext
                    // For destructuring without init, declare all leaf bindings
                    let decl_loc = span_to_loc(declaration.span);
                    declare_all_bindings_in_pattern(
                        builder,
                        &declaration.id,
                        binding_kind,
                        kind,
                        decl_loc,
                        loc,
                    );
                }
            }
        }

        // =====================================================================
        // ExpressionStatement
        // =====================================================================
        LowerableStatement::ExpressionStatement(expr_stmt) => {
            let lowerable = convert_expression(&expr_stmt.expression);
            lower_expression(builder, &lowerable)?;
        }

        // =====================================================================
        // FunctionDeclaration
        // Port of BuildHIR.ts — calls lowerFunctionToValue then StoreLocal
        // =====================================================================
        LowerableStatement::FunctionDeclaration(func) => {
            let loc = span_to_loc(func.span);

            // Lower the function body recursively
            let lowerable_func = LowerableFunction::Function(func);
            let fn_value = lower_function_to_value(
                builder,
                &lowerable_func,
                FunctionExpressionType::FunctionDeclaration,
                loc,
            )?;

            // Emit the FunctionExpression instruction
            let fn_place = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: fn_place.clone(),
                value: fn_value,
                effects: None,
                loc,
            });

            // Register the function name as a binding
            let decl_place = if let Some(id) = &func.id {
                builder.declare_binding(&id.name, BindingKind::Function, loc)
            } else {
                create_temporary_place(builder.environment_mut(), loc)
            };
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: decl_place, kind: InstructionKind::Function },
                    value: fn_place,
                    loc,
                }),
                effects: None,
                loc,
            });
        }

        // =====================================================================
        // TryStatement
        // =====================================================================
        LowerableStatement::TryStatement(try_stmt) => {
            let loc = span_to_loc(try_stmt.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            if try_stmt.handler.is_none() {
                builder.errors.push_error_detail(CompilerErrorDetail::new(
                    CompilerErrorDetailOptions {
                        category: ErrorCategory::Todo,
                        reason:
                            "(BuildHIR::lowerStatement) Handle TryStatement without a catch clause"
                                .to_string(),
                        description: None,
                        loc: Some(loc),
                        suggestions: None,
                    },
                ));
                return Ok(());
            }

            // Declare handler binding if present
            let handler_binding = if let Some(handler) = &try_stmt.handler {
                if let Some(param) = &handler.param {
                    let handler_loc = span_to_loc(param.span());
                    let place = create_temporary_place(builder.environment_mut(), handler_loc);

                    // Emit DeclareLocal for catch binding
                    let lvalue = create_temporary_place(builder.environment_mut(), handler_loc);
                    builder.push(Instruction {
                        id: InstructionId(0),
                        lvalue,
                        value: InstructionValue::DeclareLocal(DeclareLocal {
                            lvalue: LValue { place: place.clone(), kind: InstructionKind::Catch },
                            loc: handler_loc,
                        }),
                        effects: None,
                        loc: handler_loc,
                    });
                    Some(place)
                } else {
                    None
                }
            } else {
                None
            };

            // Handler block (catch block)
            let handler = builder.enter(BlockKind::Catch, |builder, _block_id| {
                if let Some(catch_clause) = &try_stmt.handler {
                    // Lower catch body
                    let stmts: Vec<_> =
                        catch_clause.body.body.iter().map(convert_statement).collect();
                    lower_block_statement(builder, &stmts).ok();
                }
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // Try block
            let block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_try_catch(handler, |builder| {
                    let stmts: Vec<_> = try_stmt.block.body.iter().map(convert_statement).collect();
                    lower_block_statement(builder, &stmts).ok();
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Try,
                    loc,
                })
            });

            builder.terminate_with_continuation(
                Terminal::Try(TryTerminal {
                    id: InstructionId(0),
                    block,
                    handler_binding,
                    handler,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );
        }

        // =====================================================================
        // DebuggerStatement
        // =====================================================================
        LowerableStatement::DebuggerStatement => {
            let lvalue = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::Debugger(crate::hir::DebuggerValue {
                    loc: GENERATED_SOURCE,
                }),
                effects: None,
                loc: GENERATED_SOURCE,
            });
        }

        // =====================================================================
        // EmptyStatement
        // =====================================================================
        LowerableStatement::EmptyStatement => {}
    }
    Ok(())
}

/// Lower a statement to HIR.
///
/// # Errors
/// Returns a `CompilerError` if the statement cannot be lowered.
pub fn lower_statement(
    builder: &mut HirBuilder,
    stmt: &LowerableStatement<'_>,
) -> Result<(), CompilerError> {
    lower_statement_with_label(builder, stmt, None)
}

/// Helper to lower the left-hand side of a for-of/for-in loop.
/// Determines the correct `InstructionKind` from the `ForStatementLeft` and
/// emits a `StoreLocal` instruction. Returns the place used as the branch test.
fn lower_for_loop_left_assignment(
    builder: &mut HirBuilder,
    left: &ast::ForStatementLeft<'_>,
    rhs_value: &crate::hir::Place,
    loc: SourceLocation,
) -> crate::hir::Place {
    let kind = match left {
        ast::ForStatementLeft::VariableDeclaration(decl) => match decl.kind {
            ast::VariableDeclarationKind::Let => InstructionKind::Let,
            ast::VariableDeclarationKind::Const => InstructionKind::Const,
            _ => InstructionKind::Let,
        },
        // Assignment target (e.g. `for (x of ...)`) uses Reassign
        _ => InstructionKind::Reassign,
    };
    let decl_place = create_temporary_place(builder.environment_mut(), loc);
    let lvalue = create_temporary_place(builder.environment_mut(), loc);
    builder.push(Instruction {
        id: InstructionId(0),
        lvalue: lvalue.clone(),
        value: InstructionValue::StoreLocal(StoreLocal {
            lvalue: LValue { place: decl_place, kind },
            value: rhs_value.clone(),
            loc,
        }),
        effects: None,
        loc,
    });
    lvalue
}

// =====================================================================================
// Expression lowering helpers
// =====================================================================================

/// A lowered expression result — the Place where the expression value is stored.
pub struct ExpressionResult {
    pub place: crate::hir::Place,
}

/// Lower an expression, emitting instructions to the builder and returning
/// the Place that holds the result.
///
/// Port of `lowerExpression()` from `HIR/BuildHIR.ts` (lines 1545-2675).
///
/// # Errors
/// Returns a `CompilerError` if the expression cannot be lowered.
pub fn lower_expression(
    builder: &mut HirBuilder,
    expr: &LowerableExpression<'_>,
) -> Result<ExpressionResult, CompilerError> {
    match expr {
        LowerableExpression::NumericLiteral(value, span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_number(*value, loc), loc)
        }
        LowerableExpression::StringLiteral(value, span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_string(value.clone(), loc), loc)
        }
        LowerableExpression::BooleanLiteral(value, span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_boolean(*value, loc), loc)
        }
        LowerableExpression::NullLiteral(span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_null(loc), loc)
        }
        LowerableExpression::Undefined(span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_undefined(loc), loc)
        }
        LowerableExpression::RegExpLiteral { pattern, flags, span } => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(
                builder,
                InstructionValue::RegExpLiteral(crate::hir::RegExpLiteral {
                    pattern: pattern.clone(),
                    flags: flags.clone(),
                    loc,
                }),
                loc,
            )
        }
        LowerableExpression::TemplateLiteral { quasis, expressions, span } => {
            let loc = span_to_loc(*span);
            let mut subexprs = Vec::new();
            for sub_expr in expressions {
                let result = lower_expression(builder, sub_expr)?;
                subexprs.push(result.place);
            }
            let quasi_values: Vec<TemplateLiteralQuasi> = quasis
                .iter()
                .map(|(raw, cooked)| TemplateLiteralQuasi {
                    raw: raw.clone(),
                    cooked: cooked.clone(),
                })
                .collect();
            lower_value_to_temporary(
                builder,
                InstructionValue::TemplateLiteral(crate::hir::TemplateLiteral {
                    subexprs,
                    quasis: quasi_values,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // ArrayExpression — lower each element (expression, spread, or hole)
        // =====================================================================
        LowerableExpression::ArrayExpression { elements, span } => {
            let loc = span_to_loc(*span);
            let mut lowered_elements = Vec::new();
            for element in elements {
                match element {
                    LowerableArrayElement::Expression(expr) => {
                        let result = lower_expression(builder, expr)?;
                        lowered_elements.push(ArrayExpressionElement::Place(result.place));
                    }
                    LowerableArrayElement::Spread(expr, _spread_span) => {
                        let result = lower_expression(builder, expr)?;
                        lowered_elements.push(ArrayExpressionElement::Spread(SpreadPattern {
                            place: result.place,
                        }));
                    }
                    LowerableArrayElement::Hole => {
                        lowered_elements.push(ArrayExpressionElement::Hole);
                    }
                }
            }
            lower_value_to_temporary(
                builder,
                InstructionValue::ArrayExpression(crate::hir::ArrayExpression {
                    elements: lowered_elements,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // ObjectExpression — lower each property (key/value, spread, method)
        // =====================================================================
        LowerableExpression::ObjectExpression { properties, span } => {
            let loc = span_to_loc(*span);
            let mut lowered_props = Vec::new();
            for prop in properties {
                match prop {
                    LowerableObjectProperty::Spread(expr, _spread_span) => {
                        let result = lower_expression(builder, expr)?;
                        lowered_props.push(ObjectPatternProperty::Spread(SpreadPattern {
                            place: result.place,
                        }));
                    }
                    LowerableObjectProperty::Property { key, value, method, .. } => {
                        let lowered_key = lower_object_property_key(builder, key)?;
                        let value_result = lower_expression(builder, value)?;
                        let property_type = if *method {
                            ObjectPropertyType::Method
                        } else {
                            ObjectPropertyType::Property
                        };
                        lowered_props.push(ObjectPatternProperty::Property(ObjectProperty {
                            key: lowered_key,
                            property_type,
                            place: value_result.place,
                        }));
                    }
                }
            }
            lower_value_to_temporary(
                builder,
                InstructionValue::ObjectExpression(crate::hir::ObjectExpression {
                    properties: lowered_props,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // BinaryExpression
        // =====================================================================
        LowerableExpression::BinaryExpression { operator, left, right, span } => {
            let loc = span_to_loc(*span);
            let left_result = lower_expression(builder, left)?;
            let right_result = lower_expression(builder, right)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                    operator: *operator,
                    left: left_result.place,
                    right: right_result.place,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // UnaryExpression
        // =====================================================================
        LowerableExpression::UnaryExpression { operator, argument, span } => {
            let loc = span_to_loc(*span);
            let arg_result = lower_expression(builder, argument)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::UnaryExpression(crate::hir::UnaryExpressionValue {
                    operator: *operator,
                    value: arg_result.place,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // LogicalExpression — creates CFG with LogicalTerminal + BranchTerminal
        // Port of BuildHIR.ts lines 1903-1981
        // =====================================================================
        LowerableExpression::LogicalExpression { operator, left, right, span } => {
            let loc = span_to_loc(*span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = create_temporary_place(builder.environment_mut(), loc);
            let left_place = create_temporary_place(builder.environment_mut(), loc);

            // Consequent block: stores the left value (short-circuit)
            let consequent = builder.enter(BlockKind::Value, |builder, _block_id| {
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: left_place.clone(),
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // Alternate block: evaluates the right expression
            let alternate = builder.enter(BlockKind::Value, |builder, _block_id| {
                let right_result =
                    lower_expression(builder, right).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: right_result.place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // LogicalTerminal -> test block
            builder.terminate_with_continuation(
                Terminal::Logical(LogicalTerminal {
                    id: InstructionId(0),
                    operator: *operator,
                    test: test_block_id,
                    fallthrough: continuation_id,
                    loc,
                }),
                test_block,
            );

            // In the test block: lower the left expression, then LoadLocal into left_place
            let left_value = lower_expression(builder, left)?;
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: left_place.clone(),
                value: InstructionValue::LoadLocal(LoadLocal { place: left_value.place, loc }),
                effects: None,
                loc,
            });

            // BranchTerminal -> continuation block
            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test: left_place,
                    consequent,
                    alternate,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );

            // Return LoadLocal of the shared place
            lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            )
        }

        // =====================================================================
        // UpdateExpression — PrefixUpdate or PostfixUpdate
        // Port of BuildHIR.ts lines 2510-2627
        //
        // For member expression arguments (e.g. `obj.prop++`), we lower as:
        //   PropertyLoad + BinaryExpression + PropertyStore (TS lines 2513-2562)
        // For context identifiers and globals, we return a Todo error (TS lines 2572-2608)
        // For local identifiers, we emit PrefixUpdate/PostfixUpdate.
        // =====================================================================
        LowerableExpression::UpdateExpression { operator, argument, prefix, span } => {
            let loc = span_to_loc(*span);
            use oxc_syntax::operator::{BinaryOperator, UpdateOperator};

            // Determine the binary operator for the update
            let binary_op = match operator {
                UpdateOperator::Increment => BinaryOperator::Addition,
                UpdateOperator::Decrement => BinaryOperator::Subtraction,
            };

            match argument.as_ref() {
                // Member expression update: obj.prop++ or obj[expr]++
                // Port of BuildHIR.ts lines 2513-2562
                LowerableExpression::PropertyAccess { object, property, span: member_span } => {
                    let member_loc = span_to_loc(*member_span);

                    // Lower the object
                    let obj_result = lower_expression(builder, object)?;

                    // PropertyLoad to get the current value
                    let current_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                            object: obj_result.place.clone(),
                            property: crate::hir::types::PropertyLiteral::String(property.clone()),
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    // Save the previous value place for postfix return
                    let previous_value_place = current_value.place.clone();

                    // Primitive(1) for the increment/decrement
                    let one_place = lower_value_to_temporary(
                        builder,
                        InstructionValue::Primitive(PrimitiveValue {
                            value: PrimitiveValueKind::Number(1.0),
                            loc: GENERATED_SOURCE,
                        }),
                        GENERATED_SOURCE,
                    )?;

                    // BinaryExpression: previousValue +/- 1
                    let updated_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                            operator: binary_op,
                            left: current_value.place,
                            right: one_place.place,
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    // PropertyStore to save the updated value back
                    let new_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::PropertyStore(crate::hir::PropertyStore {
                            object: obj_result.place,
                            property: crate::hir::types::PropertyLiteral::String(property.clone()),
                            value: updated_value.place.clone(),
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    // Return previous value for postfix, new value for prefix
                    let result_place = if *prefix { new_value.place } else { previous_value_place };

                    lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: result_place, loc }),
                        loc,
                    )
                }
                LowerableExpression::ComputedPropertyAccess {
                    object,
                    property,
                    span: member_span,
                } => {
                    let member_loc = span_to_loc(*member_span);

                    // Lower the object and computed property
                    let obj_result = lower_expression(builder, object)?;
                    let prop_result = lower_expression(builder, property)?;

                    // ComputedLoad to get the current value
                    let current_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                            object: obj_result.place.clone(),
                            property: prop_result.place.clone(),
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    let previous_value_place = current_value.place.clone();

                    // Primitive(1)
                    let one_place = lower_value_to_temporary(
                        builder,
                        InstructionValue::Primitive(PrimitiveValue {
                            value: PrimitiveValueKind::Number(1.0),
                            loc: GENERATED_SOURCE,
                        }),
                        GENERATED_SOURCE,
                    )?;

                    // BinaryExpression: previousValue +/- 1
                    let updated_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                            operator: binary_op,
                            left: current_value.place,
                            right: one_place.place,
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    // ComputedStore to save the updated value back
                    let new_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::ComputedStore(crate::hir::ComputedStore {
                            object: obj_result.place,
                            property: prop_result.place,
                            value: updated_value.place.clone(),
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;

                    let result_place = if *prefix { new_value.place } else { previous_value_place };

                    lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: result_place, loc }),
                        loc,
                    )
                }
                // Identifier update: x++ or ++x
                LowerableExpression::Identifier(name, ident_span) => {
                    let ident_loc = span_to_loc(*ident_span);

                    // Check for context identifiers (captured in closures)
                    // TS reference throws Todo for these (BuildHIR.ts lines 2572-2579)
                    if builder.is_context_identifier(name) {
                        return Err(CompilerError::todo(
                            "Handle UpdateExpression to variables captured within lambdas",
                            None,
                            loc,
                        ));
                    }

                    match builder.resolve_identifier(name) {
                        VariableBinding::Identifier { identifier, .. } => {
                            let lvalue_place = crate::hir::Place {
                                identifier,
                                effect: crate::hir::Effect::Unknown,
                                reactive: false,
                                loc: ident_loc,
                            };

                            // Lower the argument to get the current value
                            let arg_result = lower_expression(builder, argument)?;

                            if *prefix {
                                lower_value_to_temporary(
                                    builder,
                                    InstructionValue::PrefixUpdate(crate::hir::PrefixUpdate {
                                        lvalue: lvalue_place,
                                        operation: *operator,
                                        value: arg_result.place,
                                        loc,
                                    }),
                                    loc,
                                )
                            } else {
                                lower_value_to_temporary(
                                    builder,
                                    InstructionValue::PostfixUpdate(crate::hir::PostfixUpdate {
                                        lvalue: lvalue_place,
                                        operation: *operator,
                                        value: arg_result.place,
                                        loc,
                                    }),
                                    loc,
                                )
                            }
                        }
                        VariableBinding::NonLocal(_) => {
                            // Global variable update — TS throws Todo (BuildHIR.ts lines 2601-2608)
                            Err(CompilerError::todo(
                                "Support UpdateExpression where argument is a global",
                                None,
                                loc,
                            ))
                        }
                    }
                }
                // Other argument types (shouldn't normally happen)
                _ => Err(CompilerError::todo(
                    &format!("Handle UpdateExpression with complex argument"),
                    None,
                    loc,
                )),
            }
        }

        // =====================================================================
        // CallExpression — detect MethodCall when callee is a member expression
        // =====================================================================
        LowerableExpression::CallExpression { callee, arguments, span } => {
            let loc = span_to_loc(*span);

            // Detect method calls: callee is a PropertyAccess or ComputedPropertyAccess
            match callee.as_ref() {
                LowerableExpression::PropertyAccess { object, property, span: member_span } => {
                    let member_loc = span_to_loc(*member_span);
                    let receiver = lower_expression(builder, object)?;
                    // TS: lowerMemberExpression creates a PropertyLoad, then
                    // lowerValueToTemporary stores it. The property Place for a
                    // MethodCall is the result of evaluating the member expression
                    // (PropertyLoad), not a string literal.
                    let property_place = lower_value_to_temporary(
                        builder,
                        InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                            object: receiver.place.clone(),
                            property: crate::hir::types::PropertyLiteral::String(property.clone()),
                            loc: member_loc,
                        }),
                        member_loc,
                    )?;
                    let args = lower_arguments(builder, arguments)?;
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::MethodCall(crate::hir::MethodCall {
                            receiver: receiver.place,
                            property: property_place.place,
                            args,
                            loc,
                        }),
                        loc,
                    )
                }
                LowerableExpression::ComputedPropertyAccess {
                    object,
                    property,
                    span: computed_span,
                } => {
                    let computed_loc = span_to_loc(*computed_span);
                    let receiver = lower_expression(builder, object)?;
                    let key_place = lower_expression(builder, property)?;
                    // TS: lowerMemberExpression creates a ComputedLoad, then
                    // lowerValueToTemporary stores it.
                    let property_place = lower_value_to_temporary(
                        builder,
                        InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                            object: receiver.place.clone(),
                            property: key_place.place,
                            loc: computed_loc,
                        }),
                        computed_loc,
                    )?;
                    let args = lower_arguments(builder, arguments)?;
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::MethodCall(crate::hir::MethodCall {
                            receiver: receiver.place,
                            property: property_place.place,
                            args,
                            loc,
                        }),
                        loc,
                    )
                }
                _ => {
                    let callee_result = lower_expression(builder, callee)?;
                    let args = lower_arguments(builder, arguments)?;
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::CallExpression(crate::hir::CallExpression {
                            callee: callee_result.place,
                            args,
                            loc,
                        }),
                        loc,
                    )
                }
            }
        }

        // =====================================================================
        // NewExpression
        // =====================================================================
        LowerableExpression::NewExpression { callee, arguments, span } => {
            let loc = span_to_loc(*span);
            let callee_result = lower_expression(builder, callee)?;
            let args = lower_arguments(builder, arguments)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::NewExpression(crate::hir::NewExpression {
                    callee: callee_result.place,
                    args,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // PropertyAccess (static member expression)
        // =====================================================================
        LowerableExpression::PropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            let obj_result = lower_expression(builder, object)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: obj_result.place,
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // ComputedPropertyAccess (computed member expression)
        // =====================================================================
        LowerableExpression::ComputedPropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            let obj_result = lower_expression(builder, object)?;
            let prop_result = lower_expression(builder, property)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: obj_result.place,
                    property: prop_result.place,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // AwaitExpression
        // =====================================================================
        LowerableExpression::AwaitExpression { argument, span } => {
            let loc = span_to_loc(*span);
            let arg_result = lower_expression(builder, argument)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::Await(crate::hir::AwaitValue { value: arg_result.place, loc }),
                loc,
            )
        }

        // =====================================================================
        // ConditionalExpression — creates CFG with TernaryTerminal + BranchTerminal
        // Port of BuildHIR.ts lines 1830-1901
        // =====================================================================
        LowerableExpression::ConditionalExpression { test, consequent, alternate, span } => {
            let loc = span_to_loc(*span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = create_temporary_place(builder.environment_mut(), loc);

            // Consequent block: lower the consequent and store to shared place
            let consequent_block = builder.enter(BlockKind::Value, |builder, _block_id| {
                let consequent_result =
                    lower_expression(builder, consequent).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: consequent_result.place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // Alternate block: lower the alternate and store to shared place
            let alternate_block = builder.enter(BlockKind::Value, |builder, _block_id| {
                let alternate_result =
                    lower_expression(builder, alternate).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: alternate_result.place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            // TernaryTerminal -> test block
            builder.terminate_with_continuation(
                Terminal::Ternary(TernaryTerminal {
                    id: InstructionId(0),
                    test: test_block_id,
                    fallthrough: continuation_id,
                    loc,
                }),
                test_block,
            );

            // Lower the test expression in the test block
            let test_place = lower_expression(builder, test)?.place;

            // BranchTerminal -> continuation block
            builder.terminate_with_continuation(
                Terminal::Branch(BranchTerminal {
                    id: InstructionId(0),
                    test: test_place,
                    consequent: consequent_block,
                    alternate: alternate_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );

            // Return LoadLocal of the shared place
            lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            )
        }

        // =====================================================================
        // SequenceExpression — creates CFG with SequenceTerminal
        // Port of BuildHIR.ts lines 1781-1828
        // =====================================================================
        LowerableExpression::SequenceExpression { expressions, span } => {
            let loc = span_to_loc(*span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let place = create_temporary_place(builder.environment_mut(), loc);

            let sequence_block = builder.enter(BlockKind::Sequence, |builder, _block_id| {
                let mut last_place = None;
                for sub_expr in expressions {
                    let result =
                        lower_expression(builder, sub_expr).unwrap_or_else(|_| ExpressionResult {
                            place: create_temporary_place(builder.environment_mut(), loc),
                        });
                    last_place = Some(result.place);
                }
                if let Some(last) = last_place {
                    let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                    builder.push(Instruction {
                        id: InstructionId(0),
                        lvalue: store_lvalue,
                        value: InstructionValue::StoreLocal(StoreLocal {
                            lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                            value: last,
                            loc,
                        }),
                        effects: None,
                        loc,
                    });
                }
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            });

            builder.terminate_with_continuation(
                Terminal::Sequence(SequenceTerminal {
                    id: InstructionId(0),
                    block: sequence_block,
                    fallthrough: continuation_id,
                    loc,
                }),
                continuation_block,
            );

            lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            )
        }

        // =====================================================================
        // AssignmentExpression
        // Port of BuildHIR.ts lines 1983-2141
        // =====================================================================
        LowerableExpression::AssignmentExpression { operator, left, right, span } => {
            let loc = span_to_loc(*span);
            let right_result = lower_expression(builder, right)?;

            if *operator == AssignmentOperator::Assign {
                // Simple assignment: lower the left-hand side
                lower_assignment(builder, left, right_result.place, loc)
            } else {
                // Compound assignment: desugar to binary operation + assignment
                let binary_op = compound_assignment_to_binary(*operator);
                match binary_op {
                    Some(bin_op) => {
                        // Lower the left as an expression to get its current value
                        let left_result = lower_expression(builder, left)?;
                        // Compute left <op> right
                        let binary_result = lower_value_to_temporary(
                            builder,
                            InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                                operator: bin_op,
                                left: left_result.place,
                                right: right_result.place,
                                loc,
                            }),
                            loc,
                        )?;
                        // Assign the result back to the left
                        lower_assignment(builder, left, binary_result.place, loc)
                    }
                    None => {
                        // Logical assignments (&&=, ||=, ??=) need special CFG handling
                        // and are not yet supported. Push a Todo error and return UnsupportedNode,
                        // matching the TS behavior at BuildHIR.ts lines 2033-2041.
                        builder.errors.push_error_detail(CompilerErrorDetail::new(
                            CompilerErrorDetailOptions {
                                category: ErrorCategory::Todo,
                                reason: format!(
                                    "(BuildHIR::lowerExpression) Handle {} operators in AssignmentExpression",
                                    operator.as_str()
                                ),
                                description: None,
                                loc: Some(loc),
                                suggestions: None,
                            },
                        ));
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc }),
                            loc,
                        )
                    }
                }
            }
        }

        // =====================================================================
        // SpreadElement — lower the argument
        // =====================================================================
        LowerableExpression::SpreadElement { argument, span: _ } => {
            lower_expression(builder, argument)
        }

        // =====================================================================
        // TaggedTemplateExpression
        // Port of BuildHIR.ts lines 2335-2369
        // =====================================================================
        LowerableExpression::TaggedTemplateExpression { tag, quasi_raw, quasi_cooked, span } => {
            let loc = span_to_loc(*span);
            let tag_result = lower_expression(builder, tag)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::TaggedTemplateExpression(crate::hir::TaggedTemplateExpression {
                    tag: tag_result.place,
                    value: TemplateLiteralQuasi {
                        raw: quasi_raw.clone(),
                        cooked: quasi_cooked.clone(),
                    },
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // TypeCastExpression (TSAsExpression, TSSatisfiesExpression, TypeCast)
        // Port of BuildHIR.ts lines 2474-2508
        // =====================================================================
        LowerableExpression::TypeCastExpression { expression, annotation_kind, span } => {
            let loc = span_to_loc(*span);
            let value_result = lower_expression(builder, expression)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::TypeCastExpression(crate::hir::TypeCastExpression {
                    value: value_result.place,
                    type_: crate::hir::types::make_type(),
                    annotation_kind: *annotation_kind,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // MetaProperty (e.g., import.meta)
        // Port of BuildHIR.ts lines 2643-2663
        // =====================================================================
        LowerableExpression::MetaProperty { meta, property, span } => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(
                builder,
                InstructionValue::MetaProperty(crate::hir::MetaProperty {
                    meta: meta.clone(),
                    property: property.clone(),
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // LoadGlobal — explicitly global identifiers
        // =====================================================================
        LowerableExpression::LoadGlobal(name, span) => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(
                builder,
                InstructionValue::LoadGlobal(crate::hir::LoadGlobal {
                    binding: NonLocalBinding::Global { name: name.clone() },
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // Identifier — resolve through scope chain
        // Port of BuildHIR.ts lines 1552-1560 (lowerIdentifier + getLoadKind)
        // =====================================================================
        LowerableExpression::Identifier(name, span) => {
            let loc = span_to_loc(*span);
            match builder.resolve_identifier(name) {
                VariableBinding::Identifier { identifier, .. } => {
                    // Local variable: emit LoadLocal or LoadContext
                    let place = crate::hir::Place {
                        identifier,
                        effect: crate::hir::Effect::Unknown,
                        reactive: false,
                        loc,
                    };
                    if builder.is_context_identifier(name) {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadContext(LoadContext { place, loc }),
                            loc,
                        )
                    } else {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadLocal(LoadLocal { place, loc }),
                            loc,
                        )
                    }
                }
                VariableBinding::NonLocal(binding) => {
                    // Global: emit LoadGlobal
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadGlobal(crate::hir::LoadGlobal { binding, loc }),
                        loc,
                    )
                }
            }
        }

        // =====================================================================
        // JsxElement — lower tag, attributes, and children
        // Port of BuildHIR.ts lines 2156-2314
        // =====================================================================
        LowerableExpression::JsxElement {
            tag,
            props,
            children,
            span,
            opening_span,
            closing_span,
        } => {
            let loc = span_to_loc(*span);
            let opening_loc = span_to_loc(*opening_span);
            let closing_loc = closing_span.map_or(GENERATED_SOURCE, span_to_loc);

            // Lower the tag
            let lowered_tag = lower_jsx_tag(builder, tag)?;

            // Lower attributes
            let mut lowered_props = Vec::new();
            for attr in props {
                match attr {
                    LowerableJsxAttribute::SpreadAttribute { argument, .. } => {
                        let result = lower_expression(builder, argument)?;
                        lowered_props
                            .push(crate::hir::JsxAttribute::Spread { argument: result.place });
                    }
                    LowerableJsxAttribute::Attribute { name, value, span: attr_span } => {
                        let attr_loc = span_to_loc(*attr_span);
                        let value_place = match value {
                            Some(expr) => lower_expression(builder, expr)?.place,
                            None => {
                                // No value means `true` (e.g., `<input disabled />`)
                                lower_value_to_temporary(
                                    builder,
                                    lower_boolean(true, attr_loc),
                                    attr_loc,
                                )?
                                .place
                            }
                        };
                        lowered_props.push(crate::hir::JsxAttribute::Attribute {
                            name: name.clone(),
                            place: value_place,
                        });
                    }
                }
            }

            // Lower children
            let lowered_children = lower_jsx_children(builder, children)?;
            let children_opt =
                if lowered_children.is_empty() { None } else { Some(lowered_children) };

            lower_value_to_temporary(
                builder,
                InstructionValue::JsxExpression(crate::hir::JsxExpression {
                    tag: lowered_tag,
                    props: lowered_props,
                    children: children_opt,
                    loc,
                    opening_loc,
                    closing_loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // JsxFragment — lower children
        // Port of BuildHIR.ts lines 2316-2326
        // =====================================================================
        LowerableExpression::JsxFragment { children, span } => {
            let loc = span_to_loc(*span);
            let lowered_children = lower_jsx_children(builder, children)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::JsxFragment(crate::hir::JsxFragment {
                    children: lowered_children,
                    loc,
                }),
                loc,
            )
        }

        // =====================================================================
        // OptionalMemberExpression — creates CFG with OptionalTerminal
        // Port of BuildHIR.ts lines 2677-2785 (lowerOptionalMemberExpression)
        // =====================================================================
        LowerableExpression::OptionalMemberExpression { span, .. } => {
            let loc = span_to_loc(*span);
            let result = lower_optional_member_expression(builder, expr, None)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place: result.value, loc }),
                loc,
            )
        }

        // =====================================================================
        // OptionalCallExpression — creates CFG with OptionalTerminal
        // Port of BuildHIR.ts lines 2787-2947 (lowerOptionalCallExpression)
        // =====================================================================
        LowerableExpression::OptionalCallExpression { span, .. } => {
            let loc = span_to_loc(*span);
            let result_value = lower_optional_call_expression(builder, expr, None)?;
            lower_value_to_temporary(builder, result_value, loc)
        }

        // =====================================================================
        // FunctionExpression
        // Port of BuildHIR.ts lines 2328-2333 (lowerFunctionToValue)
        // =====================================================================
        LowerableExpression::FunctionExpression { func, span } => {
            let loc = span_to_loc(*span);
            let lowerable_func = LowerableFunction::Function(func);
            let value = lower_function_to_value(
                builder,
                &lowerable_func,
                FunctionExpressionType::FunctionExpression,
                loc,
            )?;
            lower_value_to_temporary(builder, value, loc)
        }

        // =====================================================================
        // ArrowFunctionExpression
        // Port of BuildHIR.ts lines 2328-2333 (lowerFunctionToValue)
        // =====================================================================
        LowerableExpression::ArrowFunctionExpression { func, span } => {
            let loc = span_to_loc(*span);
            let lowerable_func = LowerableFunction::ArrowFunction(func);
            let value = lower_function_to_value(
                builder,
                &lowerable_func,
                FunctionExpressionType::ArrowFunctionExpression,
                loc,
            )?;
            lower_value_to_temporary(builder, value, loc)
        }

        // Destructuring assignment targets should not appear directly in expression
        // lowering — they are handled by `lower_assignment`. If we reach here,
        // lower them as undefined (the assignment itself is handled elsewhere).
        LowerableExpression::ObjectAssignmentTarget { span, .. }
        | LowerableExpression::ArrayAssignmentTarget { span, .. } => {
            let loc = span_to_loc(*span);
            lower_value_to_temporary(builder, lower_undefined(loc), loc)
        }
    }
}

// =====================================================================================
// Expression lowering helper functions
// =====================================================================================

/// Push an instruction with the given value and return the place it was stored to.
///
/// This is the Rust equivalent of `lowerValueToTemporary()` from the TS reference.
fn lower_value_to_temporary(
    builder: &mut HirBuilder,
    value: InstructionValue,
    loc: SourceLocation,
) -> Result<ExpressionResult, CompilerError> {
    let lvalue = create_temporary_place(builder.environment_mut(), loc);
    builder.push(Instruction {
        id: InstructionId(0),
        lvalue: lvalue.clone(),
        value,
        effects: None,
        loc,
    });
    Ok(ExpressionResult { place: lvalue })
}

/// Lower a list of argument expressions into `CallArg` values.
fn lower_arguments(
    builder: &mut HirBuilder,
    arguments: &[LowerableExpression<'_>],
) -> Result<Vec<crate::hir::CallArg>, CompilerError> {
    let mut args = Vec::new();
    for arg in arguments {
        match arg {
            LowerableExpression::SpreadElement { argument, .. } => {
                let result = lower_expression(builder, argument)?;
                args.push(crate::hir::CallArg::Spread(SpreadPattern { place: result.place }));
            }
            _ => {
                let result = lower_expression(builder, arg)?;
                args.push(crate::hir::CallArg::Place(result.place));
            }
        }
    }
    Ok(args)
}

/// Lower an assignment expression (simple `=` operator).
///
/// For identifiers, this emits StoreLocal/StoreContext/StoreGlobal.
/// For member expressions, it emits PropertyStore or ComputedStore.
///
/// Port of `lowerAssignment()` from BuildHIR.ts (Identifier case) and
/// `lowerIdentifierForAssignment()`.
fn lower_assignment(
    builder: &mut HirBuilder,
    left: &LowerableExpression<'_>,
    value: crate::hir::Place,
    loc: SourceLocation,
) -> Result<ExpressionResult, CompilerError> {
    match left {
        LowerableExpression::Identifier(name, span) => {
            let ident_loc = span_to_loc(*span);
            match builder.resolve_identifier(name) {
                VariableBinding::Identifier { identifier, .. } => {
                    let place = crate::hir::Place {
                        identifier,
                        effect: crate::hir::Effect::Unknown,
                        reactive: false,
                        loc: ident_loc,
                    };

                    if builder.is_context_identifier(name) {
                        // Context variable: emit StoreContext
                        let temporary = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext(StoreContext {
                                lvalue_kind: InstructionKind::Reassign,
                                lvalue_place: place.clone(),
                                value,
                                loc,
                            }),
                            loc,
                        )?;
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadContext(LoadContext {
                                place: temporary.place,
                                loc,
                            }),
                            loc,
                        )
                    } else {
                        // Local variable: emit StoreLocal
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal(StoreLocal {
                                lvalue: LValue { place, kind: InstructionKind::Reassign },
                                value,
                                loc,
                            }),
                            loc,
                        )
                    }
                }
                VariableBinding::NonLocal(_) => {
                    // Global assignment: emit StoreGlobal
                    let temporary = lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreGlobal(StoreGlobal {
                            name: name.clone(),
                            value,
                            loc,
                        }),
                        loc,
                    )?;
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: temporary.place, loc }),
                        loc,
                    )
                }
            }
        }
        LowerableExpression::PropertyAccess { object, property, span: member_span } => {
            let member_loc = span_to_loc(*member_span);
            let obj_result = lower_expression(builder, object)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::PropertyStore(crate::hir::PropertyStore {
                    object: obj_result.place,
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    value,
                    loc: member_loc,
                }),
                loc,
            )
        }
        LowerableExpression::ComputedPropertyAccess { object, property, span: member_span } => {
            let member_loc = span_to_loc(*member_span);
            let obj_result = lower_expression(builder, object)?;
            let prop_result = lower_expression(builder, property)?;
            lower_value_to_temporary(
                builder,
                InstructionValue::ComputedStore(crate::hir::ComputedStore {
                    object: obj_result.place,
                    property: prop_result.place,
                    value,
                    loc: member_loc,
                }),
                loc,
            )
        }
        // Object destructuring assignment: `({ a, b } = expr)`
        LowerableExpression::ObjectAssignmentTarget { target, span } => {
            let target_loc = span_to_loc(*span);
            lower_object_assignment_target(builder, target, value, target_loc, loc)
        }
        // Array destructuring assignment: `([a, b] = expr)`
        LowerableExpression::ArrayAssignmentTarget { target, span } => {
            let target_loc = span_to_loc(*span);
            lower_array_assignment_target(builder, target, value, target_loc, loc)
        }
        _ => {
            // For truly unsupported assignment targets, return the value directly.
            Ok(ExpressionResult { place: value })
        }
    }
}

/// Lower an object property key.
fn lower_object_property_key(
    builder: &mut HirBuilder,
    key: &LowerableObjectPropertyKey<'_>,
) -> Result<ObjectPropertyKey, CompilerError> {
    match key {
        LowerableObjectPropertyKey::Identifier(name) => {
            Ok(ObjectPropertyKey::Identifier(name.clone()))
        }
        LowerableObjectPropertyKey::StringLiteral(value) => {
            Ok(ObjectPropertyKey::String(value.clone()))
        }
        LowerableObjectPropertyKey::NumericLiteral(value) => Ok(ObjectPropertyKey::Number(*value)),
        LowerableObjectPropertyKey::Computed(expr) => {
            let result = lower_expression(builder, expr)?;
            Ok(ObjectPropertyKey::Computed(result.place))
        }
    }
}

/// Lower a JSX tag to a `JsxTag`.
fn lower_jsx_tag(
    builder: &mut HirBuilder,
    tag: &LowerableJsxTag<'_>,
) -> Result<crate::hir::JsxTag, CompilerError> {
    match tag {
        LowerableJsxTag::BuiltIn(name) => Ok(crate::hir::JsxTag::BuiltIn(crate::hir::BuiltinTag {
            name: name.clone(),
            loc: GENERATED_SOURCE,
        })),
        LowerableJsxTag::Expression(expr) => {
            let result = lower_expression(builder, expr.as_ref())?;
            Ok(crate::hir::JsxTag::Place(result.place))
        }
    }
}

/// Lower a list of JSX children to HIR places.
fn lower_jsx_children(
    builder: &mut HirBuilder,
    children: &[LowerableJsxChild<'_>],
) -> Result<Vec<crate::hir::Place>, CompilerError> {
    let mut result = Vec::new();
    for child in children {
        if let Some(place) = lower_jsx_child(builder, child)? {
            result.push(place);
        }
    }
    Ok(result)
}

/// Lower a single JSX child, returning `None` for empty/whitespace-only text.
///
/// Port of `lowerJsxElement()` from BuildHIR.ts.
fn lower_jsx_child(
    builder: &mut HirBuilder,
    child: &LowerableJsxChild<'_>,
) -> Result<Option<crate::hir::Place>, CompilerError> {
    match child {
        LowerableJsxChild::Text(text, span) => {
            let trimmed = trim_jsx_text(text);
            if trimmed.is_empty() {
                return Ok(None);
            }
            let loc = span_to_loc(*span);
            let result = lower_value_to_temporary(
                builder,
                InstructionValue::JsxText(crate::hir::JsxTextValue { value: trimmed, loc }),
                loc,
            )?;
            Ok(Some(result.place))
        }
        LowerableJsxChild::Element(expr) => {
            let result = lower_expression(builder, expr)?;
            Ok(Some(result.place))
        }
        LowerableJsxChild::ExpressionContainer(expr, _span) => {
            let result = lower_expression(builder, expr)?;
            Ok(Some(result.place))
        }
        LowerableJsxChild::Fragment { children, span } => {
            let loc = span_to_loc(*span);
            let lowered_children = lower_jsx_children(builder, children)?;
            let result = lower_value_to_temporary(
                builder,
                InstructionValue::JsxFragment(crate::hir::JsxFragment {
                    children: lowered_children,
                    loc,
                }),
                loc,
            )?;
            Ok(Some(result.place))
        }
    }
}

/// Trim JSX text content following React's whitespace rules.
///
/// JSX text with only whitespace (including newlines) is removed.
/// Leading/trailing whitespace on each line is collapsed.
fn trim_jsx_text(text: &str) -> String {
    // React's JSX text trimming rules:
    // 1. Empty text or whitespace-only text is removed
    // 2. Newlines at start/end are removed
    // 3. Adjacent whitespace is collapsed to a single space
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut last_was_whitespace = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
            }
            last_was_whitespace = true;
        } else {
            result.push(ch);
            last_was_whitespace = false;
        }
    }
    result
}

// =====================================================================================
// Optional chaining lowering
// Port of BuildHIR.ts lines 2677-2947
// =====================================================================================

/// Result of lowering an optional member expression.
///
/// The `object` field is needed so that `lowerOptionalCallExpression` can use it
/// as the receiver for a MethodCall when the callee is `a?.b()`.
struct OptionalMemberResult {
    object: crate::hir::Place,
    value: crate::hir::Place,
}

/// Lower an optional member expression (e.g., `a?.b`, `a?.[b]`).
///
/// Port of `lowerOptionalMemberExpression()` from BuildHIR.ts lines 2677-2785.
///
/// `parent_alternate` is a block ID for the "null/undefined" fallback path that is
/// shared across a chain of optional expressions. When `None`, we create a new
/// alternate block; otherwise we reuse the parent's.
fn lower_optional_member_expression(
    builder: &mut HirBuilder,
    expr: &LowerableExpression<'_>,
    parent_alternate: Option<crate::hir::BlockId>,
) -> Result<OptionalMemberResult, CompilerError> {
    let (object_expr, property, optional, span) = match expr {
        LowerableExpression::OptionalMemberExpression { object, property, optional, span } => {
            (object.as_ref(), property, *optional, *span)
        }
        _ => {
            return Err(CompilerError::todo(
                "Expected OptionalMemberExpression",
                None,
                GENERATED_SOURCE,
            ));
        }
    };

    let loc = span_to_loc(span);
    let place = create_temporary_place(builder.environment_mut(), loc);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;
    let consequent = builder.reserve(BlockKind::Value);

    // Create or reuse the alternate block (evaluates when the chain short-circuits to undefined)
    let alternate =
        match parent_alternate {
            Some(alt_id) => alt_id,
            None => builder.enter(BlockKind::Value, |builder, _block_id| {
                let temp = lower_value_to_temporary(builder, lower_undefined(loc), loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: temp.place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            }),
        };

    // Lower the object in a test block. If the object is itself an optional expression,
    // recursively lower it to thread the alternate block.
    let mut lowered_object: Option<crate::hir::Place> = None;
    let test_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let obj_place = match object_expr {
            LowerableExpression::OptionalMemberExpression { .. } => {
                let result =
                    lower_optional_member_expression(builder, object_expr, Some(alternate))
                        .unwrap_or_else(|_| OptionalMemberResult {
                            object: create_temporary_place(builder.environment_mut(), loc),
                            value: create_temporary_place(builder.environment_mut(), loc),
                        });
                result.value
            }
            LowerableExpression::OptionalCallExpression { .. } => {
                let value = lower_optional_call_expression(builder, object_expr, Some(alternate))
                    .unwrap_or_else(|_| {
                        InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc })
                    });
                lower_value_to_temporary(builder, value, loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place
            }
            _ => {
                lower_expression(builder, object_expr)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place
            }
        };
        lowered_object = Some(obj_place.clone());
        Terminal::Branch(BranchTerminal {
            id: InstructionId(0),
            test: obj_place,
            consequent: consequent.id,
            alternate,
            fallthrough: continuation_id,
            loc,
        })
    });

    let object_place =
        lowered_object.unwrap_or_else(|| create_temporary_place(builder.environment_mut(), loc));

    // Consequent block: perform the actual member access on the (non-null) object
    builder.enter_reserved(consequent, |builder| {
        let member_value = match property {
            OptionalMemberProperty::Static(prop_name) => {
                InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: object_place.clone(),
                    property: crate::hir::types::PropertyLiteral::String(prop_name.clone()),
                    loc,
                })
            }
            OptionalMemberProperty::Computed(prop_expr) => {
                let prop_result =
                    lower_expression(builder, prop_expr).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: object_place.clone(),
                    property: prop_result.place,
                    loc,
                })
            }
        };
        let temp = lower_value_to_temporary(builder, member_value, loc).unwrap_or_else(|_| {
            ExpressionResult { place: create_temporary_place(builder.environment_mut(), loc) }
        });
        let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue: store_lvalue,
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                value: temp.place,
                loc,
            }),
            effects: None,
            loc,
        });
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc,
        })
    });

    // OptionalTerminal -> continuation block
    builder.terminate_with_continuation(
        Terminal::Optional(OptionalTerminal {
            id: InstructionId(0),
            optional,
            test: test_block,
            fallthrough: continuation_id,
            loc,
        }),
        continuation_block,
    );

    Ok(OptionalMemberResult { object: object_place, value: place })
}

/// Lower an optional call expression (e.g., `a?.()`, `a?.b()`).
///
/// Port of `lowerOptionalCallExpression()` from BuildHIR.ts lines 2787-2947.
///
/// Returns the `InstructionValue` (a `LoadLocal`) that represents the result of the call,
/// matching the TS pattern `return {kind: 'LoadLocal', place, loc: place.loc}`.
fn lower_optional_call_expression(
    builder: &mut HirBuilder,
    expr: &LowerableExpression<'_>,
    parent_alternate: Option<crate::hir::BlockId>,
) -> Result<InstructionValue, CompilerError> {
    let (callee_expr, arguments, optional, span) = match expr {
        LowerableExpression::OptionalCallExpression { callee, arguments, optional, span } => {
            (callee.as_ref(), arguments, *optional, *span)
        }
        _ => {
            return Err(CompilerError::todo(
                "Expected OptionalCallExpression",
                None,
                GENERATED_SOURCE,
            ));
        }
    };

    let loc = span_to_loc(span);
    let place = create_temporary_place(builder.environment_mut(), loc);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;
    let consequent = builder.reserve(BlockKind::Value);

    // Create or reuse the alternate block
    let alternate =
        match parent_alternate {
            Some(alt_id) => alt_id,
            None => builder.enter(BlockKind::Value, |builder, _block_id| {
                let temp = lower_value_to_temporary(builder, lower_undefined(loc), loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: store_lvalue,
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                        value: temp.place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc,
                })
            }),
        };

    // Determine if this is a method call (callee is a member expression or optional member)
    // and lower the callee in a test block.
    enum CalleeKind {
        CallExpression { callee: crate::hir::Place },
        MethodCall { receiver: crate::hir::Place, property: crate::hir::Place },
    }

    let mut callee_kind: Option<CalleeKind> = None;
    let test_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let (kind, test_place) = match callee_expr {
            // Callee is itself an optional call: recursively lower
            LowerableExpression::OptionalCallExpression { .. } => {
                let value = lower_optional_call_expression(builder, callee_expr, Some(alternate))
                    .unwrap_or_else(|_| {
                        InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc })
                    });
                let value_place = lower_value_to_temporary(builder, value, loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place;
                let tp = value_place.clone();
                (CalleeKind::CallExpression { callee: value_place }, tp)
            }
            // Callee is an optional member expression: method call pattern
            LowerableExpression::OptionalMemberExpression { .. } => {
                let result =
                    lower_optional_member_expression(builder, callee_expr, Some(alternate))
                        .unwrap_or_else(|_| OptionalMemberResult {
                            object: create_temporary_place(builder.environment_mut(), loc),
                            value: create_temporary_place(builder.environment_mut(), loc),
                        });
                let tp = result.value.clone();
                (CalleeKind::MethodCall { receiver: result.object, property: result.value }, tp)
            }
            // Callee is a regular (non-optional) member expression: method call pattern
            LowerableExpression::PropertyAccess { object, property, span: member_span } => {
                let member_loc = span_to_loc(*member_span);
                let obj_result =
                    lower_expression(builder, object).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let prop_value = InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: obj_result.place.clone(),
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    loc: member_loc,
                });
                let prop_place = lower_value_to_temporary(builder, prop_value, member_loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place;
                let tp = prop_place.clone();
                (CalleeKind::MethodCall { receiver: obj_result.place, property: prop_place }, tp)
            }
            LowerableExpression::ComputedPropertyAccess { object, property, span: member_span } => {
                let member_loc = span_to_loc(*member_span);
                let obj_result =
                    lower_expression(builder, object).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let prop_result =
                    lower_expression(builder, property).unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    });
                let computed_value = InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: obj_result.place.clone(),
                    property: prop_result.place,
                    loc: member_loc,
                });
                let computed_place = lower_value_to_temporary(builder, computed_value, member_loc)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place;
                let tp = computed_place.clone();
                (
                    CalleeKind::MethodCall { receiver: obj_result.place, property: computed_place },
                    tp,
                )
            }
            // Callee is a plain expression
            _ => {
                let callee_place = lower_expression(builder, callee_expr)
                    .unwrap_or_else(|_| ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    })
                    .place;
                let tp = callee_place.clone();
                (CalleeKind::CallExpression { callee: callee_place }, tp)
            }
        };
        callee_kind = Some(kind);
        Terminal::Branch(BranchTerminal {
            id: InstructionId(0),
            test: test_place,
            consequent: consequent.id,
            alternate,
            fallthrough: continuation_id,
            loc,
        })
    });

    let resolved_callee = callee_kind.unwrap_or(CalleeKind::CallExpression {
        callee: create_temporary_place(builder.environment_mut(), loc),
    });

    // Consequent block: perform the actual call/method call
    builder.enter_reserved(consequent, |builder| {
        let args = lower_arguments(builder, arguments).unwrap_or_default();
        let temp = create_temporary_place(builder.environment_mut(), loc);
        let call_value = match &resolved_callee {
            CalleeKind::CallExpression { callee } => {
                InstructionValue::CallExpression(crate::hir::CallExpression {
                    callee: callee.clone(),
                    args,
                    loc,
                })
            }
            CalleeKind::MethodCall { receiver, property } => {
                InstructionValue::MethodCall(crate::hir::MethodCall {
                    receiver: receiver.clone(),
                    property: property.clone(),
                    args,
                    loc,
                })
            }
        };
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue: temp.clone(),
            value: call_value,
            effects: None,
            loc,
        });
        let store_lvalue = create_temporary_place(builder.environment_mut(), loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue: store_lvalue,
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { place: place.clone(), kind: InstructionKind::Const },
                value: temp,
                loc,
            }),
            effects: None,
            loc,
        });
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc,
        })
    });

    // OptionalTerminal -> continuation block
    builder.terminate_with_continuation(
        Terminal::Optional(OptionalTerminal {
            id: InstructionId(0),
            optional,
            test: test_block,
            fallthrough: continuation_id,
            loc,
        }),
        continuation_block,
    );

    Ok(InstructionValue::LoadLocal(LoadLocal { place, loc }))
}

/// Convert a compound assignment operator to its corresponding binary operator.
fn compound_assignment_to_binary(
    op: AssignmentOperator,
) -> Option<oxc_syntax::operator::BinaryOperator> {
    use oxc_syntax::operator::BinaryOperator;
    match op {
        AssignmentOperator::Addition => Some(BinaryOperator::Addition),
        AssignmentOperator::Subtraction => Some(BinaryOperator::Subtraction),
        AssignmentOperator::Multiplication => Some(BinaryOperator::Multiplication),
        AssignmentOperator::Division => Some(BinaryOperator::Division),
        AssignmentOperator::Remainder => Some(BinaryOperator::Remainder),
        AssignmentOperator::Exponential => Some(BinaryOperator::Exponential),
        AssignmentOperator::BitwiseAnd => Some(BinaryOperator::BitwiseAnd),
        AssignmentOperator::BitwiseOR => Some(BinaryOperator::BitwiseOR),
        AssignmentOperator::BitwiseXOR => Some(BinaryOperator::BitwiseXOR),
        AssignmentOperator::ShiftLeft => Some(BinaryOperator::ShiftLeft),
        AssignmentOperator::ShiftRight => Some(BinaryOperator::ShiftRight),
        AssignmentOperator::ShiftRightZeroFill => Some(BinaryOperator::ShiftRightZeroFill),
        AssignmentOperator::Assign => None, // Not a compound operator
        // Logical assignments (&&=, ||=, ??=) need special CFG handling
        AssignmentOperator::LogicalAnd
        | AssignmentOperator::LogicalOr
        | AssignmentOperator::LogicalNullish => None,
    }
}

/// An element in an array expression that can be lowered.
#[derive(Debug)]
pub enum LowerableArrayElement<'a> {
    Expression(LowerableExpression<'a>),
    Spread(LowerableExpression<'a>, Span),
    Hole,
}

/// A property in an object expression that can be lowered.
#[derive(Debug)]
pub enum LowerableObjectProperty<'a> {
    Property {
        key: LowerableObjectPropertyKey<'a>,
        value: LowerableExpression<'a>,
        computed: bool,
        shorthand: bool,
        method: bool,
        span: Span,
    },
    Spread(LowerableExpression<'a>, Span),
}

/// A key of an object property.
#[derive(Debug)]
pub enum LowerableObjectPropertyKey<'a> {
    Identifier(String),
    StringLiteral(String),
    NumericLiteral(f64),
    Computed(LowerableExpression<'a>),
}

/// A JSX tag type for lowering.
#[derive(Debug)]
pub enum LowerableJsxTag<'a> {
    /// A built-in tag like "div", "span"
    BuiltIn(String),
    /// A component or member expression tag
    Expression(Box<LowerableExpression<'a>>),
}

/// A JSX attribute for lowering.
#[derive(Debug)]
pub enum LowerableJsxAttribute<'a> {
    Attribute { name: String, value: Option<LowerableExpression<'a>>, span: Span },
    SpreadAttribute { argument: LowerableExpression<'a>, span: Span },
}

/// A JSX child for lowering.
#[derive(Debug)]
pub enum LowerableJsxChild<'a> {
    Text(String, Span),
    Element(LowerableExpression<'a>),
    ExpressionContainer(LowerableExpression<'a>, Span),
    Fragment { children: Vec<LowerableJsxChild<'a>>, span: Span },
}

/// The property of an optional member expression (static or computed).
#[derive(Debug)]
pub enum OptionalMemberProperty<'a> {
    /// Static property access (e.g., `a?.b`)
    Static(String),
    /// Computed property access (e.g., `a?.[expr]`)
    Computed(Box<LowerableExpression<'a>>),
}

/// An expression that can be lowered to HIR.
#[derive(Debug)]
pub enum LowerableExpression<'a> {
    // Literals
    NumericLiteral(f64, Span),
    StringLiteral(String, Span),
    BooleanLiteral(bool, Span),
    NullLiteral(Span),
    Undefined(Span),
    RegExpLiteral {
        pattern: String,
        flags: String,
        span: Span,
    },
    TemplateLiteral {
        quasis: Vec<(String, Option<String>)>,
        expressions: Vec<LowerableExpression<'a>>,
        span: Span,
    },

    // Compound expressions
    ArrayExpression {
        elements: Vec<LowerableArrayElement<'a>>,
        span: Span,
    },
    ObjectExpression {
        properties: Vec<LowerableObjectProperty<'a>>,
        span: Span,
    },

    // Operators
    BinaryExpression {
        operator: oxc_syntax::operator::BinaryOperator,
        left: Box<LowerableExpression<'a>>,
        right: Box<LowerableExpression<'a>>,
        span: Span,
    },
    UnaryExpression {
        operator: oxc_syntax::operator::UnaryOperator,
        argument: Box<LowerableExpression<'a>>,
        span: Span,
    },
    LogicalExpression {
        operator: oxc_syntax::operator::LogicalOperator,
        left: Box<LowerableExpression<'a>>,
        right: Box<LowerableExpression<'a>>,
        span: Span,
    },
    UpdateExpression {
        operator: oxc_syntax::operator::UpdateOperator,
        argument: Box<LowerableExpression<'a>>,
        prefix: bool,
        span: Span,
    },

    // Calls
    CallExpression {
        callee: Box<LowerableExpression<'a>>,
        arguments: Vec<LowerableExpression<'a>>,
        span: Span,
    },
    NewExpression {
        callee: Box<LowerableExpression<'a>>,
        arguments: Vec<LowerableExpression<'a>>,
        span: Span,
    },

    // Property access
    PropertyAccess {
        object: Box<LowerableExpression<'a>>,
        property: String,
        span: Span,
    },
    ComputedPropertyAccess {
        object: Box<LowerableExpression<'a>>,
        property: Box<LowerableExpression<'a>>,
        span: Span,
    },

    // Assignment
    AssignmentExpression {
        operator: oxc_syntax::operator::AssignmentOperator,
        left: Box<LowerableExpression<'a>>,
        right: Box<LowerableExpression<'a>>,
        span: Span,
    },

    // Other
    AwaitExpression {
        argument: Box<LowerableExpression<'a>>,
        span: Span,
    },
    ConditionalExpression {
        test: Box<LowerableExpression<'a>>,
        consequent: Box<LowerableExpression<'a>>,
        alternate: Box<LowerableExpression<'a>>,
        span: Span,
    },
    SequenceExpression {
        expressions: Vec<LowerableExpression<'a>>,
        span: Span,
    },
    SpreadElement {
        argument: Box<LowerableExpression<'a>>,
        span: Span,
    },
    TaggedTemplateExpression {
        tag: Box<LowerableExpression<'a>>,
        quasi_raw: String,
        quasi_cooked: Option<String>,
        span: Span,
    },
    TypeCastExpression {
        expression: Box<LowerableExpression<'a>>,
        annotation_kind: crate::hir::TypeAnnotationKind,
        span: Span,
    },
    MetaProperty {
        meta: String,
        property: String,
        span: Span,
    },

    // Identifiers / globals
    Identifier(String, Span),
    LoadGlobal(String, Span),

    // JSX
    JsxElement {
        tag: LowerableJsxTag<'a>,
        props: Vec<LowerableJsxAttribute<'a>>,
        children: Vec<LowerableJsxChild<'a>>,
        span: Span,
        opening_span: Span,
        closing_span: Option<Span>,
    },
    JsxFragment {
        children: Vec<LowerableJsxChild<'a>>,
        span: Span,
    },

    // Optional chaining
    /// An optional member expression (e.g., `a?.b`, `a?.[b]`).
    ///
    /// In the Babel AST, this is `OptionalMemberExpression`.
    /// In oxc_ast, this comes from `ChainExpression` wrapping a `MemberExpression` with `optional: true`.
    OptionalMemberExpression {
        object: Box<LowerableExpression<'a>>,
        property: OptionalMemberProperty<'a>,
        optional: bool,
        span: Span,
    },
    /// An optional call expression (e.g., `a?.()`, `a?.b()`).
    ///
    /// In the Babel AST, this is `OptionalCallExpression`.
    /// In oxc_ast, this comes from `ChainExpression` wrapping a `CallExpression` with `optional: true`.
    OptionalCallExpression {
        callee: Box<LowerableExpression<'a>>,
        arguments: Vec<LowerableExpression<'a>>,
        optional: bool,
        span: Span,
    },

    // Function expressions — store references to the AST nodes for recursive lowering
    FunctionExpression {
        func: &'a ast::Function<'a>,
        span: Span,
    },
    ArrowFunctionExpression {
        func: &'a ast::ArrowFunctionExpression<'a>,
        span: Span,
    },

    // Destructuring assignment targets (for `[a, b] = expr` and `{a, b} = expr`)
    /// An object destructuring assignment target.
    ObjectAssignmentTarget {
        target: &'a ast::ObjectAssignmentTarget<'a>,
        span: Span,
    },
    /// An array destructuring assignment target.
    ArrayAssignmentTarget {
        target: &'a ast::ArrayAssignmentTarget<'a>,
        span: Span,
    },
}

/// Lower a primitive literal to an instruction value.
pub fn lower_primitive(value: PrimitiveValueKind, loc: SourceLocation) -> InstructionValue {
    InstructionValue::Primitive(PrimitiveValue { value, loc })
}

/// Lower a numeric literal.
pub fn lower_number(value: f64, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Number(value), loc)
}

/// Lower a string literal.
pub fn lower_string(value: String, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::String(value), loc)
}

/// Lower a boolean literal.
pub fn lower_boolean(value: bool, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Boolean(value), loc)
}

/// Lower null literal.
pub fn lower_null(loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Null, loc)
}

/// Lower undefined.
pub fn lower_undefined(loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Undefined, loc)
}

/// Convert an oxc Span to a SourceLocation.
pub fn span_to_loc(span: Span) -> SourceLocation {
    SourceLocation::Source(span)
}
