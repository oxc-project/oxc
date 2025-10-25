use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, ReferenceId, SymbolFlags};
use oxc_span::SPAN;
use oxc_traverse::BoundIdentifier;
use rustc_hash::FxHashMap;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

#[derive(Debug, Default)]
pub struct HoistStaticMethodsState<'a> {
    methods: FxHashMap<Atom<'a>, MethodInfo<'a>>,
}

#[derive(Debug)]
struct MethodInfo<'a> {
    ref_ids: Vec<ReferenceId>,
    binding: Option<BoundIdentifier<'a>>,
    inserted: bool,
}

impl<'a> PeepholeOptimizations {
    pub fn hoist_static_methods(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::CallExpression(call_expr) = expr else { return };
        let Expression::StaticMemberExpression(e) = &call_expr.callee else { return };

        if !e.object.is_specific_id("Object") {
            return;
        }

        if !matches!(
            e.property.name.as_str(),
            "keys" | "entries" | "freeze" | "defineProperty" | "seal" | "getOwnPropertyDescriptor"
        ) {
            return;
        }

        let method_name_key = e.property.name;
        let needs_new_binding =
            !ctx.state.hoist_known_methods_state.methods.contains_key(&method_name_key)
                || ctx.state.hoist_known_methods_state.methods[&method_name_key].binding.is_none();

        let binding_name = if needs_new_binding {
            let uid = ctx.generate_uid_in_root_scope("c", SymbolFlags::Variable);
            let name = uid.name;
            let method_info =
                ctx.state.hoist_known_methods_state.methods.entry(method_name_key).or_insert_with(
                    || MethodInfo { ref_ids: Vec::new(), binding: None, inserted: false },
                );
            method_info.binding = Some(uid);
            name
        } else {
            ctx.state.hoist_known_methods_state.methods[&method_name_key]
                .binding
                .as_ref()
                .unwrap()
                .name
        };

        let reference_id =
            ctx.create_unbound_reference(binding_name.as_str(), ReferenceFlags::Read);
        call_expr.callee = ctx.ast.expression_identifier_with_reference_id(
            SPAN,
            binding_name.as_str(),
            reference_id,
        );

        ctx.state
            .hoist_known_methods_state
            .methods
            .get_mut(&method_name_key)
            .unwrap()
            .ref_ids
            .push(reference_id);
        ctx.state.changed = true;
    }

    pub fn insert_static_methods_bindings(program: &mut Program<'a>, ctx: &mut Ctx<'a, '_>) {
        if ctx.state.hoist_known_methods_state.methods.is_empty() {
            return;
        }

        let has_methods = ctx
            .state
            .hoist_known_methods_state
            .methods
            .values()
            .any(|info| !info.ref_ids.is_empty() && !info.inserted);

        if !has_methods {
            return;
        }

        let reference_id = ctx.create_unbound_reference("Object", ReferenceFlags::Read);
        let mut properties = ctx.ast.vec();

        let mut tmp = FxHashMap::default();
        std::mem::swap(&mut ctx.state.hoist_known_methods_state.methods, &mut tmp);

        {
            for (key, method_info) in &mut tmp {
                let key_str = key.as_str();
                if method_info.ref_ids.is_empty() || method_info.inserted {
                    continue;
                }

                let binding = method_info.binding.as_ref().unwrap();
                properties.push(ctx.ast.binding_property(
                    SPAN,
                    ctx.ast.property_key_static_identifier(SPAN, key_str),
                    ctx.ast.binding_pattern(
                        ctx.ast.binding_pattern_kind_binding_identifier_with_symbol_id(
                            SPAN,
                            binding.name,
                            binding.symbol_id,
                        ),
                        NONE,
                        false,
                    ),
                    true,
                    false,
                ));

                if !method_info.ref_ids.is_empty() {
                    method_info.inserted = true;
                }
            }
        }

        if properties.is_empty() {
            return;
        }

        let declarations = ctx.ast.vec1(ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            ctx.ast.binding_pattern(
                ctx.ast.binding_pattern_kind_object_pattern(SPAN, properties, NONE),
                NONE,
                false,
            ),
            Some(ctx.ast.expression_identifier_with_reference_id(SPAN, "Object", reference_id)),
            false,
        ));

        let var_decl = ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Const,
            declarations,
            false,
        );

        program.body.insert(0, Statement::VariableDeclaration(var_decl));

        ctx.state.changed = true;
    }
}

#[cfg(test)]
mod test {
    use oxc_span::SourceType;

    use crate::{
        CompressOptions,
        tester::{
            test_options, test_options_source_type, test_same_options,
            test_same_options_source_type,
        },
    };

    #[test]
    fn testing() {}
}
