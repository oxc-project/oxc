use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_types::{TypeData, TypeFlags, TypeId};

use crate::Checker;

/// Selects which error reporter `check_non_null_type_with_reporter` uses.
/// Mirrors tsgo's function-pointer approach to `checkNonNullTypeWithReporter`.
pub(crate) enum NullableErrorReporter {
    /// TS18050 / TS2531 / TS2532 / TS2533 — general "value cannot be used here" / "possibly null".
    /// Used by most call sites (unary ops, property access, binary ops, etc.).
    Default,
    /// TS2721 / TS2722 / TS2723 — "cannot invoke an object which is possibly null/undefined".
    /// Used by call expression checking.
    CannotInvoke,
}

impl Checker<'_> {
    /// Create a union type of `type_id | undefined`.
    /// If the type already contains undefined, returns it unchanged.
    pub(crate) fn get_optional_type(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        // Already undefined or contains undefined
        if flags.intersects(TypeFlags::Undefined) {
            return type_id;
        }

        // If it's a union, check if any member is undefined
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(union_data) = self.type_arena.get_data(type_id) {
                if union_data
                    .types
                    .iter()
                    .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::Undefined))
                {
                    return type_id;
                }
            }
        }

        self.get_or_create_union_type(vec![type_id, self.undefined_type])
    }

    /// Check if a type contains undefined (directly or as a union member).
    /// Handles nested unions (union members that are themselves unions).
    pub(crate) fn contains_undefined_type(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::Undefined) {
            return true;
        }

        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(union_data) = self.type_arena.get_data(type_id) {
                return union_data.types.iter().any(|&t| self.contains_undefined_type(t));
            }
        }

        false
    }

    /// Strip `null` and `undefined` from a type.
    /// For unions: filters out Nullable constituents. For bare null/undefined: returns never.
    /// Mirrors tsgo's `GetNonNullableType` (without strictNullChecks gating for now).
    pub(crate) fn get_non_nullable_type(&mut self, type_id: TypeId) -> TypeId {
        let (has_null, has_undefined, non_nullable) = self.get_nullable_info(type_id);
        if !has_null && !has_undefined {
            return type_id;
        }
        non_nullable
    }

    /// Single-pass nullable analysis: determines whether a type contains null/undefined
    /// and computes the stripped (non-nullable) type in one traversal.
    ///
    /// Returns `(has_null, has_undefined, non_nullable_type)`.
    /// `non_nullable_type` is only meaningful when `has_null || has_undefined`.
    fn get_nullable_info(&mut self, type_id: TypeId) -> (bool, bool, TypeId) {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let mut has_null = false;
                let mut has_undefined = false;
                let mut filtered = Vec::new();

                for &t in u.types.iter() {
                    let t_flags = self.type_arena.get_flags(t);
                    if t_flags.intersects(TypeFlags::Null) {
                        has_null = true;
                    } else if t_flags.intersects(TypeFlags::Undefined) {
                        has_undefined = true;
                    } else {
                        filtered.push(t);
                    }
                }

                if !has_null && !has_undefined {
                    return (false, false, type_id);
                }

                let non_nullable = if filtered.is_empty() {
                    self.never_type
                } else {
                    self.get_or_create_union_type(filtered)
                };
                return (has_null, has_undefined, non_nullable);
            }
        }

        // Non-union: check the type directly
        let has_null = flags.intersects(TypeFlags::Null);
        let has_undefined = flags.intersects(TypeFlags::Undefined);
        let non_nullable = if has_null || has_undefined { self.never_type } else { type_id };
        (has_null, has_undefined, non_nullable)
    }

    /// Check that a type is non-null/non-undefined. If it contains nullable constituents,
    /// reports an error and returns the stripped type. Returns the type unchanged if non-nullable.
    ///
    /// Mirrors tsgo's `checkNonNullType`. Uses the default error reporter
    /// (`reportObjectPossiblyNullOrUndefinedError`).
    pub(crate) fn check_non_null_type(&mut self, type_id: TypeId, expr: &Expression<'_>) -> TypeId {
        self.check_non_null_type_with_reporter(type_id, expr, NullableErrorReporter::Default)
    }

    /// Check that a type is non-null/non-undefined, using a caller-chosen error
    /// reporter. Mirrors tsgo's `checkNonNullTypeWithReporter`.
    pub(crate) fn check_non_null_type_with_reporter(
        &mut self,
        type_id: TypeId,
        expr: &Expression<'_>,
        reporter: NullableErrorReporter,
    ) -> TypeId {
        let (has_null, has_undefined, non_nullable) = self.get_nullable_info(type_id);

        if !has_null && !has_undefined {
            return type_id;
        }

        match reporter {
            NullableErrorReporter::Default => {
                self.report_possibly_null_or_undefined_error(expr, has_null, has_undefined);
            }
            NullableErrorReporter::CannotInvoke => {
                self.report_cannot_invoke_possibly_null_or_undefined_error(
                    expr,
                    has_null,
                    has_undefined,
                );
            }
        }

        let flags = self.type_arena.get_flags(non_nullable);
        // If type was purely null/undefined, stripping leaves never → use any as error recovery
        if flags.intersects(TypeFlags::Never) {
            return self.any_type;
        }
        non_nullable
    }

    /// Report TS18050 / TS18047-18049 / TS2531-2533 depending on the expression
    /// and which nullable constituents are present.
    /// Mirrors tsgo's `reportObjectPossiblyNullOrUndefinedError`.
    fn report_possibly_null_or_undefined_error(
        &mut self,
        expr: &Expression<'_>,
        has_null: bool,
        has_undefined: bool,
    ) {
        let span = expr.span();

        // null keyword → TS18050
        if matches!(expr, Expression::NullLiteral(_)) {
            self.diagnostics.push(
                OxcDiagnostic::error("The value 'null' cannot be used here.")
                    .with_error_code("ts", "18050")
                    .with_label(span),
            );
            return;
        }

        // Entity name expression (identifier or `a.b.c` chain) →
        // TS18050 for `undefined`, else TS18047/18048/18049 with the name.
        // Mirrors tsgo's branch: isEntityNameExpression(node) && len < 100.
        if let Some(name) = entity_name_expression_text(expr) {
            if name.len() < 100 {
                if matches!(expr, Expression::Identifier(id) if id.name == "undefined") {
                    self.diagnostics.push(
                        OxcDiagnostic::error("The value 'undefined' cannot be used here.")
                            .with_error_code("ts", "18050")
                            .with_label(span),
                    );
                    return;
                }
                let (code, msg) = match (has_null, has_undefined) {
                    (true, true) => ("18049", format!("'{name}' is possibly 'null' or 'undefined'.")),
                    (true, false) => ("18047", format!("'{name}' is possibly 'null'.")),
                    _ => ("18048", format!("'{name}' is possibly 'undefined'.")),
                };
                self.diagnostics.push(
                    OxcDiagnostic::error(msg)
                        .with_error_code("ts", code)
                        .with_label(span),
                );
                return;
            }
        }

        // Other expressions → TS2531/2532/2533
        let (code, msg) = match (has_null, has_undefined) {
            (true, true) => ("2533", "Object is possibly 'null' or 'undefined'."),
            (true, false) => ("2531", "Object is possibly 'null'."),
            _ => ("2532", "Object is possibly 'undefined'."),
        };
        self.diagnostics
            .push(OxcDiagnostic::error(msg).with_error_code("ts", code).with_label(span));
    }

    /// Report TS2721 / TS2722 / TS2723 for attempting to invoke a possibly-null type.
    /// Mirrors tsgo's `reportCannotInvokePossiblyNullOrUndefinedError`.
    fn report_cannot_invoke_possibly_null_or_undefined_error(
        &mut self,
        expr: &Expression<'_>,
        has_null: bool,
        has_undefined: bool,
    ) {
        let span = expr.span();
        let (code, msg) = match (has_null, has_undefined) {
            (true, true) => {
                ("2723", "Cannot invoke an object which is possibly 'null' or 'undefined'.")
            }
            (true, false) => ("2721", "Cannot invoke an object which is possibly 'null'."),
            _ => ("2722", "Cannot invoke an object which is possibly 'undefined'."),
        };
        self.diagnostics
            .push(OxcDiagnostic::error(msg).with_error_code("ts", code).with_label(span));
    }
}

/// If `expr` is an entity name expression (identifier or `a.b.c` property-access
/// chain), return its dotted text. Returns `None` for anything else.
/// Mirrors tsgo's `isEntityNameExpression` + `entityNameToString`.
fn entity_name_expression_text(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::Identifier(id) => Some(id.name.to_string()),
        Expression::StaticMemberExpression(member) => {
            let object_text = entity_name_expression_text(&member.object)?;
            Some(format!("{object_text}.{}", member.property.name))
        }
        _ => None,
    }
}
