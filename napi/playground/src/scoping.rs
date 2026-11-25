use oxc::{
    ast::AstKind,
    semantic::{ScopeId, Semantic, SymbolFlags, SymbolId},
    span::{GetSpan, Span},
    syntax::reference::ReferenceFlags,
};
use serde::Serialize;

/// Original-source semantic data. All spans are UTF-8 bytes, captured before
/// transforms and the ESTree UTF-16 conversion. IDs are local to this snapshot.
pub fn serialize(semantic: &Semantic<'_>) -> napi::Result<String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        version: u8,
        offset_encoding: &'static str,
        scopes: Vec<Scope>,
        symbols: Vec<Symbol>,
        references: Vec<Reference>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Scope {
        id: usize,
        parent_id: Option<usize>,
        node_id: usize,
        kind: String,
        span: Span,
        flags: String,
        flags_bits: u32,
        is_function: bool,
        bindings: Vec<usize>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Symbol {
        id: usize,
        name: String,
        span: Span,
        flags: SymbolFlags,
        flags_bits: u32,
        scope_id: usize,
        node_id: usize,
        redeclarations: Vec<Declaration>,
        catch_var_binding_id: Option<usize>,
        references: Vec<usize>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Declaration {
        span: Span,
        node_id: usize,
        flags: SymbolFlags,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Reference {
        id: usize,
        name: String,
        node_id: usize,
        span: Span,
        scope_id: usize,
        symbol_id: Option<usize>,
        flags: ReferenceFlags,
        flags_bits: u32,
        is_read: bool,
        is_write: bool,
        is_type: bool,
        is_value: bool,
    }

    let scoping = semantic.scoping();
    let scopes = scoping
        .scope_descendants_from_root()
        .map(|id| {
            let node_id = scoping.get_node_id(id);
            let kind = semantic.nodes().get_node(node_id).kind();
            let flags = scoping.scope_flags(id);
            let mut bindings =
                scoping.get_bindings(id).values().map(|id| id.index()).collect::<Vec<_>>();
            bindings.sort_unstable();
            Scope {
                id: id.index(),
                parent_id: scoping.scope_parent_id(id).map(ScopeId::index),
                node_id: node_id.index(),
                kind: kind.debug_name().into_owned(),
                span: kind.span(),
                flags: format!("{flags:?}"),
                flags_bits: flags.bits().into(),
                is_function: flags.is_function(),
                bindings,
            }
        })
        .collect();
    let symbols = scoping
        .symbol_ids()
        .map(|id| {
            let flags = scoping.symbol_flags(id);
            Symbol {
                id: id.index(),
                name: scoping.symbol_name(id).into(),
                span: scoping.symbol_span(id),
                flags,
                flags_bits: flags.bits(),
                scope_id: scoping.symbol_scope_id(id).index(),
                node_id: scoping.symbol_declaration(id).index(),
                // Oxc includes the original declaration when this list is nonempty.
                redeclarations: scoping
                    .symbol_redeclarations(id)
                    .iter()
                    .map(|declaration| Declaration {
                        span: declaration.span,
                        node_id: declaration.declaration.index(),
                        flags: declaration.flags,
                    })
                    .collect(),
                catch_var_binding_id: scoping.catch_var_binding(id).map(SymbolId::index),
                references: scoping
                    .get_resolved_reference_ids(id)
                    .iter()
                    .map(|id| id.index())
                    .collect(),
            }
        })
        .collect();
    // Scanning semantic identifier nodes includes unresolved and type references,
    // without inferring occurrence scopes from source spans or identifier names.
    let mut references = semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::IdentifierReference(identifier) = node.kind() else { return None };
            let id = identifier.reference_id.get()?;
            let reference = scoping.get_reference(id);
            let flags = reference.flags();
            Some(Reference {
                id: id.index(),
                name: identifier.name.to_string(),
                node_id: reference.node_id().index(),
                span: identifier.span,
                scope_id: reference.scope_id().index(),
                symbol_id: reference.symbol_id().map(SymbolId::index),
                flags,
                flags_bits: flags.bits().into(),
                is_read: flags.is_read(),
                is_write: flags.is_write(),
                is_type: flags.is_type(),
                is_value: flags.is_value(),
            })
        })
        .collect::<Vec<_>>();
    references.sort_unstable_by_key(|reference| reference.id);

    serde_json::to_string(&Data {
        version: 1,
        offset_encoding: "utf8",
        scopes,
        symbols,
        references,
    })
    .map_err(|error| napi::Error::from_reason(error.to_string()))
}
