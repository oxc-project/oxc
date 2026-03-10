use oxc_ast::ast;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UpdateOperator};
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, CompilerErrorDetail,
        CompilerErrorDetailOptions, ErrorCategory, GENERATED_SOURCE, SourceLocation,
    },
    hir::{
        ArrayExpressionElement, BlockKind, BranchTerminal, Case, DeclareContext, DeclareLocal,
        Destructure, DoWhileTerminal, ForInTerminal, ForOfTerminal, ForTerminal,
        FunctionExpressionType, FunctionExpressionValue, GetIterator, GotoTerminal, GotoVariant,
        HIRFunction, IfTerminal, Instruction, InstructionId, InstructionKind, InstructionValue,
        IteratorNext, LValue, LValuePattern, LabelTerminal, LoadContext, LoadLocal,
        LogicalTerminal, LoweredFunction, NextPropertyOf, NonLocalBinding, ObjectMethodValue,
        ObjectPatternProperty, ObjectProperty, ObjectPropertyKey, ObjectPropertyType,
        OptionalTerminal, PrimitiveValue, PrimitiveValueKind, ReactFunctionType, ReturnVariant,
        SequenceTerminal, SpreadPattern, StoreContext, StoreGlobal, StoreLocal, SwitchTerminal,
        TemplateLiteralQuasi, Terminal, TernaryTerminal, TryTerminal, WhileTerminal,
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
                if let Some(default_expr) = &param.initializer {
                    // Parameter with a default value: `function f(x = defaultVal) {}`
                    // In the oxc AST, FormalParameter stores the default in `initializer`
                    // (not as an AssignmentPattern in the BindingPattern, per oxc AST docs).
                    // We must create a temporary for the actual parameter slot, then emit
                    // a conditional to resolve the default, matching TS reference behavior.
                    let param_place = create_promoted_temporary(builder, loc);
                    result.push(crate::hir::ReactiveParam::Place(param_place.clone()));
                    lower_identifier_param_with_default(
                        builder,
                        ident,
                        default_expr,
                        param_place,
                        loc,
                    );
                } else {
                    let place =
                        builder.declare_binding(&ident.name, BindingKind::Param, loc, ident.span);
                    result.push(crate::hir::ReactiveParam::Place(place));
                }
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
        if let ast::BindingPattern::BindingIdentifier(ident) = &rest.rest.argument {
            let place = builder.declare_binding(&ident.name, BindingKind::Param, loc, ident.span);
            result.push(crate::hir::ReactiveParam::Spread(SpreadPattern { place }));
        } else {
            // Destructured rest parameter
            let place = create_promoted_temporary(builder, loc);
            result.push(crate::hir::ReactiveParam::Spread(SpreadPattern { place: place.clone() }));
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
            let decl_place =
                builder.declare_binding(&ident.name, binding_kind, decl_loc, ident.span);
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

/// Lower a `BindingIdentifier` parameter that has a default value.
///
/// This corresponds to the `isAssignmentPattern()` branch in the TS reference
/// (BuildHIR.ts lines 130-151) applied to a simple identifier parameter.
///
/// For `function f(x = defaultVal) {}`:
/// - The parameter slot is `param_place` (a promoted temporary, e.g. `t0`)
/// - Emits: `let t1 = t0 === undefined ? defaultVal : t0`
/// - Then: `let x = t1` (StoreLocal)
fn lower_identifier_param_with_default(
    builder: &mut HirBuilder,
    ident: &ast::BindingIdentifier<'_>,
    default_expr: &ast::Expression<'_>,
    param_place: crate::hir::Place,
    loc: SourceLocation,
) {
    // TS: lowerReorderableExpression checks isReorderableExpression and pushes a Todo
    // error if the expression cannot be safely reordered (BuildHIR.ts lines 2863-2873).
    // Default parameter values are evaluated out of order (the function arity affects
    // when defaults are evaluated), so we restrict them to reorderable expressions.
    if !is_reorderable_expression(default_expr, true) {
        let expr_type_name = expression_type_name(default_expr);
        builder.errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Todo,
            reason: format!(
                "(BuildHIR::node.lowerReorderableExpression) Expression type `{expr_type_name}` cannot be safely reordered",
            ),
            description: None,
            loc: Some(span_to_loc(default_expr.span())),
            suggestions: None,
        }));
    }

    let ident_loc = span_to_loc(ident.span);

    // Create a temporary to hold the resolved value (either provided or default).
    // Use an unnamed temporary (matching TS `buildTemporaryPlace`) so that the
    // ternary result is inlined into the declaration during codegen, producing
    // `const x = t1 === undefined ? default : t1` instead of a separate assignment.
    let temp = create_temporary_place(builder.environment_mut(), ident_loc);

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    // Consequent: use the default value (when param_place === undefined).
    let lowerable_default = convert_expression(default_expr);
    let consequent_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        if let Ok(result) = lower_expression(builder, &lowerable_default) {
            let lvalue = create_temporary_place(builder.environment_mut(), ident_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                    value: result.place,
                    loc: ident_loc,
                }),
                effects: None,
                loc: ident_loc,
            });
        } else {
            let undef_place = create_temporary_place(builder.environment_mut(), ident_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: undef_place.clone(),
                value: lower_undefined(ident_loc),
                effects: None,
                loc: ident_loc,
            });
            let lvalue = create_temporary_place(builder.environment_mut(), ident_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                    value: undef_place,
                    loc: ident_loc,
                }),
                effects: None,
                loc: ident_loc,
            });
        }
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc: ident_loc,
        })
    });

    // Alternate: use the provided value (when param_place !== undefined).
    let alternate_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let lvalue = create_temporary_place(builder.environment_mut(), ident_loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue,
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { place: temp.clone(), kind: InstructionKind::Const },
                value: param_place.clone(),
                loc: ident_loc,
            }),
            effects: None,
            loc: ident_loc,
        });
        Terminal::Goto(GotoTerminal {
            id: InstructionId(0),
            block: continuation_id,
            variant: GotoVariant::Break,
            loc: ident_loc,
        })
    });

    // Emit the ternary terminal.
    builder.terminate_with_continuation(
        Terminal::Ternary(TernaryTerminal {
            id: InstructionId(0),
            test: test_block.id,
            fallthrough: continuation_id,
            loc: ident_loc,
        }),
        test_block,
    );

    // In the test block: compare param_place === undefined.
    let undef = lower_value_to_temporary(builder, lower_undefined(ident_loc), ident_loc);
    let test_result = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
            operator: oxc_syntax::operator::BinaryOperator::StrictEquality,
            left: param_place,
            right: undef.place,
            loc: ident_loc,
        }),
        ident_loc,
    );

    builder.terminate_with_continuation(
        Terminal::Branch(BranchTerminal {
            id: InstructionId(0),
            test: test_result.place,
            consequent: consequent_block,
            alternate: alternate_block,
            fallthrough: continuation_id,
            loc: ident_loc,
        }),
        continuation_block,
    );

    // Now declare the identifier and store the resolved temporary into it.
    let decl_place = builder.declare_binding(&ident.name, BindingKind::Let, ident_loc, ident.span);
    if builder.is_context_identifier(&ident.name) {
        let lvalue = create_temporary_place(builder.environment_mut(), loc);
        builder.push(Instruction {
            id: InstructionId(0),
            lvalue,
            value: InstructionValue::StoreContext(StoreContext {
                lvalue_kind: InstructionKind::Let,
                lvalue_place: decl_place,
                value: temp,
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
                lvalue: LValue { place: decl_place, kind: InstructionKind::Let },
                value: temp,
                loc,
            }),
            effects: None,
            loc,
        });
    }
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
            let decl_place =
                builder.declare_binding(&ident.name, binding_kind, ident_loc, ident.span);

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

        // Port of BuildHIR.ts line 3847-3854: computed properties in ObjectPattern are unsupported
        if prop.computed {
            builder.errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                crate::compiler_error::CompilerErrorDetailOptions {
                    category: crate::compiler_error::ErrorCategory::Todo,
                    reason:
                        "(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern"
                            .to_string(),
                    description: None,
                    loc: Some(prop_loc),
                    suggestions: None,
                },
            ));
            continue;
        }

        let key = lower_binding_property_key(builder, &prop.key)?;

        // Get the value pattern
        match &prop.value {
            ast::BindingPattern::BindingIdentifier(ident) => {
                if builder.will_be_context_identifier(&ident.name, ident.span) {
                    // Context variable: use a promoted temporary in the pattern,
                    // then emit a StoreContext followup to assign it to the real binding.
                    // This matches the TS reference where getStoreKind returns 'StoreContext'
                    // causing the else branch to create a temporary + followup.
                    let temp = create_promoted_temporary(builder, prop_loc);
                    properties.push(ObjectPatternProperty::Property(ObjectProperty {
                        key,
                        property_type: ObjectPropertyType::Property,
                        place: temp.clone(),
                    }));
                    followups.push((temp, FollowupPattern::Binding(&prop.value)));
                } else {
                    // Simple identifier: declare it directly and use its place in the pattern
                    let ident_loc = span_to_loc(ident.span);
                    let place =
                        builder.declare_binding(&ident.name, binding_kind, ident_loc, ident.span);
                    properties.push(ObjectPatternProperty::Property(ObjectProperty {
                        key,
                        property_type: ObjectPropertyType::Property,
                        place,
                    }));
                }
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
                let place =
                    builder.declare_binding(&ident.name, binding_kind, ident_loc, ident.span);
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
    );

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
                        if builder.will_be_context_identifier(&ident.name, ident.span) {
                            // Context variable: use a promoted temporary in the pattern,
                            // then emit a StoreContext followup to assign it to the real binding.
                            let temp = create_promoted_temporary(builder, elem_loc);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((temp, FollowupPattern::Binding(binding)));
                        } else {
                            // Simple identifier element
                            let ident_loc = span_to_loc(ident.span);
                            let place = builder.declare_binding(
                                &ident.name,
                                binding_kind,
                                ident_loc,
                                ident.span,
                            );
                            items.push(ArrayPatternElement::Place(place));
                        }
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
                let place =
                    builder.declare_binding(&ident.name, binding_kind, ident_loc, ident.span);
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
    );

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

    // TS: lowerReorderableExpression checks isReorderableExpression and pushes a Todo
    // error if the expression cannot be safely reordered (BuildHIR.ts line 4297).
    // Destructuring default values are evaluated out of order, so we restrict them
    // to reorderable expressions.
    if !is_reorderable_expression(&assign_pat.right, true) {
        let expr_type_name = expression_type_name(&assign_pat.right);
        builder.errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Todo,
            reason: format!(
                "(BuildHIR::node.lowerReorderableExpression) Expression type `{expr_type_name}` cannot be safely reordered",
            ),
            description: None,
            loc: Some(span_to_loc(assign_pat.right.span())),
            suggestions: None,
        }));
    }

    // Create a temporary to hold the resolved value (either the provided value or the default).
    // Use an unnamed temporary (matching TS `buildTemporaryPlace`) so the ternary
    // result gets inlined into the declaration during codegen.
    let temp = create_temporary_place(builder.environment_mut(), pat_loc);

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
    let undef = lower_value_to_temporary(builder, lower_undefined(pat_loc), pat_loc);
    let test_result = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
            operator: oxc_syntax::operator::BinaryOperator::StrictEquality,
            left: value,
            right: undef.place,
            loc: pat_loc,
        }),
        pat_loc,
    );

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

    // Match the TS reference's `forceTemporaries` logic (BuildHIR.ts lines 3952-3962):
    // For reassignments, force temporaries if any property has a rest element,
    // or any property value is not a simple local identifier (nested destructuring,
    // default value, context variable, or non-local binding).
    // When forceTemporaries is true, ALL property values are replaced with unnamed
    // promoted temporaries + followup assignments, which causes
    // RewriteInstructionKindsBasedOnReassignment to convert kind=Reassign to kind=Const.
    let force_temporaries = target.rest.is_some()
        || target.properties.iter().any(|prop| match prop {
            ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident_prop) => {
                // Shorthand `{ a }`: check if identifier resolves to non-local
                !matches!(
                    builder.resolve_identifier(&ident_prop.binding.name),
                    VariableBinding::Identifier { .. }
                )
            }
            ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop_prop) => {
                // `{ key: value }`: check if value is not a simple local identifier
                match &prop_prop.binding {
                    ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                        !matches!(
                            builder.resolve_identifier(&ident.name),
                            VariableBinding::Identifier { .. }
                        )
                    }
                    _ => true, // nested patterns, defaults, member expressions
                }
            }
        });

    for prop in &target.properties {
        match prop {
            ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident_prop) => {
                // Shorthand: `{ a }` or `{ a = defaultVal }`
                let name = &ident_prop.binding.name;
                let ident_loc = span_to_loc(ident_prop.binding.span);
                let key = ObjectPropertyKey::Identifier(name.to_string());

                if force_temporaries
                    || ident_prop.init.is_some()
                    || builder.is_context_identifier(name)
                {
                    // forceTemporaries, has default value, or is a context variable:
                    // create a promoted temporary and defer the assignment via a followup.
                    // For context variables, the followup's `lower_assignment` will
                    // emit a StoreContext instruction.
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
                        if force_temporaries || builder.is_context_identifier(&ident.name) {
                            // forceTemporaries or context variable: create temporary + followup
                            let temp = create_promoted_temporary(builder, prop_loc);
                            properties.push(ObjectPatternProperty::Property(ObjectProperty {
                                key,
                                property_type: ObjectPropertyType::Property,
                                place: temp.clone(),
                            }));
                            followups.push((
                                temp,
                                AssignmentFollowup::IdentifierWithDefault {
                                    name: ident.name.to_string(),
                                    default_expr: None,
                                    loc: prop_loc,
                                },
                            ));
                        } else {
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
            if force_temporaries || builder.is_context_identifier(&ident.name) {
                let temp = create_promoted_temporary(builder, rest_loc);
                properties
                    .push(ObjectPatternProperty::Spread(SpreadPattern { place: temp.clone() }));
                followups.push((
                    temp,
                    AssignmentFollowup::IdentifierWithDefault {
                        name: ident.name.to_string(),
                        default_expr: None,
                        loc: rest_loc,
                    },
                ));
            } else {
                let place = resolve_identifier_for_reassignment(
                    builder,
                    &ident.name,
                    span_to_loc(ident.span),
                );
                properties.push(ObjectPatternProperty::Spread(SpreadPattern { place }));
            }
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
    );

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
    // Match the TS reference's `forceTemporaries` logic (BuildHIR.ts lines 3822-3830):
    // For reassignments, force temporaries if any element is not a simple local identifier,
    // i.e. has nested destructuring, is a context variable, or is non-local.
    let force_temporaries = target.elements.iter().any(|el| {
        match el {
            Some(ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident)) => {
                builder.is_context_identifier(&ident.name)
                    || !matches!(
                        builder.resolve_identifier(&ident.name),
                        VariableBinding::Identifier { .. }
                    )
            }
            Some(_) => true, // nested patterns or defaults
            None => false,   // holes are fine
        }
    });

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
                        if force_temporaries {
                            // Context variable or forced: use temporary + followup
                            let temp = create_promoted_temporary(builder, elem_loc);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((
                                temp,
                                AssignmentFollowup::IdentifierWithDefault {
                                    name: ident.name.to_string(),
                                    default_expr: None,
                                    loc: elem_loc,
                                },
                            ));
                        } else {
                            let place = resolve_identifier_for_reassignment(
                                builder,
                                &ident.name,
                                span_to_loc(ident.span),
                            );
                            items.push(ArrayPatternElement::Place(place));
                        }
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
            if force_temporaries || builder.is_context_identifier(&ident.name) {
                let temp = create_promoted_temporary(builder, rest_loc);
                items.push(ArrayPatternElement::Spread(SpreadPattern { place: temp.clone() }));
                followups.push((
                    temp,
                    AssignmentFollowup::IdentifierWithDefault {
                        name: ident.name.to_string(),
                        default_expr: None,
                        loc: rest_loc,
                    },
                ));
            } else {
                let place = resolve_identifier_for_reassignment(
                    builder,
                    &ident.name,
                    span_to_loc(ident.span),
                );
                items.push(ArrayPatternElement::Spread(SpreadPattern { place }));
            }
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
    );

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
                    lower_conditional_default_assignment(builder, temp_place, default, ident_loc)
                } else {
                    temp_place
                };
                // Assign to the identifier
                let lowerable = LowerableExpression::Identifier(name, Span::default());
                lower_assignment(builder, &lowerable, resolved, loc)?;
            }
            AssignmentFollowup::TargetWithDefault { target, default_expr, loc: target_loc } => {
                let resolved = if let Some(default) = default_expr {
                    lower_conditional_default_assignment(builder, temp_place, default, target_loc)
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
) -> crate::hir::Place {
    // Use an unnamed temporary (matching TS `buildTemporaryPlace`) so the ternary
    // result gets inlined into the declaration during codegen.
    let temp = create_temporary_place(builder.environment_mut(), loc);

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
    let undef = lower_value_to_temporary(builder, lower_undefined(loc), loc);
    let test_result = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
            operator: oxc_syntax::operator::BinaryOperator::StrictEquality,
            left: value,
            right: undef.place,
            loc,
        }),
        loc,
    );

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

    temp
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
                        effects: None,
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

/// Collect all binding names declared in a function's scope.
///
/// This performs a shallow walk of the function body (not recursing into inner
/// functions) to collect ALL variable and function declaration names. This
/// mimics Babel's scope analysis which pre-knows all bindings before lowering.
///
/// Used to populate `HirBuilder::scope_binding_names` so that
/// `gather_captured_context` can correctly identify captured variables even
/// when the capturing function appears before the variable declaration.
fn collect_all_scope_binding_names(func: &LowerableFunction<'_>) -> rustc_hash::FxHashSet<String> {
    let mut names = rustc_hash::FxHashSet::default();

    // Collect parameter names
    let params = match func {
        LowerableFunction::Function(f) => &f.params,
        LowerableFunction::ArrowFunction(a) => &a.params,
    };
    for param in &params.items {
        collect_binding_pattern_names(&param.pattern, &mut names);
    }
    if let Some(rest) = &params.rest {
        collect_binding_pattern_names(&rest.rest.argument, &mut names);
    }

    // Collect body binding names
    match func {
        LowerableFunction::Function(f) => {
            if let Some(body) = &f.body {
                for stmt in &body.statements {
                    collect_stmt_binding_names(stmt, &mut names);
                }
            }
        }
        LowerableFunction::ArrowFunction(a) => {
            for stmt in &a.body.statements {
                collect_stmt_binding_names(stmt, &mut names);
            }
        }
    }

    names
}

/// Collect binding names from a statement (shallow - doesn't recurse into inner functions).
fn collect_stmt_binding_names(
    stmt: &ast::Statement<'_>,
    names: &mut rustc_hash::FxHashSet<String>,
) {
    match stmt {
        ast::Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                collect_binding_pattern_names(&declarator.id, names);
            }
        }
        ast::Statement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                names.insert(id.name.to_string());
            }
        }
        ast::Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_stmt_binding_names(s, names);
            }
        }
        ast::Statement::IfStatement(if_stmt) => {
            collect_stmt_binding_names(&if_stmt.consequent, names);
            if let Some(alt) = &if_stmt.alternate {
                collect_stmt_binding_names(alt, names);
            }
        }
        ast::Statement::WhileStatement(while_stmt) => {
            collect_stmt_binding_names(&while_stmt.body, names);
        }
        ast::Statement::DoWhileStatement(do_while) => {
            collect_stmt_binding_names(&do_while.body, names);
        }
        ast::Statement::ForStatement(for_stmt) => {
            if let Some(ast::ForStatementInit::VariableDeclaration(decl)) = &for_stmt.init {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, names);
                }
            }
            collect_stmt_binding_names(&for_stmt.body, names);
        }
        ast::Statement::ForOfStatement(for_of) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, names);
                }
            }
            collect_stmt_binding_names(&for_of.body, names);
        }
        ast::Statement::ForInStatement(for_in) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_in.left {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, names);
                }
            }
            collect_stmt_binding_names(&for_in.body, names);
        }
        ast::Statement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                collect_stmt_binding_names(s, names);
            }
            if let Some(handler) = &try_stmt.handler {
                if let Some(param) = &handler.param {
                    collect_binding_pattern_names(&param.pattern, names);
                }
                for s in &handler.body.body {
                    collect_stmt_binding_names(s, names);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for s in &finalizer.body {
                    collect_stmt_binding_names(s, names);
                }
            }
        }
        ast::Statement::SwitchStatement(switch) => {
            for case in &switch.cases {
                for s in &case.consequent {
                    collect_stmt_binding_names(s, names);
                }
            }
        }
        ast::Statement::LabeledStatement(labeled) => {
            collect_stmt_binding_names(&labeled.body, names);
        }
        _ => {}
    }
}

/// Collect import bindings from a program's body.
///
/// This corresponds to what Babel's scope analysis provides via
/// `parentFunction.scope.parent.getBinding()` in the TS `HIRBuilder.ts`.
/// It scans the top-level program statements for import declarations and
/// builds a map from local binding names to their `NonLocalBinding` info.
///
/// Also includes module-local declarations (top-level `const`/`let`/`var`/
/// `function` that are not the function being compiled) as `ModuleLocal`.
pub fn collect_import_bindings(
    program_body: &[ast::Statement<'_>],
) -> FxHashMap<String, NonLocalBinding> {
    let mut bindings = FxHashMap::default();

    for stmt in program_body {
        match stmt {
            ast::Statement::ImportDeclaration(import_decl) => {
                let module = import_decl.source.value.to_string();
                if let Some(specifiers) = &import_decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                bindings.insert(
                                    s.local.name.to_string(),
                                    NonLocalBinding::ImportDefault {
                                        name: s.local.name.to_string(),
                                        module: module.clone(),
                                    },
                                );
                            }
                            ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                bindings.insert(
                                    s.local.name.to_string(),
                                    NonLocalBinding::ImportNamespace {
                                        name: s.local.name.to_string(),
                                        module: module.clone(),
                                    },
                                );
                            }
                            ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let imported = match &s.imported {
                                    ast::ModuleExportName::IdentifierName(id) => {
                                        id.name.to_string()
                                    }
                                    ast::ModuleExportName::IdentifierReference(id) => {
                                        id.name.to_string()
                                    }
                                    ast::ModuleExportName::StringLiteral(lit) => {
                                        lit.value.to_string()
                                    }
                                };
                                bindings.insert(
                                    s.local.name.to_string(),
                                    NonLocalBinding::ImportSpecifier {
                                        name: s.local.name.to_string(),
                                        module: module.clone(),
                                        imported,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            // Top-level variable declarations are ModuleLocal bindings.
            ast::Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let ast::BindingPattern::BindingIdentifier(id) = &d.id {
                        bindings.insert(
                            id.name.to_string(),
                            NonLocalBinding::ModuleLocal { name: id.name.to_string() },
                        );
                    }
                }
            }
            // Top-level function declarations are ModuleLocal bindings.
            ast::Statement::FunctionDeclaration(f) => {
                if let Some(id) = &f.id {
                    bindings.insert(
                        id.name.to_string(),
                        NonLocalBinding::ModuleLocal { name: id.name.to_string() },
                    );
                }
            }
            _ => {}
        }
    }

    bindings
}

/// Lower an oxc AST function into HIR.
///
/// Port of `lower()` from `HIR/BuildHIR.ts` (lines 72-264).
///
/// # Errors
/// Returns a `CompilerError` if lowering fails due to unsupported syntax.
// Cannot generalize over BuildHasher: .clippy.toml disallows std::collections::HashMap
#[expect(clippy::implicit_hasher)]
pub fn lower(
    env: &Environment,
    fn_type: ReactFunctionType,
    func: &LowerableFunction<'_>,
    outer_bindings: FxHashMap<String, NonLocalBinding>,
) -> Result<HIRFunction, CompilerError> {
    // Find context identifiers (variables captured by inner closures)
    let context_identifiers = match func {
        LowerableFunction::Function(f) => find_context_identifiers(f),
        LowerableFunction::ArrowFunction(a) => find_context_identifiers_arrow(a),
    };

    let mut builder = HirBuilder::new(env.clone(), None, context_identifiers, outer_bindings);

    // Pre-collect all binding names in this function's scope before lowering.
    // This mimics Babel's scope analysis which knows about all bindings before
    // any lowering occurs. It enables `gather_captured_context` to correctly
    // identify captured variables even when the capturing function appears before
    // the variable declaration (e.g., `const foo = () => bar; const bar = 3;`).
    let scope_binding_names = collect_all_scope_binding_names(func);
    builder.set_scope_binding_names(scope_binding_names);

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
            effects: None,
            loc: GENERATED_SOURCE,
        }),
        None,
    );

    let (body, mut built_env) = builder.build_with_env();
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
/// as a binding in the outer function's scope. If so, it is a captured
/// context variable.
///
/// Uses `has_scope_binding()` to check against ALL bindings in the outer
/// function's scope (pre-collected from the AST), not just those declared
/// so far in the sequential lowering. This correctly handles hoisted
/// references (e.g., `const foo = () => bar; const bar = 3;`).
///
/// For variables that haven't been declared yet in the builder, uses
/// `pre_declare_binding()` to eagerly create a binding entry. When the
/// actual declaration is processed later, `resolve_binding` will recognize
/// the pre-declared entry and reuse its identifier.
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

    // Walk the inner function body to find free variable references
    let inner_refs = collect_inner_function_references(func);

    for (name, loc) in inner_refs {
        // Check if the name is bound anywhere in the outer function's scope.
        // This uses the pre-collected scope_binding_names set which knows about
        // ALL declarations, including those not yet processed in the sequential lowering.
        if outer_builder.has_scope_binding(&name) {
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
        ast::Expression::UpdateExpression(update) => match &update.argument {
            ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                refs.push((ident.name.to_string(), span_to_loc(ident.span)));
            }
            ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                collect_expression_refs(&member.object, refs);
            }
            ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                collect_expression_refs(&member.object, refs);
                collect_expression_refs(&member.expression, refs);
            }
            _ => {}
        },
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
        ast::Expression::TSInstantiationExpression(ts_inst) => {
            collect_expression_refs(&ts_inst.expression, refs);
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
        // FunctionExpression and ArrowFunctionExpression: recurse into the
        // function body to find transitive captures. The TS reference uses
        // Babel's fn.traverse() which traverses into nested functions, so
        // `const f0 = function() { z.b = 1; }` correctly captures `z` from
        // the outer scope even though the reference is inside a nested function.
        //
        // We collect refs from the nested function's body but filter out
        // any references to the nested function's own parameters and local
        // declarations, since those are not captures from the outer scope.
        ast::Expression::FunctionExpression(func) => {
            let mut nested_declared = rustc_hash::FxHashSet::default();
            // Declare function's own name
            if let Some(id) = &func.id {
                nested_declared.insert(id.name.to_string());
            }
            // Declare parameters
            for param in &func.params.items {
                collect_binding_pattern_names(&param.pattern, &mut nested_declared);
            }
            if let Some(rest) = &func.params.rest {
                collect_binding_pattern_names(&rest.rest.argument, &mut nested_declared);
            }
            // Collect refs from body, tracking nested declarations
            if let Some(body) = &func.body {
                let mut nested_refs = Vec::new();
                for stmt in &body.statements {
                    collect_statement_refs(stmt, &mut nested_refs, &mut nested_declared);
                }
                // Only add refs that are NOT declared within the nested function
                for r in nested_refs {
                    if !nested_declared.contains(&r.0) {
                        refs.push(r);
                    }
                }
            }
        }
        ast::Expression::ArrowFunctionExpression(arrow) => {
            let mut nested_declared = rustc_hash::FxHashSet::default();
            // Declare parameters
            for param in &arrow.params.items {
                collect_binding_pattern_names(&param.pattern, &mut nested_declared);
            }
            if let Some(rest) = &arrow.params.rest {
                collect_binding_pattern_names(&rest.rest.argument, &mut nested_declared);
            }
            // Collect refs from body, tracking nested declarations
            let mut nested_refs = Vec::new();
            for stmt in &arrow.body.statements {
                collect_statement_refs(stmt, &mut nested_refs, &mut nested_declared);
            }
            // Only add refs that are NOT declared within the nested function
            for r in nested_refs {
                if !nested_declared.contains(&r.0) {
                    refs.push(r);
                }
            }
        }
        // Literals and other expression types that don't reference identifiers
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
    // Tag name - identifier (e.g., <Component>)
    if let ast::JSXElementName::IdentifierReference(ident) = &element.opening_element.name
        && ident.name.starts_with(|c: char| c.is_ascii_uppercase())
    {
        refs.push((ident.name.to_string(), span_to_loc(ident.span)));
    }
    // Tag name - member expression (e.g., <localVar.Stringify>)
    // Walk the chain to find the root identifier
    if let ast::JSXElementName::MemberExpression(member) = &element.opening_element.name {
        let mut current = &member.object;
        loop {
            match current {
                ast::JSXMemberExpressionObject::IdentifierReference(ident) => {
                    refs.push((ident.name.to_string(), span_to_loc(ident.span)));
                    break;
                }
                ast::JSXMemberExpressionObject::MemberExpression(inner) => {
                    current = &inner.object;
                }
                ast::JSXMemberExpressionObject::ThisExpression(_) => {
                    break;
                }
            }
        }
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

    // Use the OUTER builder's context_identifiers for the inner function.
    //
    // In the TS reference, inner functions share the same `Environment` (and
    // thus the same `#contextIdentifiers` set) as the outer function. The set
    // is populated once by `findContextIdentifiers` for the top-level compiled
    // function and never modified. This means a variable that is merely captured
    // (referenced but not reassigned) will NOT be a context identifier — only
    // variables that are reassigned by/in inner functions are context identifiers.
    //
    // Previously, we computed `find_context_identifiers(inner_func)` and merged
    // ALL captured variable spans into it, which incorrectly marked non-reassigned
    // captured variables as context identifiers. This caused `LoadContext` to be
    // emitted where the TS reference would emit `LoadLocal`, breaking DCE behavior
    // (since `LoadContext` is non-pruneable but `LoadLocal` is pruneable).
    let merged_context = outer_builder.context_identifiers().clone();

    // Phase 1: Resolve captured context variables on the OUTER builder FIRST.
    // This must happen before cloning the environment, because pre_declare_binding
    // allocates new IdentifierIds from the outer builder's environment. If we cloned
    // the environment first, the inner builder would have a stale ID counter and
    // could allocate the same IDs, causing collisions (e.g., both parameter `x` and
    // captured `factorial` would get the same IdentifierId).
    let mut context_entries: Vec<(String, SourceLocation, crate::hir::Place, BindingKind, Span)> =
        Vec::new();
    for (name, ctx_loc) in &captured_context {
        match outer_builder.resolve_identifier(name) {
            VariableBinding::Identifier { identifier, binding_kind } => {
                // Variable is already declared in the outer builder — use its identifier
                let decl_span = outer_builder.get_binding_decl_span(name).unwrap_or_default();
                context_entries.push((
                    name.clone(),
                    *ctx_loc,
                    crate::hir::Place {
                        identifier,
                        effect: crate::hir::Effect::Unknown,
                        reactive: false,
                        loc: *ctx_loc,
                    },
                    binding_kind,
                    decl_span,
                ));
            }
            VariableBinding::NonLocal(_) => {
                // Variable is not yet declared in the outer builder (hoisted reference —
                // e.g., `const foo = () => bar; const bar = 3;`).
                // Eagerly pre-declare it so it gets a real HIR identifier that will be
                // reused when the actual declaration is processed later.
                let place = outer_builder.pre_declare_binding(name, *ctx_loc);
                let decl_span = outer_builder.get_binding_decl_span(name).unwrap_or_default();
                context_entries.push((name.clone(), *ctx_loc, place, BindingKind::Let, decl_span));
            }
        }
    }

    // Phase 2: NOW clone the environment — it has up-to-date ID counters that
    // include any identifiers allocated by pre_declare_binding above.
    let env = outer_builder.environment();
    let mut inner_builder =
        HirBuilder::new(env.clone(), None, merged_context, outer_builder.outer_bindings().clone());

    // Pre-collect scope binding names for the inner function too, so nested
    // functions can also correctly identify captured hoisted variables.
    let inner_scope_names = collect_all_scope_binding_names(func);
    inner_builder.set_scope_binding_names(inner_scope_names);

    // Phase 3: Register the resolved context variables on the inner builder
    // and build the context places vector.
    let mut context_places = Vec::new();
    for (name, _ctx_loc, place, binding_kind, decl_span) in context_entries {
        inner_builder.register_outer_binding(
            &name,
            place.identifier.clone(),
            binding_kind,
            decl_span,
        );
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
            effects: None,
            loc: GENERATED_SOURCE,
        }),
        None,
    );

    let (mut body, mut built_env) = inner_builder.build_with_env();

    // Remove unreachable blocks from the inner function's HIR.
    // This matches TS behavior: HIRBuilder.build() calls getReversePostorderedBlocks()
    // which only returns reachable blocks, effectively removing any unreachable blocks
    // (e.g. the fallthrough block after an expression-body arrow function's implicit
    // return is unreachable since Return has no successors).
    // Without this, `has_single_exit_return_terminal` in inline_iife counts the extra
    // unreachable Return block and incorrectly takes the multi-return path.
    crate::hir::hir_builder::reverse_postorder_blocks(&mut body);
    crate::hir::hir_builder::remove_unreachable_for_updates(&mut body);
    crate::hir::hir_builder::remove_dead_do_while_statements(&mut body);
    crate::hir::hir_builder::remove_unnecessary_try_catch(&mut body);
    crate::hir::hir_builder::mark_instruction_ids(&mut body);
    crate::hir::hir_builder::mark_predecessors(&mut body);

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
    lower_block_statement_with_extra_hoistable(builder, stmts, None)
}

/// Lower a block statement with optional extra hoistable bindings.
///
/// The `extra_hoistable` parameter allows callers to inject additional bindings
/// into the hoisting analysis. This is used for catch clause parameters, which
/// are not declared by any statement in the catch body but ARE bindings in the
/// catch handler scope. In Babel, `stmt.scope.bindings` automatically includes
/// catch parameters; here we must add them explicitly.
///
/// # Errors
/// Returns a `CompilerError` if any statement in the block cannot be lowered.
fn lower_block_statement_with_extra_hoistable(
    builder: &mut HirBuilder,
    stmts: &[LowerableStatement<'_>],
    extra_hoistable: Option<(&str, HoistableBinding)>,
) -> Result<(), CompilerError> {
    // =========================================================================
    // Hoisting analysis (port of BlockStatement handling in TS BuildHIR.ts)
    //
    // Collect all hoistable bindings in this block scope. For each statement,
    // before lowering it, scan for references to not-yet-declared bindings
    // that appear inside inner functions. For each such reference, emit a
    // `DeclareContext` with the appropriate Hoisted* kind so that the variable
    // becomes a context variable (matching the TS reference behavior).
    // =========================================================================
    let mut hoistable: rustc_hash::FxHashMap<String, HoistableBinding> =
        rustc_hash::FxHashMap::default();

    // Phase 1: collect all bindings declared in this block.
    for stmt in stmts {
        collect_hoistable_bindings_from_statement(stmt, &mut hoistable);
    }

    // Add extra hoistable bindings (e.g. catch clause parameters).
    // These are bindings in the scope that are not declared by statements
    // but can still be referenced by inner functions and need hoisting.
    if let Some((name, binding)) = extra_hoistable {
        hoistable.insert(name.to_string(), binding);
    }

    // Phase 2: for each statement, hoist as needed, then lower.
    for stmt in stmts {
        // Scan the statement for references to hoistable identifiers in inner functions.
        let mut will_hoist: Vec<String> = Vec::new();
        let fn_depth_start = u32::from(matches!(stmt, LowerableStatement::FunctionDeclaration(_)));
        scan_for_hoistable_refs(stmt, fn_depth_start, &hoistable, &mut will_hoist);

        // Emit DeclareContext for each hoisted identifier.
        // NOTE: We must emit DeclareContext BEFORE removing the declared bindings from
        // the hoistable set, because the DeclareContext emission needs to look up the
        // binding info from `hoistable`. In the TS reference, the binding info comes
        // from `stmt.scope.getBinding()` which is never modified by the deletion loop.
        // Our `hoistable` map serves both roles (scan filter + binding info lookup),
        // so we must ensure the binding info is still available during emission.
        for name in &will_hoist {
            if builder.is_hoisted_identifier(name) {
                continue; // Already hoisted
            }
            let Some(binding_info) = hoistable.get(name).cloned() else {
                continue;
            };
            let kind = binding_info.hoisted_kind();

            // Use pre_declare_binding so that when the actual declaration is
            // processed later (e.g., `const bar = 3` after `const foo = () => bar`),
            // `declare_binding` will recognize the pre-declared entry and reuse the
            // same identifier instead of creating a new `bar_0` due to a name collision.
            // The binding kind will be updated by `declare_binding` when the real
            // declaration is processed.
            let loc = GENERATED_SOURCE;
            let decl_place = builder.pre_declare_binding(name, loc);

            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::DeclareContext(DeclareContext {
                    lvalue_kind: kind,
                    lvalue_place: decl_place,
                    loc,
                }),
                effects: None,
                loc,
            });
            let hoisted_decl_span = builder.get_binding_decl_span(name).unwrap_or_default();
            builder.add_hoisted_identifier(name, hoisted_decl_span);
        }

        // Remove bindings that are declared by this statement from hoistable set.
        // This must happen AFTER DeclareContext emission (which reads from hoistable)
        // but BEFORE the next iteration, so subsequent statements don't try to hoist
        // already-declared bindings.
        remove_declared_bindings_from_statement(stmt, &mut hoistable);

        lower_statement(builder, stmt)?;
    }
    Ok(())
}

/// Information about a hoistable binding in a block scope.
#[derive(Clone, Debug)]
struct HoistableBinding {
    /// The original declaration kind from the source.
    decl_kind: ast::VariableDeclarationKind,
    /// Whether this is a function declaration.
    is_function_decl: bool,
}

impl HoistableBinding {
    /// Get the hoisted `InstructionKind` for this binding.
    fn hoisted_kind(&self) -> InstructionKind {
        if self.is_function_decl {
            InstructionKind::HoistedFunction
        } else {
            match self.decl_kind {
                ast::VariableDeclarationKind::Const
                | ast::VariableDeclarationKind::Var
                | ast::VariableDeclarationKind::Using
                | ast::VariableDeclarationKind::AwaitUsing => InstructionKind::HoistedConst,
                ast::VariableDeclarationKind::Let => InstructionKind::HoistedLet,
            }
        }
    }
}

/// Collect all binding names declared in a statement.
fn collect_hoistable_bindings_from_statement(
    stmt: &LowerableStatement<'_>,
    hoistable: &mut rustc_hash::FxHashMap<String, HoistableBinding>,
) {
    match stmt {
        LowerableStatement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                collect_binding_names_from_pattern(&declarator.id, decl.kind, hoistable);
            }
        }
        LowerableStatement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                hoistable.insert(
                    id.name.to_string(),
                    HoistableBinding {
                        decl_kind: ast::VariableDeclarationKind::Let,
                        is_function_decl: true,
                    },
                );
            }
        }
        LowerableStatement::ForStatement(for_stmt) => {
            if let Some(ast::ForStatementInit::VariableDeclaration(decl)) = &for_stmt.init {
                for declarator in &decl.declarations {
                    collect_binding_names_from_pattern(&declarator.id, decl.kind, hoistable);
                }
            }
        }
        LowerableStatement::ForOfStatement(for_of) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left {
                for declarator in &decl.declarations {
                    collect_binding_names_from_pattern(&declarator.id, decl.kind, hoistable);
                }
            }
        }
        LowerableStatement::ForInStatement(for_in) => {
            if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_in.left {
                for declarator in &decl.declarations {
                    collect_binding_names_from_pattern(&declarator.id, decl.kind, hoistable);
                }
            }
        }
        LowerableStatement::BlockStatement(_block) => {
            // Do NOT recurse into nested blocks. The TS reference uses
            // `stmt.scope.bindings` which only returns bindings declared at
            // the current block scope level. Inner block bindings are handled
            // when that inner block is lowered separately.
        }
        _ => {}
    }
}

/// Collect binding names from a binding pattern.
fn collect_binding_names_from_pattern(
    pattern: &ast::BindingPattern<'_>,
    kind: ast::VariableDeclarationKind,
    hoistable: &mut rustc_hash::FxHashMap<String, HoistableBinding>,
) {
    match pattern {
        ast::BindingPattern::BindingIdentifier(ident) => {
            hoistable.insert(
                ident.name.to_string(),
                HoistableBinding { decl_kind: kind, is_function_decl: false },
            );
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names_from_pattern(&prop.value, kind, hoistable);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names_from_pattern(&rest.argument, kind, hoistable);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names_from_pattern(elem, kind, hoistable);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names_from_pattern(&rest.argument, kind, hoistable);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names_from_pattern(&assign.left, kind, hoistable);
        }
    }
}

/// Remove bindings declared by a statement from the hoistable set.
/// This is called after scanning but before lowering, to mark that subsequent
/// statements no longer need hoisting for these names.
fn remove_declared_bindings_from_statement(
    stmt: &LowerableStatement<'_>,
    hoistable: &mut rustc_hash::FxHashMap<String, HoistableBinding>,
) {
    match stmt {
        LowerableStatement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                remove_binding_names_from_pattern(&declarator.id, hoistable);
            }
        }
        LowerableStatement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                hoistable.remove(id.name.as_str());
            }
        }
        _ => {}
    }
}

/// Remove binding names from a binding pattern from the hoistable set.
fn remove_binding_names_from_pattern(
    pattern: &ast::BindingPattern<'_>,
    hoistable: &mut rustc_hash::FxHashMap<String, HoistableBinding>,
) {
    match pattern {
        ast::BindingPattern::BindingIdentifier(ident) => {
            hoistable.remove(ident.name.as_str());
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                remove_binding_names_from_pattern(&prop.value, hoistable);
            }
            if let Some(rest) = &obj.rest {
                remove_binding_names_from_pattern(&rest.argument, hoistable);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                remove_binding_names_from_pattern(elem, hoistable);
            }
            if let Some(rest) = &arr.rest {
                remove_binding_names_from_pattern(&rest.argument, hoistable);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            remove_binding_names_from_pattern(&assign.left, hoistable);
        }
    }
}

/// Scan a statement for references to hoistable identifiers that appear in inner functions.
/// `fn_depth` tracks the nesting depth of function expressions (0 = top level).
fn scan_for_hoistable_refs(
    stmt: &LowerableStatement<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match stmt {
        LowerableStatement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    scan_expr_for_hoistable_refs(init, fn_depth, hoistable, will_hoist);
                }
            }
        }
        LowerableStatement::ExpressionStatement(expr_stmt) => {
            scan_expr_for_hoistable_refs(&expr_stmt.expression, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                scan_expr_for_hoistable_refs(arg, fn_depth, hoistable, will_hoist);
            }
        }
        LowerableStatement::IfStatement(if_stmt) => {
            scan_expr_for_hoistable_refs(&if_stmt.test, fn_depth, hoistable, will_hoist);
            let cons = convert_statement(&if_stmt.consequent);
            scan_for_hoistable_refs(&cons, fn_depth, hoistable, will_hoist);
            if let Some(alt) = &if_stmt.alternate {
                let alt = convert_statement(alt);
                scan_for_hoistable_refs(&alt, fn_depth, hoistable, will_hoist);
            }
        }
        LowerableStatement::WhileStatement(while_stmt) => {
            scan_expr_for_hoistable_refs(&while_stmt.test, fn_depth, hoistable, will_hoist);
            let body = convert_statement(&while_stmt.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::DoWhileStatement(do_while) => {
            let body = convert_statement(&do_while.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&do_while.test, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                match init {
                    ast::ForStatementInit::VariableDeclaration(decl) => {
                        for declarator in &decl.declarations {
                            if let Some(init_expr) = &declarator.init {
                                scan_expr_for_hoistable_refs(
                                    init_expr, fn_depth, hoistable, will_hoist,
                                );
                            }
                        }
                    }
                    _ => {
                        scan_expr_for_hoistable_refs(
                            init.to_expression(),
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
            if let Some(test) = &for_stmt.test {
                scan_expr_for_hoistable_refs(test, fn_depth, hoistable, will_hoist);
            }
            if let Some(update) = &for_stmt.update {
                scan_expr_for_hoistable_refs(update, fn_depth, hoistable, will_hoist);
            }
            let body = convert_statement(&for_stmt.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::ForOfStatement(for_of) => {
            scan_expr_for_hoistable_refs(&for_of.right, fn_depth, hoistable, will_hoist);
            let body = convert_statement(&for_of.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::ForInStatement(for_in) => {
            scan_expr_for_hoistable_refs(&for_in.right, fn_depth, hoistable, will_hoist);
            let body = convert_statement(&for_in.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::ThrowStatement(throw) => {
            scan_expr_for_hoistable_refs(&throw.argument, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::SwitchStatement(switch) => {
            scan_expr_for_hoistable_refs(&switch.discriminant, fn_depth, hoistable, will_hoist);
            for case in &switch.cases {
                if let Some(test) = &case.test {
                    scan_expr_for_hoistable_refs(test, fn_depth, hoistable, will_hoist);
                }
                for s in &case.consequent {
                    let s = convert_statement(s);
                    scan_for_hoistable_refs(&s, fn_depth, hoistable, will_hoist);
                }
            }
        }
        LowerableStatement::BlockStatement(block) => {
            for s in &block.body {
                let s = convert_statement(s);
                scan_for_hoistable_refs(&s, fn_depth, hoistable, will_hoist);
            }
        }
        LowerableStatement::LabeledStatement(labeled) => {
            let body = convert_statement(&labeled.body);
            scan_for_hoistable_refs(&body, fn_depth, hoistable, will_hoist);
        }
        LowerableStatement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                let s = convert_statement(s);
                scan_for_hoistable_refs(&s, fn_depth, hoistable, will_hoist);
            }
            if let Some(handler) = &try_stmt.handler {
                for s in &handler.body.body {
                    let s = convert_statement(s);
                    scan_for_hoistable_refs(&s, fn_depth, hoistable, will_hoist);
                }
            }
        }
        LowerableStatement::FunctionDeclaration(func) => {
            // Function declarations enter an inner function scope.
            // Filter out names bound in the function (params + local decls).
            if let Some(body) = &func.body {
                let filtered =
                    filter_hoistable_for_inner_function(hoistable, &func.params, &body.statements);
                for s in &body.statements {
                    let s = convert_statement(s);
                    scan_for_hoistable_refs(&s, fn_depth + 1, &filtered, will_hoist);
                }
            }
        }
        LowerableStatement::BreakStatement(_)
        | LowerableStatement::ContinueStatement(_)
        | LowerableStatement::DebuggerStatement
        | LowerableStatement::EmptyStatement => {}
    }
}

/// Scan an expression for references to hoistable identifiers.
fn scan_expr_for_hoistable_refs(
    expr: &ast::Expression<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match expr {
        ast::Expression::Identifier(ident) => {
            // Hoist if we're inside an inner function (fn_depth > 0), or if the
            // binding is a function declaration (equivalent to Babel's
            // `binding.kind === 'hoisted'`). Function declarations in JavaScript
            // are hoisted to the top of their enclosing scope, so references
            // before the declaration at the same scope level need hoisting too.
            // See CodegenReactiveFunction.ts BuildHIR line 430:
            //   `(fnDepth > 0 || binding.kind === 'hoisted')`
            if let Some(binding) = hoistable.get(ident.name.as_str())
                && (fn_depth > 0 || binding.is_function_decl)
                && !will_hoist.contains(&ident.name.to_string())
            {
                will_hoist.push(ident.name.to_string());
            }
        }
        ast::Expression::AssignmentExpression(assign) => {
            scan_expr_for_hoistable_refs(&assign.right, fn_depth, hoistable, will_hoist);
            scan_assignment_target_for_hoistable_refs(
                &assign.left,
                fn_depth,
                hoistable,
                will_hoist,
            );
        }
        ast::Expression::UpdateExpression(update) => {
            scan_simple_assignment_target_for_hoistable_refs(
                &update.argument,
                fn_depth,
                hoistable,
                will_hoist,
            );
        }
        ast::Expression::BinaryExpression(bin) => {
            scan_expr_for_hoistable_refs(&bin.left, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&bin.right, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::LogicalExpression(logical) => {
            scan_expr_for_hoistable_refs(&logical.left, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&logical.right, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::UnaryExpression(unary) => {
            scan_expr_for_hoistable_refs(&unary.argument, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::ConditionalExpression(cond) => {
            scan_expr_for_hoistable_refs(&cond.test, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&cond.consequent, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&cond.alternate, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::CallExpression(call) => {
            scan_expr_for_hoistable_refs(&call.callee, fn_depth, hoistable, will_hoist);
            for arg in &call.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        scan_expr_for_hoistable_refs(
                            &spread.argument,
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                    _ => {
                        scan_expr_for_hoistable_refs(
                            arg.to_expression(),
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
        }
        ast::Expression::NewExpression(new_expr) => {
            scan_expr_for_hoistable_refs(&new_expr.callee, fn_depth, hoistable, will_hoist);
            for arg in &new_expr.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        scan_expr_for_hoistable_refs(
                            &spread.argument,
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                    _ => {
                        scan_expr_for_hoistable_refs(
                            arg.to_expression(),
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
        }
        ast::Expression::StaticMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::ComputedMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&member.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::PrivateFieldExpression(pf) => {
            scan_expr_for_hoistable_refs(&pf.object, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    ast::ArrayExpressionElement::SpreadElement(spread) => {
                        scan_expr_for_hoistable_refs(
                            &spread.argument,
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                    ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        scan_expr_for_hoistable_refs(
                            elem.to_expression(),
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
        }
        ast::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        scan_expr_for_hoistable_refs(
                            &spread.argument,
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                    ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        if prop.computed {
                            scan_expr_for_hoistable_refs(
                                prop.key.to_expression(),
                                fn_depth,
                                hoistable,
                                will_hoist,
                            );
                        }
                        scan_expr_for_hoistable_refs(&prop.value, fn_depth, hoistable, will_hoist);
                    }
                }
            }
        }
        ast::Expression::TemplateLiteral(tpl) => {
            for sub in &tpl.expressions {
                scan_expr_for_hoistable_refs(sub, fn_depth, hoistable, will_hoist);
            }
        }
        ast::Expression::TaggedTemplateExpression(tagged) => {
            scan_expr_for_hoistable_refs(&tagged.tag, fn_depth, hoistable, will_hoist);
            for sub in &tagged.quasi.expressions {
                scan_expr_for_hoistable_refs(sub, fn_depth, hoistable, will_hoist);
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            for sub in &seq.expressions {
                scan_expr_for_hoistable_refs(sub, fn_depth, hoistable, will_hoist);
            }
        }
        ast::Expression::AwaitExpression(await_expr) => {
            scan_expr_for_hoistable_refs(&await_expr.argument, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                scan_expr_for_hoistable_refs(arg, fn_depth, hoistable, will_hoist);
            }
        }
        ast::Expression::ArrowFunctionExpression(arrow) => {
            // Enter inner function scope. Filter out names bound in the arrow
            // function (params + local declarations) so they don't shadow outer
            // hoistable bindings. This matches the TS reference which uses
            // `id.scope.getBinding(id.node.name)` for proper scope resolution.
            let filtered = filter_hoistable_for_inner_function(
                hoistable,
                &arrow.params,
                &arrow.body.statements,
            );
            for s in &arrow.body.statements {
                let s = convert_statement(s);
                scan_for_hoistable_refs(&s, fn_depth + 1, &filtered, will_hoist);
            }
        }
        ast::Expression::FunctionExpression(func) => {
            // Enter inner function scope with filtered hoistable map.
            if let Some(body) = &func.body {
                let filtered =
                    filter_hoistable_for_inner_function(hoistable, &func.params, &body.statements);
                // Also exclude the function's own name (named function expressions)
                let mut filtered = filtered;
                if let Some(id) = &func.id {
                    filtered.remove(id.name.as_str());
                }
                for s in &body.statements {
                    let s = convert_statement(s);
                    scan_for_hoistable_refs(&s, fn_depth + 1, &filtered, will_hoist);
                }
            }
        }
        ast::Expression::ChainExpression(chain) => match &chain.expression {
            ast::ChainElement::CallExpression(call) => {
                scan_expr_for_hoistable_refs(&call.callee, fn_depth, hoistable, will_hoist);
                for arg in &call.arguments {
                    match arg {
                        ast::Argument::SpreadElement(spread) => {
                            scan_expr_for_hoistable_refs(
                                &spread.argument,
                                fn_depth,
                                hoistable,
                                will_hoist,
                            );
                        }
                        _ => {
                            scan_expr_for_hoistable_refs(
                                arg.to_expression(),
                                fn_depth,
                                hoistable,
                                will_hoist,
                            );
                        }
                    }
                }
            }
            ast::ChainElement::StaticMemberExpression(member) => {
                scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
            }
            ast::ChainElement::ComputedMemberExpression(member) => {
                scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
                scan_expr_for_hoistable_refs(&member.expression, fn_depth, hoistable, will_hoist);
            }
            ast::ChainElement::TSNonNullExpression(ts_nn) => {
                scan_expr_for_hoistable_refs(&ts_nn.expression, fn_depth, hoistable, will_hoist);
            }
            ast::ChainElement::PrivateFieldExpression(pf) => {
                scan_expr_for_hoistable_refs(&pf.object, fn_depth, hoistable, will_hoist);
            }
        },
        ast::Expression::ParenthesizedExpression(paren) => {
            scan_expr_for_hoistable_refs(&paren.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::TSAsExpression(ts_as) => {
            scan_expr_for_hoistable_refs(&ts_as.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::TSSatisfiesExpression(ts_sat) => {
            scan_expr_for_hoistable_refs(&ts_sat.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::TSNonNullExpression(ts_nn) => {
            scan_expr_for_hoistable_refs(&ts_nn.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::TSTypeAssertion(ts_ta) => {
            scan_expr_for_hoistable_refs(&ts_ta.expression, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::JSXElement(jsx) => {
            scan_jsx_element_for_hoistable_refs(jsx, fn_depth, hoistable, will_hoist);
        }
        ast::Expression::JSXFragment(frag) => {
            for child in &frag.children {
                scan_jsx_child_for_hoistable_refs(child, fn_depth, hoistable, will_hoist);
            }
        }
        // Literals and other leaf expressions
        _ => {}
    }
}

/// Scan an assignment target for hoistable refs.
fn scan_assignment_target_for_hoistable_refs(
    target: &ast::AssignmentTarget<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            if let Some(binding) = hoistable.get(ident.name.as_str())
                && (fn_depth > 0 || binding.is_function_decl)
                && !will_hoist.contains(&ident.name.to_string())
            {
                will_hoist.push(ident.name.to_string());
            }
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&member.expression, fn_depth, hoistable, will_hoist);
        }
        ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
            for elem in arr.elements.iter().flatten() {
                scan_assignment_target_maybe_default_for_hoistable_refs(
                    elem, fn_depth, hoistable, will_hoist,
                );
            }
        }
        ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                        if let Some(binding) = hoistable.get(ident.binding.name.as_str())
                            && (fn_depth > 0 || binding.is_function_decl)
                            && !will_hoist.contains(&ident.binding.name.to_string())
                        {
                            will_hoist.push(ident.binding.name.to_string());
                        }
                    }
                    ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                        scan_assignment_target_maybe_default_for_hoistable_refs(
                            &prop.binding,
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// Scan an assignment target (maybe with default) for hoistable refs.
fn scan_assignment_target_maybe_default_for_hoistable_refs(
    target: &ast::AssignmentTargetMaybeDefault<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match target {
        ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            scan_expr_for_hoistable_refs(&with_default.init, fn_depth, hoistable, will_hoist);
            scan_assignment_target_for_hoistable_refs(
                &with_default.binding,
                fn_depth,
                hoistable,
                will_hoist,
            );
        }
        _ => {
            scan_assignment_target_for_hoistable_refs(
                target.to_assignment_target(),
                fn_depth,
                hoistable,
                will_hoist,
            );
        }
    }
}

/// Scan a simple assignment target for hoistable refs (for update expressions).
fn scan_simple_assignment_target_for_hoistable_refs(
    target: &ast::SimpleAssignmentTarget<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match target {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
            if let Some(binding) = hoistable.get(ident.name.as_str())
                && (fn_depth > 0 || binding.is_function_decl)
                && !will_hoist.contains(&ident.name.to_string())
            {
                will_hoist.push(ident.name.to_string());
            }
        }
        ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
        }
        ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            scan_expr_for_hoistable_refs(&member.object, fn_depth, hoistable, will_hoist);
            scan_expr_for_hoistable_refs(&member.expression, fn_depth, hoistable, will_hoist);
        }
        _ => {}
    }
}

/// Scan a JSX element for hoistable refs.
fn scan_jsx_element_for_hoistable_refs(
    element: &ast::JSXElement<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    // Scan opening element name and attributes
    if let ast::JSXElementName::IdentifierReference(ident) = &element.opening_element.name
        && fn_depth > 0
        && hoistable.contains_key(ident.name.as_str())
        && !will_hoist.contains(&ident.name.to_string())
    {
        will_hoist.push(ident.name.to_string());
    }
    for attr in &element.opening_element.attributes {
        match attr {
            ast::JSXAttributeItem::Attribute(attr) => {
                if let Some(value) = &attr.value
                    && let ast::JSXAttributeValue::ExpressionContainer(container) = value
                {
                    if let ast::JSXExpression::EmptyExpression(_) = &container.expression {
                        // skip
                    } else {
                        scan_expr_for_hoistable_refs(
                            container.expression.to_expression(),
                            fn_depth,
                            hoistable,
                            will_hoist,
                        );
                    }
                }
            }
            ast::JSXAttributeItem::SpreadAttribute(spread) => {
                scan_expr_for_hoistable_refs(&spread.argument, fn_depth, hoistable, will_hoist);
            }
        }
    }
    for child in &element.children {
        scan_jsx_child_for_hoistable_refs(child, fn_depth, hoistable, will_hoist);
    }
}

/// Scan a JSX child for hoistable refs.
fn scan_jsx_child_for_hoistable_refs(
    child: &ast::JSXChild<'_>,
    fn_depth: u32,
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    will_hoist: &mut Vec<String>,
) {
    match child {
        ast::JSXChild::ExpressionContainer(container) => {
            if let ast::JSXExpression::EmptyExpression(_) = &container.expression {
                // skip
            } else {
                scan_expr_for_hoistable_refs(
                    container.expression.to_expression(),
                    fn_depth,
                    hoistable,
                    will_hoist,
                );
            }
        }
        ast::JSXChild::Element(element) => {
            scan_jsx_element_for_hoistable_refs(element, fn_depth, hoistable, will_hoist);
        }
        ast::JSXChild::Spread(spread) => {
            scan_expr_for_hoistable_refs(&spread.expression, fn_depth, hoistable, will_hoist);
        }
        _ => {}
    }
}

/// Collect all binding names from a function's scope (parameters + local declarations).
///
/// This is used during hoisting analysis to determine which names are shadowed
/// inside an inner function, so that references to those names inside the function
/// are not incorrectly treated as references to outer hoistable bindings.
///
/// The TS reference uses Babel's `id.scope.getBinding(id.node.name)` which resolves
/// each identifier to its actual binding. This function provides a lightweight
/// approximation by collecting all names that would be bound within the function scope.
fn collect_function_scope_binding_names(
    params: &ast::FormalParameters<'_>,
    body_stmts: &[ast::Statement<'_>],
) -> rustc_hash::FxHashSet<String> {
    let mut names: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();

    // Collect parameter names
    for param in &params.items {
        collect_binding_pattern_names(&param.pattern, &mut names);
    }
    if let Some(rest) = &params.rest {
        collect_binding_pattern_names(&rest.rest.argument, &mut names);
    }

    // Collect local declaration names from the function body
    collect_statement_binding_names_recursive(body_stmts, &mut names);

    names
}

/// Recursively collect all `let`/`const`/`var`/function declaration names from statements.
/// This traverses into blocks and other compound statements to find all declarations.
fn collect_statement_binding_names_recursive(
    stmts: &[ast::Statement<'_>],
    names: &mut rustc_hash::FxHashSet<String>,
) {
    for stmt in stmts {
        match stmt {
            ast::Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    collect_binding_pattern_names(&declarator.id, names);
                }
            }
            ast::Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    names.insert(id.name.to_string());
                }
            }
            ast::Statement::BlockStatement(block) => {
                collect_statement_binding_names_recursive(&block.body, names);
            }
            ast::Statement::IfStatement(if_stmt) => {
                collect_statement_binding_names_from_single(&if_stmt.consequent, names);
                if let Some(alt) = &if_stmt.alternate {
                    collect_statement_binding_names_from_single(alt, names);
                }
            }
            ast::Statement::WhileStatement(while_stmt) => {
                collect_statement_binding_names_from_single(&while_stmt.body, names);
            }
            ast::Statement::DoWhileStatement(do_while) => {
                collect_statement_binding_names_from_single(&do_while.body, names);
            }
            ast::Statement::ForStatement(for_stmt) => {
                if let Some(ast::ForStatementInit::VariableDeclaration(decl)) = &for_stmt.init {
                    for declarator in &decl.declarations {
                        collect_binding_pattern_names(&declarator.id, names);
                    }
                }
                collect_statement_binding_names_from_single(&for_stmt.body, names);
            }
            ast::Statement::ForOfStatement(for_of) => {
                if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left {
                    for declarator in &decl.declarations {
                        collect_binding_pattern_names(&declarator.id, names);
                    }
                }
                collect_statement_binding_names_from_single(&for_of.body, names);
            }
            ast::Statement::ForInStatement(for_in) => {
                if let ast::ForStatementLeft::VariableDeclaration(decl) = &for_in.left {
                    for declarator in &decl.declarations {
                        collect_binding_pattern_names(&declarator.id, names);
                    }
                }
                collect_statement_binding_names_from_single(&for_in.body, names);
            }
            ast::Statement::SwitchStatement(switch) => {
                for case in &switch.cases {
                    collect_statement_binding_names_recursive(&case.consequent, names);
                }
            }
            ast::Statement::TryStatement(try_stmt) => {
                collect_statement_binding_names_recursive(&try_stmt.block.body, names);
                if let Some(handler) = &try_stmt.handler {
                    if let Some(param) = &handler.param {
                        collect_binding_pattern_names(&param.pattern, names);
                    }
                    collect_statement_binding_names_recursive(&handler.body.body, names);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    collect_statement_binding_names_recursive(&finalizer.body, names);
                }
            }
            ast::Statement::LabeledStatement(labeled) => {
                collect_statement_binding_names_from_single(&labeled.body, names);
            }
            _ => {}
        }
    }
}

/// Helper for single statements (not slices).
fn collect_statement_binding_names_from_single(
    stmt: &ast::Statement<'_>,
    names: &mut rustc_hash::FxHashSet<String>,
) {
    collect_statement_binding_names_recursive(std::slice::from_ref(stmt), names);
}

/// Create a filtered hoistable map that excludes names bound within an inner function.
///
/// When scanning an inner function for hoistable references, any names that are
/// declared within that function (as parameters or local variables) shadow the
/// outer hoistable bindings and should not be treated as references to the outer scope.
fn filter_hoistable_for_inner_function(
    hoistable: &rustc_hash::FxHashMap<String, HoistableBinding>,
    params: &ast::FormalParameters<'_>,
    body_stmts: &[ast::Statement<'_>],
) -> rustc_hash::FxHashMap<String, HoistableBinding> {
    let inner_names = collect_function_scope_binding_names(params, body_stmts);
    if inner_names.is_empty() {
        return hoistable.clone();
    }
    hoistable
        .iter()
        .filter(|(name, _)| !inner_names.contains(name.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
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
    BreakStatement(Option<&'a str>),
    ContinueStatement(Option<&'a str>),
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
            // Port of BuildHIR.ts line 272-283: throw inside try/catch is unsupported
            if builder.resolve_throw_handler().is_some() {
                builder.errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                    crate::compiler_error::CompilerErrorDetailOptions {
                        category: crate::compiler_error::ErrorCategory::Todo,
                        reason:
                            "(BuildHIR::lowerStatement) Support ThrowStatement inside of try/catch"
                                .to_string(),
                        description: None,
                        loc: Some(loc),
                        suggestions: None,
                    },
                ));
            }
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
                    effects: None,
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

            // Block for the consequent (if the test is truthy).
            // Push a binding scope so that const/let declarations inside the
            // consequent block don't pollute the outer scope's binding map.
            let consequent_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.push_binding_scope();
                let consequent = convert_statement(&if_stmt.consequent);
                let _ = lower_statement_with_label(builder, &consequent, None); // errors accumulated in builder
                builder.pop_binding_scope();
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    loc: span_to_loc(if_stmt.consequent.span()),
                })
            });

            // Block for the alternate (if the test is not truthy).
            let alternate_block = if let Some(alternate) = &if_stmt.alternate {
                builder.enter(BlockKind::Block, |builder, _block_id| {
                    builder.push_binding_scope();
                    let alt = convert_statement(alternate);
                    let _ = lower_statement_with_label(builder, &alt, None);
                    builder.pop_binding_scope();
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
            // Push a binding scope for the block's const/let declarations.
            builder.push_binding_scope();
            lower_block_statement(builder, &stmts)?;
            builder.pop_binding_scope();
        }

        // =====================================================================
        // BreakStatement
        // =====================================================================
        LowerableStatement::BreakStatement(label) => {
            let target = builder.lookup_break(*label)?;
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
        LowerableStatement::ContinueStatement(label) => {
            let target = builder.lookup_continue(*label)?;
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
                    if let ast::ForStatementInit::VariableDeclaration(decl) = init {
                        let init_stmt = LowerableStatement::VariableDeclaration(decl);
                        let _ = lower_statement_with_label(builder, &init_stmt, None);
                    } else {
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
                    let _ = lower_expression(builder, &update_expr);
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
                        builder.push_binding_scope();
                        let body = convert_statement(&for_stmt.body);
                        let _ = lower_statement_with_label(builder, &body, None);
                        builder.pop_binding_scope();
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: continue_target,
                    variant: GotoVariant::Continue,
                    loc: span_to_loc(for_stmt.body.span()),
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
                        builder.push_binding_scope();
                        let body = convert_statement(&while_stmt.body);
                        let _ = lower_statement_with_label(builder, &body, None);
                        builder.pop_binding_scope();
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: conditional_id,
                    variant: GotoVariant::Continue,
                    loc: span_to_loc(while_stmt.body.span()),
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
                        builder.push_binding_scope();
                        let body = convert_statement(&do_while.body);
                        let _ = lower_statement_with_label(builder, &body, None);
                        builder.pop_binding_scope();
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: conditional_id,
                    variant: GotoVariant::Continue,
                    loc: span_to_loc(do_while.body.span()),
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

            // Port of BuildHIR.ts line 1011-1018: for-await-of is unsupported
            if for_of.r#await {
                builder.errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                    crate::compiler_error::CompilerErrorDetailOptions {
                        category: crate::compiler_error::ErrorCategory::Todo,
                        reason: "(BuildHIR::lowerStatement) Handle for-await loops".to_string(),
                        description: None,
                        loc: Some(loc),
                        suggestions: None,
                    },
                ));
                return Ok(());
            }

            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;
            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;

            // Pre-declare any destructuring bindings from the for-of left side so
            // that references to those variables inside the loop body are resolved
            // as local bindings rather than globals. This is needed because the loop
            // body is lowered BEFORE the test block where the actual Destructure
            // instruction is emitted (same ordering as the TS reference compiler,
            // which relies on Babel's pre-built scope for this).
            let left_loc = span_to_loc(for_of.left.span());
            pre_declare_for_loop_left_bindings(builder, &for_of.left, left_loc);

            // Loop body
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&for_of.body);
                        let _ = lower_statement_with_label(builder, &body, None);
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: init_block_id,
                    variant: GotoVariant::Continue,
                    loc: span_to_loc(for_of.body.span()),
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
            // (left_loc is already declared above for pre_declare_for_loop_left_bindings)
            let advance_iterator = create_temporary_place(builder.environment_mut(), left_loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: advance_iterator.clone(),
                value: InstructionValue::IteratorNext(IteratorNext {
                    iterator,
                    collection: value,
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

            // Pre-declare any destructuring bindings from the for-in left side so
            // that references inside the loop body resolve as local bindings.
            let left_loc_pre = span_to_loc(for_in.left.span());
            pre_declare_for_loop_left_bindings(builder, &for_in.left, left_loc_pre);

            // Loop body
            let loop_block = builder.enter(BlockKind::Block, |builder, _block_id| {
                builder.enter_loop(
                    label.map(String::from),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        let body = convert_statement(&for_in.body);
                        let _ = lower_statement_with_label(builder, &body, None);
                    },
                );
                Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: init_block_id,
                    variant: GotoVariant::Continue,
                    loc: span_to_loc(for_in.body.span()),
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
                value: InstructionValue::NextPropertyOf(NextPropertyOf { value, loc: left_loc }),
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
                            let _ = lower_statement_with_label(builder, &body, None);
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
                            let _ = lower_statement_with_label(builder, &stmt, None);
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
            // Port of BuildHIR.ts line 824-831: `var` declarations are unsupported
            if var_decl.kind == ast::VariableDeclarationKind::Var {
                builder.errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                    crate::compiler_error::CompilerErrorDetailOptions {
                        category: crate::compiler_error::ErrorCategory::Todo,
                        reason:
                            "(BuildHIR::lowerStatement) Handle var kinds in VariableDeclaration"
                                .to_string(),
                        description: None,
                        loc: Some(loc),
                        suggestions: None,
                    },
                ));
                return Ok(());
            }
            let kind = match var_decl.kind {
                ast::VariableDeclarationKind::Let | ast::VariableDeclarationKind::Var => {
                    InstructionKind::Let
                }
                ast::VariableDeclarationKind::Const
                | ast::VariableDeclarationKind::Using
                | ast::VariableDeclarationKind::AwaitUsing => InstructionKind::Const,
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
                    // Pre-declare all bindings in the pattern BEFORE lowering the
                    // initializer, so that self-referencing initializers like
                    // `const x = identity(x)` emit LoadLocal (not LoadGlobal).
                    // The TS reference doesn't need this because Babel's scope
                    // analysis pre-resolves all bindings before HIR lowering.
                    // Without this, SSA can't detect use-before-definition errors.
                    {
                        let pre_loc = span_to_loc(declaration.span);
                        let mut names = rustc_hash::FxHashSet::default();
                        collect_binding_pattern_names(&declaration.id, &mut names);
                        for name in &names {
                            builder.pre_declare_binding(name, pre_loc);
                        }
                    }

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
                            let decl_place = builder.declare_binding(
                                &ident.name,
                                binding_kind,
                                decl_loc,
                                ident.span,
                            );

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

            // Register the function name as a binding.
            // If the name was hoisted (via DeclareContext), emit StoreContext
            // instead of StoreLocal to match the TS reference's lowerAssignment
            // behavior (BuildHIR.ts lines 3693-3727).
            //
            // For redeclared function declarations (e.g. `function x(a){...}; function x(){}`),
            // JavaScript semantics dictate that the last declaration wins. In the TS reference,
            // Babel's scope system makes both declarations share the same binding node, so
            // `resolveBinding` returns the same `Identifier` and the second `StoreLocal`
            // naturally overwrites the first. We replicate this by resolving the existing
            // binding when one already exists, rather than creating a new suffixed binding.
            let decl_place = if let Some(id) = &func.id {
                match builder.resolve_identifier(&id.name) {
                    VariableBinding::Identifier { identifier, .. } => {
                        // Binding already exists — reuse it (handles redeclaration).
                        crate::hir::Place {
                            identifier,
                            effect: crate::hir::Effect::Unknown,
                            reactive: false,
                            loc,
                        }
                    }
                    VariableBinding::NonLocal(_) => {
                        // No existing binding — declare a new one.
                        builder.declare_binding(&id.name, BindingKind::Function, loc, id.span)
                    }
                }
            } else {
                create_temporary_place(builder.environment_mut(), loc)
            };
            let is_context =
                func.id.as_ref().is_some_and(|id| builder.is_context_identifier(&id.name));
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            if is_context {
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue,
                    value: InstructionValue::StoreContext(StoreContext {
                        lvalue_kind: InstructionKind::Function,
                        lvalue_place: decl_place,
                        value: fn_place,
                        loc,
                    }),
                    effects: None,
                    loc,
                });
            } else {
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

            // Declare handler binding if present.
            // This creates a promoted temporary place that will hold the caught exception value.
            // Also extract the catch param name (for identifier params) so we can
            // emit a StoreLocal inside the handler block to bind the user-named variable.
            // Using create_promoted_temporary mirrors TS's promoteTemporary(place.identifier)
            // which gives the temp a sequential name like `t0`, `t1`, etc. in output.
            let handler_binding = if let Some(handler) = &try_stmt.handler {
                if let Some(param) = &handler.param {
                    let handler_loc = span_to_loc(param.span());
                    let place = create_promoted_temporary(builder, handler_loc);

                    // Emit DeclareLocal for catch binding (the promoted temporary that holds the exception)
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

            // Extract catch param info before entering the handler block so we can
            // bind the user-named identifier inside the handler.
            // This mirrors TS BuildHIR which calls lowerAssignment(InstructionKind.Catch, ...)
            // inside the handler block to emit StoreLocal { lvalue: { e, Catch }, value: handler_temp }.
            let catch_param_info: Option<(&str, SourceLocation, Span)> =
                try_stmt.handler.as_ref().and_then(|handler| {
                    handler.param.as_ref().and_then(|param| match &param.pattern {
                        ast::BindingPattern::BindingIdentifier(ident) => {
                            Some((ident.name.as_str(), span_to_loc(ident.span), ident.span))
                        }
                        // Destructured catch params (e.g. `catch ({status})`) are not
                        // supported by the TS compiler. The TS BuildHIR tries to resolve
                        // each destructured identifier as a binding but fails, emitting:
                        //   "(BuildHIR::lowerAssignment) Could not find binding for declaration."
                        // We match that invariant error here.
                        ast::BindingPattern::ObjectPattern(pat) => {
                            let param_loc = span_to_loc(pat.span);
                            builder.errors.merge(CompilerError::invariant(
                                "(BuildHIR::lowerAssignment) Could not find binding for declaration.",
                                None,
                                param_loc,
                            ));
                            None
                        }
                        ast::BindingPattern::ArrayPattern(pat) => {
                            let param_loc = span_to_loc(pat.span);
                            builder.errors.merge(CompilerError::invariant(
                                "(BuildHIR::lowerAssignment) Could not find binding for declaration.",
                                None,
                                param_loc,
                            ));
                            None
                        }
                        ast::BindingPattern::AssignmentPattern(pat) => {
                            let param_loc = span_to_loc(pat.span);
                            builder.errors.merge(CompilerError::invariant(
                                "(BuildHIR::lowerAssignment) Could not find binding for declaration.",
                                None,
                                param_loc,
                            ));
                            None
                        }
                    })
                });

            // Handler block (catch block)
            let handler = builder.enter(BlockKind::Catch, |builder, _block_id| {
                // Bind the catch param identifier to the handler temp (the exception value).
                // e.g. for `catch (e) { ... }`, emit:
                //   StoreLocal { lvalue: { place: e, kind: Catch }, value: handler_temp }
                // This is the equivalent of TS lowerAssignment(InstructionKind.Catch, ...) call.
                if let (Some(handler_temp), Some((param_name, param_loc, param_span))) =
                    (&handler_binding, catch_param_info)
                {
                    // The catch param is always stored with StoreLocal, matching the
                    // TS compiler. If the catch param is also a context identifier
                    // (captured by an inner closure), this creates an inconsistency:
                    // the outer scope stores it as local, but the inner function
                    // loads it as context. The TS compiler hits an invariant for this
                    // in ValidateContextVariableLValues. We detect it here and emit
                    // the same error.
                    if builder.is_context_identifier(param_name) {
                        builder.errors.merge(CompilerError::invariant(
                            "Expected all references to a variable to be consistently local or context references",
                            None,
                            param_loc,
                        ));
                    }

                    let e_place = builder.declare_binding(
                        param_name,
                        BindingKind::Let,
                        param_loc,
                        param_span,
                    );
                    let lvalue = create_temporary_place(builder.environment_mut(), param_loc);
                    builder.push(Instruction {
                        id: InstructionId(0),
                        lvalue,
                        value: InstructionValue::StoreLocal(StoreLocal {
                            lvalue: LValue { place: e_place, kind: InstructionKind::Catch },
                            value: handler_temp.clone(),
                            loc: param_loc,
                        }),
                        effects: None,
                        loc: param_loc,
                    });
                }

                if let Some(catch_clause) = &try_stmt.handler {
                    // Lower catch body.
                    // The catch parameter needs to be included in the hoistable
                    // bindings so that references to it from inner functions
                    // trigger DeclareContext emission (matching TS behavior where
                    // Babel's scope.bindings includes catch params automatically).
                    let stmts: Vec<_> =
                        catch_clause.body.body.iter().map(convert_statement).collect();
                    let extra_hoistable = catch_param_info.map(|(name, _, _)| {
                        (
                            name,
                            HoistableBinding {
                                decl_kind: ast::VariableDeclarationKind::Let,
                                is_function_decl: false,
                            },
                        )
                    });
                    let _ = lower_block_statement_with_extra_hoistable(
                        builder,
                        &stmts,
                        extra_hoistable,
                    );
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
                    let _ = lower_block_statement(builder, &stmts);
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

/// Pre-declare all bindings from a for-of/for-in loop's left-hand side pattern.
///
/// This must be called BEFORE entering the loop body closure so that
/// references to the loop variables (e.g. `v` in `for (const {v} of items)`)
/// inside the loop body are resolved as local bindings rather than globals.
///
/// For destructuring patterns like `{v}` or `[a, b]`, each named binding is
/// pre-declared. For simple identifiers like `x`, this is a no-op (the
/// identifier name is already in scope_binding_names from the function-level
/// scan, or the binding will be created normally).
/// Pre-declare all binding names from the left-hand side of a for-of/for-in
/// loop so that references in the loop body (which is lowered before the test
/// block where the actual StoreLocal is emitted) resolve as local bindings
/// rather than globals. Without this, `for (const x of items)` would cause
/// `x` in the body to emit LoadGlobal instead of LoadLocal, breaking mutation
/// propagation through the aliasing graph.
///
/// The TS reference compiler does not need this because Babel's scope analysis
/// pre-resolves all bindings before HIR lowering begins.
fn pre_declare_for_loop_left_bindings(
    builder: &mut HirBuilder,
    left: &ast::ForStatementLeft<'_>,
    loc: SourceLocation,
) {
    if let ast::ForStatementLeft::VariableDeclaration(decl) = left
        && let Some(declarator) = decl.declarations.first()
    {
        let mut names = rustc_hash::FxHashSet::default();
        collect_binding_pattern_names(&declarator.id, &mut names);
        for name in &names {
            builder.pre_declare_binding(name, loc);
        }
    }
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
            ast::VariableDeclarationKind::Const => InstructionKind::Const,
            _ => InstructionKind::Let,
        },
        // Assignment target (e.g. `for (x of ...)`) uses Reassign
        _ => InstructionKind::Reassign,
    };

    // Check if this is a destructuring pattern (ObjectPattern or ArrayPattern).
    // If so, we need to:
    // 1. Assign the iterator value to a temporary via Destructure
    // 2. Return that temporary as the test place.
    if let ast::ForStatementLeft::VariableDeclaration(decl) = left
        && let Some(declarator) = decl.declarations.first()
    {
        match &declarator.id {
            ast::BindingPattern::ObjectPattern(_) | ast::BindingPattern::ArrayPattern(_) => {
                let binding_kind = match decl.kind {
                    ast::VariableDeclarationKind::Const
                    | ast::VariableDeclarationKind::Using
                    | ast::VariableDeclarationKind::AwaitUsing => BindingKind::Const,
                    ast::VariableDeclarationKind::Let | ast::VariableDeclarationKind::Var => {
                        BindingKind::Let
                    }
                };
                // Emit a Destructure instruction to extract pattern bindings
                // from the iterator value.
                let _ = lower_destructuring_declaration(
                    builder,
                    &declarator.id,
                    rhs_value.clone(),
                    kind,
                    binding_kind,
                    loc,
                );
                // Return a temporary that carries the iterator-value test.
                // The branch condition checks whether the iterator had a value,
                // so we still need a temp representing the StoreLocal result.
                let temp_decl = create_temporary_place(builder.environment_mut(), loc);
                let lvalue = create_temporary_place(builder.environment_mut(), loc);
                builder.push(Instruction {
                    id: InstructionId(0),
                    lvalue: lvalue.clone(),
                    value: InstructionValue::StoreLocal(StoreLocal {
                        lvalue: LValue { place: temp_decl, kind },
                        value: rhs_value.clone(),
                        loc,
                    }),
                    effects: None,
                    loc,
                });
                return lvalue;
            }
            _ => {}
        }
    }

    // Extract the variable name from the left-hand side and create a named place
    // instead of an unnamed temporary. This preserves the original variable name
    // (e.g., `x` in `for (const x in obj)`) in the codegen output.
    let decl_place = match left {
        ast::ForStatementLeft::VariableDeclaration(decl) => {
            if let Some(declarator) = decl.declarations.first() {
                if let ast::BindingPattern::BindingIdentifier(ident) = &declarator.id {
                    let binding_kind = match decl.kind {
                        ast::VariableDeclarationKind::Const
                        | ast::VariableDeclarationKind::Using
                        | ast::VariableDeclarationKind::AwaitUsing => BindingKind::Const,
                        ast::VariableDeclarationKind::Let | ast::VariableDeclarationKind::Var => {
                            BindingKind::Let
                        }
                    };
                    builder.declare_binding(&ident.name, binding_kind, loc, ident.span)
                } else {
                    create_temporary_place(builder.environment_mut(), loc)
                }
            } else {
                create_temporary_place(builder.environment_mut(), loc)
            }
        }
        _ => create_temporary_place(builder.environment_mut(), loc),
    };

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
            Ok(lower_value_to_temporary(builder, lower_number(*value, loc), loc))
        }
        LowerableExpression::StringLiteral(value, span) => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(builder, lower_string(value.clone(), loc), loc))
        }
        LowerableExpression::BooleanLiteral(value, span) => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(builder, lower_boolean(*value, loc), loc))
        }
        LowerableExpression::NullLiteral(span) => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(builder, lower_null(loc), loc))
        }
        // Destructuring assignment targets should not appear directly in expression
        // lowering — they are handled by `lower_assignment`. If we reach here,
        // lower them as undefined (the assignment itself is handled elsewhere).
        LowerableExpression::Undefined(span)
        | LowerableExpression::ObjectAssignmentTarget { span, .. }
        | LowerableExpression::ArrayAssignmentTarget { span, .. } => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(builder, lower_undefined(loc), loc))
        }
        LowerableExpression::RegExpLiteral { pattern, flags, span } => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::RegExpLiteral(crate::hir::RegExpLiteral {
                    pattern: pattern.clone(),
                    flags: flags.clone(),
                    loc,
                }),
                loc,
            ))
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
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::TemplateLiteral(crate::hir::TemplateLiteral {
                    subexprs,
                    quasis: quasi_values,
                    loc,
                }),
                loc,
            ))
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
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::ArrayExpression(crate::hir::ArrayExpression {
                    elements: lowered_elements,
                    loc,
                }),
                loc,
            ))
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
                    LowerableObjectProperty::Property {
                        key, value, method, kind, span, ..
                    } => {
                        // Port of BuildHIR.ts line 1563-1570: get/set functions are unsupported
                        if *kind != LowerablePropertyKind::Init && !*method {
                            let kind_str = match kind {
                                LowerablePropertyKind::Get => "get",
                                LowerablePropertyKind::Set => "set",
                                LowerablePropertyKind::Init => unreachable!(),
                            };
                            builder.errors.push_error_detail(
                                crate::compiler_error::CompilerErrorDetail::new(
                                    crate::compiler_error::CompilerErrorDetailOptions {
                                        category: crate::compiler_error::ErrorCategory::Todo,
                                        reason: format!("(BuildHIR::lowerExpression) Handle {kind_str} functions in ObjectExpression"),
                                        description: None,
                                        loc: Some(span_to_loc(*span)),
                                        suggestions: None,
                                    },
                                ),
                            );
                            continue;
                        }
                        let lowered_key = lower_object_property_key(builder, key)?;
                        if *method {
                            // Port of lowerObjectMethod() from BuildHIR.ts lines 1491-1506.
                            // For methods, emit an ObjectMethod instruction instead of
                            // FunctionExpression so codegen can reconstruct the method syntax.
                            let method_loc = span_to_loc(*span);
                            let place = match value {
                                LowerableExpression::FunctionExpression { func, span: fn_span } => {
                                    let fn_loc = span_to_loc(*fn_span);
                                    let lowerable_func = LowerableFunction::Function(func);
                                    let func_instr = lower_function_to_value(
                                        builder,
                                        &lowerable_func,
                                        FunctionExpressionType::FunctionExpression,
                                        fn_loc,
                                    )?;
                                    // Convert FunctionExpression → ObjectMethod
                                    let method_value = match func_instr {
                                        InstructionValue::FunctionExpression(fe) => {
                                            InstructionValue::ObjectMethod(ObjectMethodValue {
                                                loc: method_loc,
                                                lowered_func: fe.lowered_func,
                                            })
                                        }
                                        other => other,
                                    };
                                    lower_value_to_temporary(builder, method_value, method_loc)
                                        .place
                                }
                                _ => {
                                    // Fallback: non-function method (shouldn't happen normally)
                                    lower_expression(builder, value)?.place
                                }
                            };
                            lowered_props.push(ObjectPatternProperty::Property(ObjectProperty {
                                key: lowered_key,
                                property_type: ObjectPropertyType::Method,
                                place,
                            }));
                        } else {
                            let value_result = lower_expression(builder, value)?;
                            lowered_props.push(ObjectPatternProperty::Property(ObjectProperty {
                                key: lowered_key,
                                property_type: ObjectPropertyType::Property,
                                place: value_result.place,
                            }));
                        }
                    }
                }
            }
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::ObjectExpression(crate::hir::ObjectExpression {
                    properties: lowered_props,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // BinaryExpression
        // =====================================================================
        LowerableExpression::BinaryExpression { operator, left, right, span } => {
            let loc = span_to_loc(*span);
            let left_result = lower_expression(builder, left)?;
            let right_result = lower_expression(builder, right)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                    operator: *operator,
                    left: left_result.place,
                    right: right_result.place,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // UnaryExpression
        // =====================================================================
        LowerableExpression::UnaryExpression { operator, argument, span } => {
            let loc = span_to_loc(*span);
            // `delete` requires special handling: decompose the member expression
            // argument into object + property and emit PropertyDelete/ComputedDelete,
            // matching the TS reference (BuildHIR.ts lines 2407-2464).
            if *operator == oxc_syntax::operator::UnaryOperator::Delete {
                match argument.as_ref() {
                    LowerableExpression::PropertyAccess { object, property, .. } => {
                        let obj_result = lower_expression(builder, object)?;
                        return Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::PropertyDelete(crate::hir::PropertyDelete {
                                object: obj_result.place,
                                property: crate::hir::types::PropertyLiteral::String(
                                    property.clone(),
                                ),
                                loc,
                            }),
                            loc,
                        ));
                    }
                    LowerableExpression::ComputedPropertyAccess { object, property, .. } => {
                        let obj_result = lower_expression(builder, object)?;
                        let prop_result = lower_expression(builder, property)?;
                        return Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::ComputedDelete(crate::hir::ComputedDelete {
                                object: obj_result.place,
                                property: prop_result.place,
                                loc,
                            }),
                            loc,
                        ));
                    }
                    _ => {
                        // Non-member delete (e.g. `delete x`) — treat as unsupported
                        // like the TS reference does.
                    }
                }
            }
            let arg_result = lower_expression(builder, argument)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::UnaryExpression(crate::hir::UnaryExpressionValue {
                    operator: *operator,
                    value: arg_result.place,
                    loc,
                }),
                loc,
            ))
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
                let right_result = lower_expression(builder, right).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
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
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            ))
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
                    );

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
                    );

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
                    );

                    // PropertyStore to save the updated value back
                    let new_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::PropertyStore(crate::hir::PropertyStore {
                            object: obj_result.place,
                            property: crate::hir::types::PropertyLiteral::String(property.clone()),
                            value: updated_value.place,
                            loc: member_loc,
                        }),
                        member_loc,
                    );

                    // Return previous value for postfix, new value for prefix
                    let result_place = if *prefix { new_value.place } else { previous_value_place };

                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: result_place, loc }),
                        loc,
                    ))
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
                    );

                    let previous_value_place = current_value.place.clone();

                    // Primitive(1)
                    let one_place = lower_value_to_temporary(
                        builder,
                        InstructionValue::Primitive(PrimitiveValue {
                            value: PrimitiveValueKind::Number(1.0),
                            loc: GENERATED_SOURCE,
                        }),
                        GENERATED_SOURCE,
                    );

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
                    );

                    // ComputedStore to save the updated value back
                    let new_value = lower_value_to_temporary(
                        builder,
                        InstructionValue::ComputedStore(crate::hir::ComputedStore {
                            object: obj_result.place,
                            property: prop_result.place,
                            value: updated_value.place,
                            loc: member_loc,
                        }),
                        member_loc,
                    );

                    let result_place = if *prefix { new_value.place } else { previous_value_place };

                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: result_place, loc }),
                        loc,
                    ))
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
                                Ok(lower_value_to_temporary(
                                    builder,
                                    InstructionValue::PrefixUpdate(crate::hir::PrefixUpdate {
                                        lvalue: lvalue_place,
                                        operation: *operator,
                                        value: arg_result.place,
                                        loc,
                                    }),
                                    loc,
                                ))
                            } else {
                                Ok(lower_value_to_temporary(
                                    builder,
                                    InstructionValue::PostfixUpdate(crate::hir::PostfixUpdate {
                                        lvalue: lvalue_place,
                                        operation: *operator,
                                        value: arg_result.place,
                                        loc,
                                    }),
                                    loc,
                                ))
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
                    "Handle UpdateExpression with complex argument",
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
                    );
                    let args = lower_arguments(builder, arguments)?;
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::MethodCall(crate::hir::MethodCall {
                            receiver: receiver.place,
                            property: property_place.place,
                            args,
                            loc,
                        }),
                        loc,
                    ))
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
                    );
                    let args = lower_arguments(builder, arguments)?;
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::MethodCall(crate::hir::MethodCall {
                            receiver: receiver.place,
                            property: property_place.place,
                            args,
                            loc,
                        }),
                        loc,
                    ))
                }
                _ => {
                    let callee_result = lower_expression(builder, callee)?;
                    let args = lower_arguments(builder, arguments)?;
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::CallExpression(crate::hir::CallExpression {
                            callee: callee_result.place,
                            args,
                            loc,
                        }),
                        loc,
                    ))
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
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::NewExpression(crate::hir::NewExpression {
                    callee: callee_result.place,
                    args,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // PropertyAccess (static member expression)
        // =====================================================================
        LowerableExpression::PropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            let obj_result = lower_expression(builder, object)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: obj_result.place,
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // ComputedPropertyAccess (computed member expression)
        // =====================================================================
        LowerableExpression::ComputedPropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            // Match TypeScript BuildHIR behavior: `x[0]` (numeric literal index)
            // is treated as a PropertyLoad with a numeric property, not a
            // ComputedLoad.  TypeScript's check is:
            //   `!expr.node.computed || expr.node.property.type === 'NumericLiteral'`
            // This ensures numeric-indexed accesses are tracked as simple property
            // dependencies (e.g. in useMemo dep lists), just like named-property
            // accesses.  Using PropertyLiteral::Number also ensures the codegen
            // emits `x[0]` (bracket notation) rather than the invalid `x.0`.
            if let LowerableExpression::NumericLiteral(n, _) = property.as_ref() {
                let obj_result = lower_expression(builder, object)?;
                return Ok(lower_value_to_temporary(
                    builder,
                    InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                        object: obj_result.place,
                        property: crate::hir::types::PropertyLiteral::Number(*n),
                        loc,
                    }),
                    loc,
                ));
            }
            let obj_result = lower_expression(builder, object)?;
            let prop_result = lower_expression(builder, property)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: obj_result.place,
                    property: prop_result.place,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // AwaitExpression
        // =====================================================================
        LowerableExpression::AwaitExpression { argument, span } => {
            let loc = span_to_loc(*span);
            let arg_result = lower_expression(builder, argument)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::Await(crate::hir::AwaitValue { value: arg_result.place, loc }),
                loc,
            ))
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
                    lower_expression(builder, consequent).unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        ExpressionResult {
                            place: create_temporary_place(builder.environment_mut(), loc),
                        }
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
                let alternate_result = lower_expression(builder, alternate).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
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
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            ))
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
                    let result = lower_expression(builder, sub_expr).unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        ExpressionResult {
                            place: create_temporary_place(builder.environment_mut(), loc),
                        }
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

            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                loc,
            ))
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
                if let Some(bin_op) = binary_op {
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
                    );
                    // For compound assignment on a simple identifier (e.g. `i += 1`
                    // or `count += n` where count is a context variable), match the
                    // TypeScript reference compiler behaviour (BuildHIR.ts lines 2056-2083):
                    //   1. Emit StoreLocal/StoreContext via lowerValueToTemporary
                    //   2. Return LoadLocal/LoadContext with the identifier place
                    //
                    // This ensures the assignment expression value (the identifier)
                    // appears as a separate expression statement when used in statement
                    // position (e.g., `count = count + x; count;`).
                    //
                    // For non-identifier left-hand sides (member expressions, globals)
                    // we fall back to lower_assignment which handles those cases generically.
                    if let LowerableExpression::Identifier(name, ident_span) = &**left
                        && let VariableBinding::Identifier { identifier, .. } =
                            builder.resolve_identifier(name)
                    {
                        let ident_loc = span_to_loc(*ident_span);
                        let place = crate::hir::Place {
                            identifier,
                            effect: crate::hir::Effect::Unknown,
                            reactive: false,
                            loc: ident_loc,
                        };
                        if builder.is_context_identifier(name) {
                            // Context variable: emit StoreContext + return LoadContext
                            // (BuildHIR.ts lines 2073-2083)
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreContext(StoreContext {
                                    lvalue_kind: InstructionKind::Reassign,
                                    lvalue_place: place.clone(),
                                    value: binary_result.place,
                                    loc,
                                }),
                                loc,
                            );
                            Ok(lower_value_to_temporary(
                                builder,
                                InstructionValue::LoadContext(LoadContext { place, loc }),
                                loc,
                            ))
                        } else {
                            // Local variable: emit StoreLocal + return LoadLocal
                            // (BuildHIR.ts lines 2060-2071)
                            let store_lvalue =
                                create_temporary_place(builder.environment_mut(), loc);
                            builder.push(Instruction {
                                id: InstructionId(0),
                                lvalue: store_lvalue,
                                value: InstructionValue::StoreLocal(StoreLocal {
                                    lvalue: LValue {
                                        place: place.clone(),
                                        kind: InstructionKind::Reassign,
                                    },
                                    value: binary_result.place,
                                    loc,
                                }),
                                effects: None,
                                loc,
                            });
                            // Return LoadLocal(identifier) — the new value of the variable.
                            Ok(lower_value_to_temporary(
                                builder,
                                InstructionValue::LoadLocal(LoadLocal { place, loc }),
                                loc,
                            ))
                        }
                    } else {
                        // Assign the result back to the left (non-identifier LHS)
                        lower_assignment(builder, left, binary_result.place, loc)
                    }
                } else {
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
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc }),
                        loc,
                    ))
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
            Ok(lower_value_to_temporary(
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
            ))
        }

        // =====================================================================
        // TypeCastExpression (TSAsExpression, TSSatisfiesExpression, TypeCast)
        // Port of BuildHIR.ts lines 2474-2508
        // =====================================================================
        LowerableExpression::TypeCastExpression { expression, annotation_kind, span } => {
            let loc = span_to_loc(*span);
            let value_result = lower_expression(builder, expression)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::TypeCastExpression(crate::hir::TypeCastExpression {
                    value: value_result.place,
                    type_: crate::hir::types::make_type(),
                    annotation_kind: *annotation_kind,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // MetaProperty (e.g., import.meta)
        // Port of BuildHIR.ts lines 2643-2663
        // =====================================================================
        LowerableExpression::MetaProperty { meta, property, span } => {
            let loc = span_to_loc(*span);
            // Port of BuildHIR.ts line 2564-2579: only import.meta is supported
            if meta == "import" && property == "meta" {
                Ok(lower_value_to_temporary(
                    builder,
                    InstructionValue::MetaProperty(crate::hir::MetaProperty {
                        meta: meta.clone(),
                        property: property.clone(),
                        loc,
                    }),
                    loc,
                ))
            } else {
                builder.errors.push_error_detail(
                    crate::compiler_error::CompilerErrorDetail::new(
                        crate::compiler_error::CompilerErrorDetailOptions {
                            category: crate::compiler_error::ErrorCategory::Todo,
                            reason: "(BuildHIR::lowerExpression) Handle MetaProperty expressions other than import.meta".to_string(),
                            description: None,
                            loc: Some(loc),
                            suggestions: None,
                        },
                    ),
                );
                Ok(lower_value_to_temporary(
                    builder,
                    InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc }),
                    loc,
                ))
            }
        }

        // =====================================================================
        // LoadGlobal — explicitly global identifiers
        // =====================================================================
        LowerableExpression::LoadGlobal(name, span) => {
            let loc = span_to_loc(*span);
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::LoadGlobal(crate::hir::LoadGlobal {
                    binding: NonLocalBinding::Global { name: name.clone() },
                    loc,
                }),
                loc,
            ))
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
                        Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadContext(LoadContext { place, loc }),
                            loc,
                        ))
                    } else {
                        Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadLocal(LoadLocal { place, loc }),
                            loc,
                        ))
                    }
                }
                VariableBinding::NonLocal(binding) => {
                    // Port of BuildHIR.ts line 3420-3428: eval is unsupported
                    if matches!(&binding, NonLocalBinding::Global { name: n } if n == "eval") {
                        builder.errors.push_error_detail(
                            CompilerErrorDetail::new(
                                CompilerErrorDetailOptions {
                                    category: ErrorCategory::UnsupportedSyntax,
                                    reason: "The 'eval' function is not supported".to_string(),
                                    description: Some("Eval is an anti-pattern in JavaScript, and the code executed cannot be evaluated by React Compiler".to_string()),
                                    loc: Some(loc),
                                    suggestions: None,
                                },
                            ),
                        );
                    }
                    // Global: emit LoadGlobal
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadGlobal(crate::hir::LoadGlobal { binding, loc }),
                        loc,
                    ))
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

            // =====================================================================
            // FBT duplicate-tag detection
            // Port of BuildHIR.ts lines 2165-2220
            //
            // If this is an fbt/fbs builtin element, scan all children recursively
            // for <fbt:enum>, <fbt:plural>, <fbt:pronoun> and throw a Todo error if
            // any kind appears more than once.
            // =====================================================================
            let is_fbt = matches!(&tag,
                LowerableJsxTag::BuiltIn(name, _) if name == "fbt" || name == "fbs"
            );
            if is_fbt {
                let tag_name = match &tag {
                    LowerableJsxTag::BuiltIn(name, _) => name.as_str(),
                    LowerableJsxTag::Expression(_) => unreachable!(),
                };
                let mut fbt_enum_locs: Vec<SourceLocation> = Vec::new();
                let mut fbt_plural_locs: Vec<SourceLocation> = Vec::new();
                let mut fbt_pronoun_locs: Vec<SourceLocation> = Vec::new();
                collect_fbt_namespaced_children(
                    children,
                    tag_name,
                    &mut fbt_enum_locs,
                    &mut fbt_plural_locs,
                    &mut fbt_pronoun_locs,
                );
                for (kind, locs) in [
                    ("enum", &fbt_enum_locs),
                    ("plural", &fbt_plural_locs),
                    ("pronoun", &fbt_pronoun_locs),
                ] {
                    if locs.len() > 1 {
                        let mut diagnostic = CompilerDiagnostic::create(
                            ErrorCategory::Todo,
                            "Support duplicate fbt tags".to_string(),
                            Some(format!(
                                "Support `<{tag_name}>` tags with multiple `<{tag_name}:{kind}>` values"
                            )),
                            None,
                        );
                        for &loc in locs {
                            diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(loc),
                                message: Some(format!("Multiple `<{tag_name}:{kind}>` tags found")),
                            });
                        }
                        builder.errors.push_diagnostic(diagnostic);
                        return Err(builder.errors.clone());
                    }
                }
            }

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
                                )
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

            Ok(lower_value_to_temporary(
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
            ))
        }

        // =====================================================================
        // JsxFragment — lower children
        // Port of BuildHIR.ts lines 2316-2326
        // =====================================================================
        LowerableExpression::JsxFragment { children, span } => {
            let loc = span_to_loc(*span);
            let lowered_children = lower_jsx_children(builder, children)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::JsxFragment(crate::hir::JsxFragment {
                    children: lowered_children,
                    loc,
                }),
                loc,
            ))
        }

        // =====================================================================
        // OptionalMemberExpression — creates CFG with OptionalTerminal
        // Port of BuildHIR.ts lines 2677-2785 (lowerOptionalMemberExpression)
        // =====================================================================
        LowerableExpression::OptionalMemberExpression { span, .. } => {
            let loc = span_to_loc(*span);
            let result = lower_optional_member_expression(builder, expr, None)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal(LoadLocal { place: result.value, loc }),
                loc,
            ))
        }

        // =====================================================================
        // OptionalCallExpression — creates CFG with OptionalTerminal
        // Port of BuildHIR.ts lines 2787-2947 (lowerOptionalCallExpression)
        // =====================================================================
        LowerableExpression::OptionalCallExpression { span, .. } => {
            let loc = span_to_loc(*span);
            let result_value = lower_optional_call_expression(builder, expr, None)?;
            Ok(lower_value_to_temporary(builder, result_value, loc))
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
            Ok(lower_value_to_temporary(builder, value, loc))
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
            Ok(lower_value_to_temporary(builder, value, loc))
        }

        // Port of BuildHIR.ts default case: unsupported expression types push a Todo error
        LowerableExpression::UnsupportedExpression { kind, span } => {
            let loc = span_to_loc(*span);
            builder.errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                crate::compiler_error::CompilerErrorDetailOptions {
                    category: crate::compiler_error::ErrorCategory::Todo,
                    reason: format!("(BuildHIR::lowerExpression) Handle {kind} expressions"),
                    description: None,
                    loc: Some(loc),
                    suggestions: None,
                },
            ));
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc }),
                loc,
            ))
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
) -> ExpressionResult {
    let lvalue = create_temporary_place(builder.environment_mut(), loc);
    builder.push(Instruction {
        id: InstructionId(0),
        lvalue: lvalue.clone(),
        value,
        effects: None,
        loc,
    });
    ExpressionResult { place: lvalue }
}

/// Lower a list of argument expressions into `CallArg` values.
fn lower_arguments(
    builder: &mut HirBuilder,
    arguments: &[LowerableExpression<'_>],
) -> Result<Vec<crate::hir::CallArg>, CompilerError> {
    let mut args = Vec::new();
    for arg in arguments {
        if let LowerableExpression::SpreadElement { argument, .. } = arg {
            let result = lower_expression(builder, argument)?;
            args.push(crate::hir::CallArg::Spread(SpreadPattern { place: result.place }));
        } else {
            let result = lower_expression(builder, arg)?;
            args.push(crate::hir::CallArg::Place(result.place));
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
                VariableBinding::Identifier { identifier, binding_kind } => {
                    // Port of BuildHIR.ts line 3487-3497: const reassignment error
                    if binding_kind == BindingKind::Const {
                        builder.errors.push_error_detail(CompilerErrorDetail::new(
                            CompilerErrorDetailOptions {
                                category: ErrorCategory::Syntax,
                                reason: "Cannot reassign a `const` variable".to_string(),
                                description: Some(format!("`{name}` is declared as const")),
                                loc: Some(ident_loc),
                                suggestions: None,
                            },
                        ));
                    }

                    let place = crate::hir::Place {
                        identifier,
                        effect: crate::hir::Effect::Unknown,
                        reactive: false,
                        loc: ident_loc,
                    };

                    if builder.is_context_identifier(name) {
                        // Context variable: emit StoreContext, then return LoadLocal
                        // with the temporary (matching TS reference BuildHIR.ts line 3748:
                        // `return {kind: 'LoadLocal', place: temporary, loc: temporary.loc}`).
                        // The TS reference returns LoadLocal (not LoadContext) because the
                        // `temporary` is the StoreContext instruction's lvalue — a regular
                        // temporary that can be read with LoadLocal.
                        let temporary = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext(StoreContext {
                                lvalue_kind: InstructionKind::Reassign,
                                lvalue_place: place,
                                value,
                                loc,
                            }),
                            loc,
                        );
                        Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::LoadLocal(LoadLocal { place: temporary.place, loc }),
                            loc,
                        ))
                    } else {
                        // Local variable: emit StoreLocal
                        Ok(lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal(StoreLocal {
                                lvalue: LValue { place, kind: InstructionKind::Reassign },
                                value,
                                loc,
                            }),
                            loc,
                        ))
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
                    );
                    Ok(lower_value_to_temporary(
                        builder,
                        InstructionValue::LoadLocal(LoadLocal { place: temporary.place, loc }),
                        loc,
                    ))
                }
            }
        }
        LowerableExpression::PropertyAccess { object, property, span: member_span } => {
            let member_loc = span_to_loc(*member_span);
            let obj_result = lower_expression(builder, object)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::PropertyStore(crate::hir::PropertyStore {
                    object: obj_result.place,
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    value,
                    loc: member_loc,
                }),
                loc,
            ))
        }
        LowerableExpression::ComputedPropertyAccess { object, property, span: member_span } => {
            let member_loc = span_to_loc(*member_span);
            let obj_result = lower_expression(builder, object)?;
            let prop_result = lower_expression(builder, property)?;
            Ok(lower_value_to_temporary(
                builder,
                InstructionValue::ComputedStore(crate::hir::ComputedStore {
                    object: obj_result.place,
                    property: prop_result.place,
                    value,
                    loc: member_loc,
                }),
                loc,
            ))
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

/// Recursively collect source locations of `<fbt:enum>`, `<fbt:plural>`, `<fbt:pronoun>`
/// children within an `<fbt>`/`<fbs>` element.
///
/// Port of the `JSXNamespacedName` traversal in BuildHIR.ts lines 2185-2203.
fn collect_fbt_namespaced_children(
    children: &[LowerableJsxChild<'_>],
    tag_name: &str,
    enum_locs: &mut Vec<SourceLocation>,
    plural_locs: &mut Vec<SourceLocation>,
    pronoun_locs: &mut Vec<SourceLocation>,
) {
    for child in children {
        match child {
            LowerableJsxChild::Element(LowerableExpression::JsxElement {
                tag: LowerableJsxTag::BuiltIn(name, span),
                children: inner_children,
                ..
            }) => {
                // Check if this is a namespaced tag like "fbt:enum"
                let prefix = format!("{tag_name}:");
                if let Some(suffix) = name.strip_prefix(&prefix) {
                    let loc = span_to_loc(*span);
                    match suffix {
                        "enum" => enum_locs.push(loc),
                        "plural" => plural_locs.push(loc),
                        "pronoun" => pronoun_locs.push(loc),
                        _ => {}
                    }
                }
                // Recurse into this element's children
                collect_fbt_namespaced_children(
                    inner_children,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
                );
            }
            LowerableJsxChild::Fragment { children: inner_children, .. } => {
                collect_fbt_namespaced_children(
                    inner_children,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
                );
            }
            _ => {}
        }
    }
}

/// Lower a JSX tag to a `JsxTag`.
fn lower_jsx_tag(
    builder: &mut HirBuilder,
    tag: &LowerableJsxTag<'_>,
) -> Result<crate::hir::JsxTag, CompilerError> {
    match tag {
        LowerableJsxTag::BuiltIn(name, span) => {
            Ok(crate::hir::JsxTag::BuiltIn(crate::hir::BuiltinTag {
                name: name.clone(),
                loc: span_to_loc(*span),
            }))
        }
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
            let decoded = decode_jsx_entities(&trimmed);
            let loc = span_to_loc(*span);
            let result = lower_value_to_temporary(
                builder,
                InstructionValue::JsxText(crate::hir::JsxTextValue { value: decoded, loc }),
                loc,
            );
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
            );
            Ok(Some(result.place))
        }
    }
}

/// Trim JSX text content following React's whitespace rules.
///
/// JSX text with only whitespace (including newlines) is removed.
/// Leading/trailing whitespace on each line is collapsed.
fn trim_jsx_text(text: &str) -> String {
    // React/Babel JSX text trimming algorithm (cleanJSXElementLiteralChild):
    //
    // 1. Split by newlines
    // 2. For each line:
    //    - If it's the first line, only trim trailing whitespace
    //    - If it's the last line, only trim leading whitespace
    //    - If it's a middle line, trim both sides
    //    - If a line becomes empty after trimming, it's removed
    // 3. Collapse adjacent whitespace within each surviving line to a single space
    // 4. Join the surviving lines with a single space
    //
    // This preserves significant whitespace like "Status: " (trailing space before
    // an expression container on the same line).

    let lines: Vec<&str> = text.split('\n').collect();
    let num_lines = lines.len();

    if num_lines == 1 {
        // Single line: no newline trimming, just collapse internal whitespace.
        // Preserve leading/trailing spaces since they may be significant
        // (e.g., " {expr}" has a leading space JSX text node).
        let line = lines[0];
        if line.chars().all(char::is_whitespace) {
            // Single-line all-whitespace text is significant (e.g., space between elements).
            // Preserve as a single space, matching React/Babel behavior.
            return " ".to_string();
        }
        let mut result = String::new();
        let mut last_was_whitespace = false;
        for ch in line.chars() {
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
        return result;
    }

    // Multi-line: apply the standard Babel/React trimming
    let mut trimmed_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let processed = if i == 0 {
            // First line: trim trailing whitespace only
            line.trim_end()
        } else if i == num_lines - 1 {
            // Last line: trim leading whitespace only
            line.trim_start()
        } else {
            // Middle lines: trim both sides
            line.trim()
        };

        if processed.is_empty() {
            continue;
        }

        // Collapse internal whitespace to single spaces
        let mut collapsed = String::new();
        let mut last_was_whitespace = false;
        for ch in processed.chars() {
            if ch.is_whitespace() {
                if !last_was_whitespace {
                    collapsed.push(' ');
                }
                last_was_whitespace = true;
            } else {
                collapsed.push(ch);
                last_was_whitespace = false;
            }
        }
        trimmed_lines.push(collapsed);
    }

    trimmed_lines.join(" ")
}

/// Decode HTML/XML entities in JSX text (e.g. `&amp;` → `&`, `&#65;` → `A`, `&#x41;` → `A`).
fn decode_jsx_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '&' {
            result.push(ch);
            continue;
        }
        // Collect entity name up to `;` (max 10 chars to avoid runaway)
        let mut entity = String::new();
        let mut found_semi = false;
        let mut consumed = Vec::new();
        for _ in 0..10 {
            match chars.peek() {
                Some(&';') => {
                    chars.next();
                    found_semi = true;
                    break;
                }
                Some(&c) if c.is_ascii_alphanumeric() || c == '#' => {
                    consumed.push(c);
                    entity.push(c);
                    chars.next();
                }
                _ => break,
            }
        }
        if !found_semi || entity.is_empty() {
            // Not a valid entity, emit literally
            result.push('&');
            for c in consumed {
                result.push(c);
            }
            continue;
        }
        // Try to decode the entity
        if let Some(decoded) = decode_entity(&entity) {
            result.push(decoded);
        } else {
            // Unknown entity, emit literally
            result.push('&');
            result.push_str(&entity);
            result.push(';');
        }
    }
    result
}

fn decode_entity(entity: &str) -> Option<char> {
    if let Some(num_str) = entity.strip_prefix('#') {
        // Numeric entity
        let code = if num_str.starts_with('x') || num_str.starts_with('X') {
            u32::from_str_radix(&num_str[1..], 16).ok()?
        } else {
            num_str.parse::<u32>().ok()?
        };
        char::from_u32(code)
    } else {
        // Named entity — use the XML_ENTITIES map from oxc_syntax
        oxc_syntax::xml_entities::XML_ENTITIES.get(entity).copied()
    }
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
    let alternate = match parent_alternate {
        Some(alt_id) => alt_id,
        None => builder.enter(BlockKind::Value, |builder, _block_id| {
            let temp = lower_value_to_temporary(builder, lower_undefined(loc), loc);
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
                        .unwrap_or_else(|err| {
                            builder.errors.merge(err);
                            OptionalMemberResult {
                                object: create_temporary_place(builder.environment_mut(), loc),
                                value: create_temporary_place(builder.environment_mut(), loc),
                            }
                        });
                result.value
            }
            LowerableExpression::OptionalCallExpression { .. } => {
                let value = lower_optional_call_expression(builder, object_expr, Some(alternate))
                    .unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc })
                    });
                lower_value_to_temporary(builder, value, loc).place
            }
            _ => {
                lower_expression(builder, object_expr)
                    .unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        ExpressionResult {
                            place: create_temporary_place(builder.environment_mut(), loc),
                        }
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
                let prop_result = lower_expression(builder, prop_expr).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
                });
                InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: object_place.clone(),
                    property: prop_result.place,
                    loc,
                })
            }
        };
        let temp = lower_value_to_temporary(builder, member_value, loc);
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
    // Determine if this is a method call (callee is a member expression or optional member)
    // and lower the callee in a test block.
    enum CalleeKind {
        CallExpression { callee: crate::hir::Place },
        MethodCall { receiver: crate::hir::Place, property: crate::hir::Place },
    }

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
    let alternate = match parent_alternate {
        Some(alt_id) => alt_id,
        None => builder.enter(BlockKind::Value, |builder, _block_id| {
            let temp = lower_value_to_temporary(builder, lower_undefined(loc), loc);
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

    let mut callee_kind: Option<CalleeKind> = None;
    let test_block = builder.enter(BlockKind::Value, |builder, _block_id| {
        let (kind, test_place) = match callee_expr {
            // Callee is itself an optional call: recursively lower
            LowerableExpression::OptionalCallExpression { .. } => {
                let value = lower_optional_call_expression(builder, callee_expr, Some(alternate))
                    .unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc })
                    });
                let value_place = lower_value_to_temporary(builder, value, loc).place;
                let tp = value_place.clone();
                (CalleeKind::CallExpression { callee: value_place }, tp)
            }
            // Callee is an optional member expression: method call pattern
            LowerableExpression::OptionalMemberExpression { .. } => {
                let result =
                    lower_optional_member_expression(builder, callee_expr, Some(alternate))
                        .unwrap_or_else(|err| {
                            builder.errors.merge(err);
                            OptionalMemberResult {
                                object: create_temporary_place(builder.environment_mut(), loc),
                                value: create_temporary_place(builder.environment_mut(), loc),
                            }
                        });
                let tp = result.value.clone();
                (CalleeKind::MethodCall { receiver: result.object, property: result.value }, tp)
            }
            // Callee is a regular (non-optional) member expression: method call pattern
            LowerableExpression::PropertyAccess { object, property, span: member_span } => {
                let member_loc = span_to_loc(*member_span);
                let obj_result = lower_expression(builder, object).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
                });
                let prop_value = InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: obj_result.place.clone(),
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    loc: member_loc,
                });
                let prop_place = lower_value_to_temporary(builder, prop_value, member_loc).place;
                let tp = prop_place.clone();
                (CalleeKind::MethodCall { receiver: obj_result.place, property: prop_place }, tp)
            }
            LowerableExpression::ComputedPropertyAccess { object, property, span: member_span } => {
                let member_loc = span_to_loc(*member_span);
                let obj_result = lower_expression(builder, object).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
                });
                let prop_result = lower_expression(builder, property).unwrap_or_else(|err| {
                    builder.errors.merge(err);
                    ExpressionResult {
                        place: create_temporary_place(builder.environment_mut(), loc),
                    }
                });
                let computed_value = InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: obj_result.place.clone(),
                    property: prop_result.place,
                    loc: member_loc,
                });
                let computed_place =
                    lower_value_to_temporary(builder, computed_value, member_loc).place;
                let tp = computed_place.clone();
                (
                    CalleeKind::MethodCall { receiver: obj_result.place, property: computed_place },
                    tp,
                )
            }
            // Callee is a plain expression
            _ => {
                let callee_place = lower_expression(builder, callee_expr)
                    .unwrap_or_else(|err| {
                        builder.errors.merge(err);
                        ExpressionResult {
                            place: create_temporary_place(builder.environment_mut(), loc),
                        }
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
fn compound_assignment_to_binary(op: AssignmentOperator) -> Option<BinaryOperator> {
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
        // Not compound operators; logical assignments (&&=, ||=, ??=) need special CFG handling
        AssignmentOperator::Assign
        | AssignmentOperator::LogicalAnd
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
        kind: LowerablePropertyKind,
        span: Span,
    },
    Spread(LowerableExpression<'a>, Span),
}

/// The kind of an object property (init, get, or set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LowerablePropertyKind {
    Init,
    Get,
    Set,
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
    /// A built-in tag like "div", "span" or namespaced like "fbt:enum".
    /// The Span covers the tag name (used for error reporting).
    BuiltIn(String, Span),
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

    /// An unsupported expression type (e.g., YieldExpression).
    /// Port of the default case in BuildHIR.ts lowerExpression.
    UnsupportedExpression {
        kind: String,
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

/// Return the string name of an expression's type, matching Babel's AST node type names.
/// Used for error messages matching the TS reference (BuildHIR.ts `expr.type`).
fn expression_type_name(expr: &ast::Expression<'_>) -> &'static str {
    match expr {
        ast::Expression::Identifier(_) => "Identifier",
        ast::Expression::StringLiteral(_) => "StringLiteral",
        ast::Expression::NumericLiteral(_) => "NumericLiteral",
        ast::Expression::BooleanLiteral(_) => "BooleanLiteral",
        ast::Expression::NullLiteral(_) => "NullLiteral",
        ast::Expression::BigIntLiteral(_) => "BigIntLiteral",
        ast::Expression::RegExpLiteral(_) => "RegExpLiteral",
        ast::Expression::TemplateLiteral(_) => "TemplateLiteral",
        ast::Expression::UnaryExpression(_) => "UnaryExpression",
        ast::Expression::BinaryExpression(_) => "BinaryExpression",
        ast::Expression::LogicalExpression(_) => "LogicalExpression",
        ast::Expression::ConditionalExpression(_) => "ConditionalExpression",
        ast::Expression::AssignmentExpression(_) => "AssignmentExpression",
        ast::Expression::SequenceExpression(_) => "SequenceExpression",
        ast::Expression::CallExpression(_) => "CallExpression",
        ast::Expression::NewExpression(_) => "NewExpression",
        ast::Expression::StaticMemberExpression(_)
        | ast::Expression::ComputedMemberExpression(_)
        | ast::Expression::PrivateFieldExpression(_) => "MemberExpression",
        ast::Expression::ArrayExpression(_) => "ArrayExpression",
        ast::Expression::ObjectExpression(_) => "ObjectExpression",
        ast::Expression::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
        ast::Expression::FunctionExpression(_) => "FunctionExpression",
        ast::Expression::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
        ast::Expression::TSAsExpression(_) => "TSAsExpression",
        ast::Expression::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        ast::Expression::TSNonNullExpression(_) => "TSNonNullExpression",
        ast::Expression::TSInstantiationExpression(_) => "TSInstantiationExpression",
        ast::Expression::TSTypeAssertion(_) => "TSTypeAssertion",
        ast::Expression::UpdateExpression(_) => "UpdateExpression",
        ast::Expression::AwaitExpression(_) => "AwaitExpression",
        ast::Expression::YieldExpression(_) => "YieldExpression",
        ast::Expression::ClassExpression(_) => "ClassExpression",
        ast::Expression::JSXElement(_) => "JSXElement",
        ast::Expression::JSXFragment(_) => "JSXFragment",
        ast::Expression::ChainExpression(_) => "ChainExpression",
        ast::Expression::ImportExpression(_) => "ImportExpression",
        ast::Expression::MetaProperty(_) => "MetaProperty",
        ast::Expression::Super(_) => "Super",
        ast::Expression::ThisExpression(_) => "ThisExpression",
        ast::Expression::ParenthesizedExpression(_) => "ParenthesizedExpression",
        ast::Expression::PrivateInExpression(_) => "PrivateInExpression",
        ast::Expression::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
    }
}

/// Check if an expression can be safely reordered (i.e., its evaluation order is unobservable).
///
/// Port of `isReorderableExpression` from BuildHIR.ts lines 2875-3020.
/// Used by `lower_reorderable_expression` to validate default values in destructuring
/// and switch case tests where evaluation order may differ from the original source.
fn is_reorderable_expression(expr: &ast::Expression<'_>, allow_local_identifiers: bool) -> bool {
    match expr {
        ast::Expression::Identifier(_) => {
            // Local identifiers are reorderable only when explicitly allowed.
            // Global identifiers are always reorderable, but since we cannot
            // distinguish here without builder context, conservatively use the flag.
            allow_local_identifiers
        }
        ast::Expression::RegExpLiteral(_)
        | ast::Expression::StringLiteral(_)
        | ast::Expression::NumericLiteral(_)
        | ast::Expression::NullLiteral(_)
        | ast::Expression::BooleanLiteral(_)
        | ast::Expression::BigIntLiteral(_)
        | ast::Expression::TemplateLiteral(_) => true,
        ast::Expression::UnaryExpression(unary) => {
            use oxc_syntax::operator::UnaryOperator;
            matches!(
                unary.operator,
                UnaryOperator::LogicalNot | UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) && is_reorderable_expression(&unary.argument, allow_local_identifiers)
        }
        ast::Expression::TSAsExpression(ts_as) => {
            is_reorderable_expression(&ts_as.expression, allow_local_identifiers)
        }
        ast::Expression::TSNonNullExpression(ts_nn) => {
            is_reorderable_expression(&ts_nn.expression, allow_local_identifiers)
        }
        ast::Expression::TSInstantiationExpression(ts_inst) => {
            is_reorderable_expression(&ts_inst.expression, allow_local_identifiers)
        }
        ast::Expression::LogicalExpression(logical) => {
            is_reorderable_expression(&logical.left, allow_local_identifiers)
                && is_reorderable_expression(&logical.right, allow_local_identifiers)
        }
        ast::Expression::ConditionalExpression(cond) => {
            is_reorderable_expression(&cond.test, allow_local_identifiers)
                && is_reorderable_expression(&cond.consequent, allow_local_identifiers)
                && is_reorderable_expression(&cond.alternate, allow_local_identifiers)
        }
        ast::Expression::ArrayExpression(arr) => arr.elements.iter().all(|el| match el {
            ast::ArrayExpressionElement::SpreadElement(_)
            | ast::ArrayExpressionElement::Elision(_) => false,
            // All expression variants: delegate to recursive check
            other => is_reorderable_expression(other.to_expression(), allow_local_identifiers),
        }),
        ast::Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            ast::ObjectPropertyKind::SpreadProperty(_) => false,
            ast::ObjectPropertyKind::ObjectProperty(p) => {
                !p.computed && is_reorderable_expression(&p.value, allow_local_identifiers)
            }
        }),
        // TS: ArrowFunctionExpression is reorderable if its body is an empty BlockStatement,
        // or if it's an expression body with no local identifier references
        // (BuildHIR.ts lines 2983-2996).
        ast::Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // Expression body `() => expr`: reorderable if the body expression is reorderable
                // with allowLocalIdentifiers=false (no captured variables allowed).
                // In oxc_ast, the expression body is stored as an ExpressionStatement in body.statements.
                match arrow.body.statements.first() {
                    Some(ast::Statement::ExpressionStatement(expr_stmt)) => {
                        is_reorderable_expression(&expr_stmt.expression, false)
                    }
                    _ => false,
                }
            } else {
                // Block body: reorderable only if the block is empty (`() => {}`)
                arrow.body.statements.is_empty()
            }
        }
        // CallExpression: reorderable if callee and all arguments are reorderable
        // (BuildHIR.ts lines 2998-3010).
        ast::Expression::CallExpression(call) => {
            let callee_reorderable = match &call.callee {
                ast::Expression::Identifier(_) => allow_local_identifiers,
                other => is_reorderable_expression(other, allow_local_identifiers),
            };
            callee_reorderable
                && call.arguments.iter().all(|arg| match arg {
                    ast::Argument::SpreadElement(_) => false,
                    other => {
                        is_reorderable_expression(other.to_expression(), allow_local_identifiers)
                    }
                })
        }
        // MemberExpression: reorderable if the innermost object is a global-like identifier
        // (i.e., not a locally bound variable). Since we can't resolve bindings here without
        // builder context, we use `allow_local_identifiers` to decide — when called from
        // destructuring defaults (allow_local_identifiers=true), member expressions on local
        // variables are considered reorderable. This matches TS behavior where the check
        // resolves identifiers against the scope (BuildHIR.ts lines 3219-3240).
        ast::Expression::StaticMemberExpression(member) => {
            let mut inner: &ast::Expression<'_> = &member.object;
            while let ast::Expression::StaticMemberExpression(m) = inner {
                inner = &m.object;
            }
            if let ast::Expression::ComputedMemberExpression(m) = inner {
                inner = &m.object;
            }
            matches!(inner, ast::Expression::Identifier(_)) && allow_local_identifiers
        }
        ast::Expression::ComputedMemberExpression(member) => {
            let mut inner: &ast::Expression<'_> = &member.object;
            while let ast::Expression::StaticMemberExpression(m) = inner {
                inner = &m.object;
            }
            if let ast::Expression::ComputedMemberExpression(m) = inner {
                inner = &m.object;
            }
            matches!(inner, ast::Expression::Identifier(_)) && allow_local_identifiers
        }
        // FunctionExpression and all other expression types are NOT reorderable.
        _ => false,
    }
}
