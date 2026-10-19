//! Generator for `oxc_formatter`.
//! Generates the `AstNodes` and `AstNode` types.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::define_generator,
    output::Output,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, StructOrEnum, TypeDef, TypeId},
};

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub fn formatter_output_path(file_name: &str) -> String {
    format!("{FORMATTER_CRATE_PATH}/src/ast_nodes/generated/{file_name}.rs")
}

pub fn get_node_type(ty: &TokenStream) -> TokenStream {
    quote! { AstNode<'a, #ty> }
}

/// AST nodes whose last child should have `following_span_start = 0`.
///
/// This ensures trailing comments are correctly attributed to the last child itself,
/// rather than being treated as leading comments of a following sibling outside the parent.
///
/// `ImportExpression` prints its children as an argument list (`to_arguments`), whose last item has none either.
const AST_NODE_WITHOUT_FOLLOWING_NODE_LIST: &[&str] = &[
    "FormalParameters",
    "ArrayPattern",
    "ObjectPattern",
    "ArrayAssignmentTarget",
    "ObjectAssignmentTarget",
    "ImportExpression",
];

const AST_NODE_WITH_FOLLOWING_NODE_LIST: &[&str] = &["Function", "Class"];

pub struct FormatterAstNodesGenerator;

define_generator!(FormatterAstNodesGenerator);

impl Generator for FormatterAstNodesGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let no_following_node_type_ids = get_no_following_node_type_ids(schema);

        let impls = schema
            .structs_and_enums()
            .filter_map(|type_def| match type_def {
                StructOrEnum::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_impls(struct_def, &no_following_node_type_ids, schema))
                }
                StructOrEnum::Enum(enum_def) if enum_def.visit.has_visitor() => {
                    Some(generate_enum_impls(enum_def, schema))
                }
                _ => None,
            })
            .collect::<TokenStream>();

        let ast_nodes_names = schema
            .structs()
            .filter_map(|struct_def| {
                if struct_def.kind.has_kind {
                    Some((
                        struct_def.ident(),
                        struct_def.lifetime(schema),
                        has_node_fields(struct_def, schema),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let ast_nodes_variants = ast_nodes_names.iter().map(|(name, lifetime, _)| {
            quote! {
                #name(AstNode<'a, #name #lifetime>),
            }
        });

        let span_match_arms = ast_nodes_names.iter().map(|(name, _, _)| {
            quote! { Self::#name(n) => n.span(), }
        });

        let address_match_arms = ast_nodes_names.iter().map(|(name, _, _)| {
            quote! { Self::#name(n) => n.address(), }
        });

        let following_of_child_match_arms = ast_nodes_names
            .iter()
            .filter(|(_, _, has_node_fields)| *has_node_fields)
            .map(|(name, _, _)| {
                quote! { Self::#name(n) => n.following_of_child(child, child_start), }
            });
        let childless_names = ast_nodes_names
            .iter()
            .filter(|(_, _, has_node_fields)| !*has_node_fields)
            .map(|(name, _, _)| name);

        let ast_nodes_debug_names = ast_nodes_names.iter().map(|(name, _, _)| {
            let debug_name = name.to_string();
            quote! { Self::#name(_) => #debug_name, }
        });

        let output = quote! {
            use oxc_allocator::ArenaVec;
            use oxc_ast::ast::*;
            use oxc_formatter_core::Format;
            use oxc_span::GetSpan;
            use oxc_str::{Ident, JSStr};
            use oxc_syntax::node::NodeId;
            ///@@line_break
            use crate::ast_nodes::{AsAstNodes, AstNode, Following, list_following, node_address};
            use crate::formatter::{
                JsFormatter,
                trivia::{format_leading_comments, format_trailing_comments},
            };

            ///@@line_break
            /// A typed view of an AST node with a kind, the entries of the formatter's ancestor stack.
            #[derive(Clone, Copy)]
            pub enum AstNodes<'a> {
                Dummy(),
                #(#ast_nodes_variants)*
            }

            impl AstNodes<'_> {
                /// Returns the span of this AST node.
                ///
                /// # Panics
                ///
                /// Panics when called on a `Dummy` node, which should never appear in a real AST.
                #[inline]
                pub fn span(&self) -> Span {
                    match self {
                        Self::Dummy() => panic!("Should never be called on a dummy node"),
                        #(#span_match_arms)*
                    }
                }

                ///@@line_break
                /// The address of the AST node, its identity together with the variant (0 for `Dummy`).
                #[inline]
                pub fn address(&self) -> usize {
                    match self {
                        Self::Dummy() => 0,
                        #(#address_match_arms)*
                    }
                }

                ///@@line_break
                /// The [`Following`] of the child at the address `child` (starting at `child_start`) if this node is its parent.
                /// `Dummy` is the parent of the root, which has no following node.
                pub fn following_of_child(&self, child: usize, child_start: u32) -> Option<Following> {
                    match self {
                        Self::Dummy() => Some(Following::Start(0)),
                        #(#following_of_child_match_arms)*
                        #(Self::#childless_names(_))|* => None,
                    }
                }

                ///@@line_break
                #[inline]
                pub fn debug_name(&self) -> &'static str {
                    match self {
                        Self::Dummy() => "Dummy",
                        #(#ast_nodes_debug_names)*
                    }
                }
            }

            #impls
        };

        Output::Rust { path: formatter_output_path("ast_nodes"), tokens: output }
    }
}

/// Get [`TypeId`]s of types which do not have a following node.
///
/// These are:
/// * All types which are variants of `Statement`.
/// * PLUS types listed in `AST_NODE_WITHOUT_FOLLOWING_NODE_LIST`.
/// * MINUS types listed in `AST_NODE_WITH_FOLLOWING_NODE_LIST`.
fn get_no_following_node_type_ids(schema: &Schema) -> Vec<TypeId> {
    let exclude_type_ids = AST_NODE_WITH_FOLLOWING_NODE_LIST
        .iter()
        .map(|&name| schema.type_names[name])
        .collect::<Vec<_>>();

    let mut type_ids = AST_NODE_WITHOUT_FOLLOWING_NODE_LIST
        .iter()
        .map(|&name| schema.type_names[name])
        .collect::<Vec<_>>();

    let statement_enum = schema.type_by_name("Statement").as_enum().unwrap();
    type_ids.extend(
        statement_enum
            .all_variants(schema)
            .filter_map(|variant| variant.field_type(schema))
            .map(|variant_type| variant_type.innermost_type(schema).id())
            .filter(|type_id| !exclude_type_ids.contains(type_id)),
    );

    type_ids
}

/// The type of a field unwrapped from `Option` and `Box`, and whether it was boxed.
fn unwrap_field_type<'s>(field_type_def: &'s TypeDef, schema: &'s Schema) -> (&'s TypeDef, bool) {
    let unwrapped = field_type_def.as_option().map_or(field_type_def, |opt| opt.inner_type(schema));
    match unwrapped {
        TypeDef::Box(box_def) => (box_def.inner_type(schema), true),
        _ => (unwrapped, false),
    }
}

/// `(is_not_ast_node, is_copyable)` of a field type from [`unwrap_field_type`], `None` for a `Cell`.
fn classify_field_type(type_def: &TypeDef, schema: &Schema) -> Option<(bool, bool)> {
    Some(match type_def {
        TypeDef::Struct(s_def) => {
            let is_copyable = is_copyable(&s_def.derives);
            (is_copyable || !s_def.visit.has_visitor(), is_copyable)
        }
        TypeDef::Enum(e_def) => (!e_def.visit.has_visitor(), is_copyable(&e_def.derives)),
        TypeDef::Primitive(_) => (true, true),
        TypeDef::Vec(vec) => {
            (vec.inner_type(schema).as_struct().is_some_and(|s| !s.visit.has_visitor()), false)
        }
        TypeDef::Cell(_) => return None,
        TypeDef::Option(_) | TypeDef::Box(_) | TypeDef::Pointer(_) => {
            unreachable!("Option/Box/pointer should have been unwrapped");
        }
    })
}

/// Whether the struct has a field holding AST nodes, so it has children.
fn has_node_fields(struct_def: &StructDef, schema: &Schema) -> bool {
    struct_def.fields.iter().filter(|field| field.name != "span").any(|field| {
        let (type_def, _) = unwrap_field_type(field.type_def(schema), schema);
        classify_field_type(type_def, schema).is_some_and(|(is_not_ast_node, _)| !is_not_ast_node)
    })
}

/// A field holding AST nodes, with its type unwrapped from `Option` and `Box`.
struct NodeField<'s> {
    field: &'s FieldDef,
    index: usize,
    is_option: bool,
    is_box: bool,
    type_def: &'s TypeDef,
}

fn generate_struct_impls(
    struct_def: &StructDef,
    no_following_node_type_ids: &[TypeId],
    schema: &Schema,
) -> TokenStream {
    let type_ty = struct_def.ty(schema);
    let has_kind = struct_def.kind.has_kind;

    let fields = &struct_def.fields;
    let mut node_fields = vec![];
    let mut methods = vec![];
    for (index, field) in fields.iter().enumerate() {
        if field.name == "span" {
            continue;
        }

        let field_type_def = field.type_def(schema);
        let is_option = field_type_def.is_option();
        let (original_field_type, is_box) = unwrap_field_type(field_type_def, schema);

        let field_name = &field.ident();
        let field_inner_ty = original_field_type.ty(schema);

        let Some((is_not_ast_node, is_copyable)) = classify_field_type(original_field_type, schema)
        else {
            if field.name == "node_id" {
                methods.push(quote! {
                    ///@@line_break
                    #[inline]
                    pub fn node_id(self) -> NodeId {
                        self.inner.node_id()
                    }
                });
            }
            continue;
        };

        let (return_type, body) = if is_not_ast_node {
            if is_copyable {
                (quote! { #field_inner_ty }, quote! { self.inner.#field_name })
            } else if is_option {
                (quote! { &'a #field_inner_ty }, quote! { self.inner.#field_name.as_ref() })
            } else {
                (quote! { &'a #field_inner_ty }, quote! { &self.inner.#field_name })
            }
        } else {
            node_fields.push(NodeField {
                field,
                index,
                is_option,
                is_box,
                type_def: original_field_type,
            });
            let body = match (is_option, is_box) {
                (true, true) => quote! { self.inner.#field_name.as_deref().map(AstNode::new) },
                (true, false) => quote! { self.inner.#field_name.as_ref().map(AstNode::new) },
                (false, true) => quote! { AstNode::new(self.inner.#field_name.as_ref()) },
                (false, false) => quote! { AstNode::new(&self.inner.#field_name) },
            };
            (quote! { AstNode<'a, #field_inner_ty> }, body)
        };

        let return_type = if is_option {
            quote! { Option<#return_type> }
        } else {
            return_type
        };

        methods.push(quote! {
            ///@@line_break
            #[inline]
            pub fn #field_name(self) -> #return_type {
                #body
            }
        });
    }

    let should_not_have_following_node = no_following_node_type_ids.contains(&struct_def.id);
    let following_of_child =
        generate_following_of_child(&node_fields, fields, should_not_have_following_node, schema);

    // A node without a kind is not on the ancestor stack, it is printed in its parent's frame
    let kindless_methods = (!has_kind).then(|| {
        quote! {
            ///@@line_break
            /// The node on top of the ancestor stack, as this node is not on it.
            pub fn parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
                let parent = f.context().ancestors().current();
                debug_assert!(
                    parent.following_of_child(self.address(), self.span().start).is_some(),
                    "a node without a kind is not a child of the current node {}",
                    parent.debug_name()
                );
                parent
            }

            ///@@line_break
            pub fn following_span_start(self, f: &JsFormatter<'_, 'a>) -> u32 {
                f.context().ancestors().following_span_start_of_child(self.address(), self.span().start)
            }
        }
    });

    let as_ast_nodes = has_kind.then(|| {
        let struct_name = struct_def.ident();
        quote! {
            ///@@line_break
            impl<'a> AsAstNodes<'a> for AstNode<'a, #type_ty> {
                #[inline]
                fn as_ast_nodes(&self) -> AstNodes<'a> {
                    AstNodes::#struct_name(*self)
                }
            }
        }
    });

    quote! {
        ///@@line_break
        impl<'a> AstNode<'a, #type_ty> {
            #(#methods)*

            #following_of_child

            #kindless_methods

            ///@@line_break
            pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
                format_leading_comments(
                    self.span()
                )
                .fmt(f);
            }

            ///@@line_break
            pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
                // The common case, skips the lookup of the enclosing and following nodes
                if f.context().comments().unprinted_comments().is_empty() {
                    return;
                }
                format_trailing_comments(
                    self.parent(f).span(),
                    self.inner.span(),
                    self.following_span_start(f),
                )
                .fmt(f);
            }
        }

        #as_ast_nodes
    }
}

/// Generates `following_of_child`: finds the field holding the child at the address `child`,
/// and returns the start of the node following it.
/// `None` for a struct without AST node fields, which has no children.
fn generate_following_of_child(
    node_fields: &[NodeField],
    fields: &[FieldDef],
    should_not_have_following_node: bool,
    schema: &Schema,
) -> Option<TokenStream> {
    if node_fields.is_empty() {
        return None;
    }
    let has_list = node_fields.iter().any(|node_field| node_field.type_def.is_vec());
    let has_kindless = node_fields.iter().any(|node_field| {
        node_field.type_def.as_struct().is_some_and(|struct_def| !struct_def.kind.has_kind)
    });
    let child_start =
        if has_list || has_kindless { quote!(child_start) } else { quote!(_child_start) };

    let checks = node_fields.iter().map(|node_field| {
        let field_name = node_field.field.ident();
        let following = generate_field_following(
            fields,
            node_field.index,
            should_not_have_following_node,
            schema,
        );

        // `&node`, or `Option<&node>` for an optional field
        let node = match (node_field.is_option, node_field.is_box) {
            (true, true) => quote! { self.inner.#field_name.as_deref() },
            (true, false) => quote! { self.inner.#field_name.as_ref() },
            (false, true) => quote! { self.inner.#field_name.as_ref() },
            (false, false) => quote! { &self.inner.#field_name },
        };
        // `if` the field's node is the child, by `is_node`, a `bool` expression of `node`
        let if_child = |is_node: TokenStream| {
            if node_field.is_option {
                quote! { if #node.is_some_and(|node| #is_node) }
            } else {
                quote! { let node = #node; if #is_node }
            }
        };

        match node_field.type_def {
            TypeDef::Struct(struct_def) if struct_def.kind.has_kind => {
                let if_child = if_child(quote! { node_address(node) == child });
                quote! {
                    #if_child {
                        return Some(#following);
                    }
                }
            }
            // Its children are printed as this node's children
            TypeDef::Struct(_) => {
                let checks = quote! {
                    if node_address(node) == child {
                        return Some(#following);
                    }
                    if let Some(following) = AstNode::new(node).following_of_child(child, child_start) {
                        return Some(following.or_inherited(|| #following));
                    }
                };
                if node_field.is_option {
                    quote! { if let Some(node) = #node { #checks } }
                } else {
                    quote! { let node = #node; #checks }
                }
            }
            TypeDef::Enum(_) => {
                let if_child =
                    if_child(quote! { AstNode::new(node).as_ast_nodes().address() == child });
                quote! {
                    #if_child {
                        return Some(#following);
                    }
                }
            }
            TypeDef::Vec(_) => {
                let following = if node_field.is_option {
                    quote! { #node.and_then(|node| list_following(node, child, child_start, || #following)) }
                } else {
                    quote! { list_following(#node, child, child_start, || #following) }
                };
                quote! {
                    if let Some(following) = #following {
                        return Some(following);
                    }
                }
            }
            _ => unreachable!("Unexpected node field type: {}", node_field.field.name),
        }
    });

    Some(quote! {
        ///@@line_break
        /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
        pub fn following_of_child(self, child: usize, #child_start: u32) -> Option<Following> {
            #(#checks)*
            None
        }
    })
}

/// Generates the [`Following`] of the field at `index`:
/// the start of the first node in the fields after it, or else the parent's own following node.
fn generate_field_following(
    fields: &[FieldDef],
    index: usize,
    should_not_have_following_node: bool,
    schema: &Schema,
) -> TokenStream {
    // The optional fields before the first non-optional one, `Option<&node>` each
    let mut optional_nodes = vec![];
    let mut start = None;
    for field in &fields[index + 1..] {
        match generate_next_following_node(field, schema) {
            Some(NextNode::Optional(node)) => optional_nodes.push(node),
            Some(NextNode::Required(node)) => {
                start = Some(quote! { #node.span().start });
                break;
            }
            None => {}
        }
    }

    let fallback = match start {
        Some(start) => quote! { Following::Start(#start) },
        None if should_not_have_following_node => quote! { Following::Start(0) },
        None => quote! { Following::Inherit },
    };
    match optional_nodes.as_slice() {
        [] => fallback,
        [node] => quote! { #node.map_or(#fallback, |n| Following::Start(n.span().start)) },
        [first, rest @ ..] => {
            let chain =
                rest.iter().fold(quote! { #first.map(|n| n.span().start) }, |chain, node| {
                    quote! { #chain.or_else(|| #node.map(|n| n.span().start)) }
                });
            quote! { #chain.map_or(#fallback, Following::Start) }
        }
    }
}

/// A field that can be the following node of the fields before it.
enum NextNode {
    /// `Option<&node>`, an optional field or the first item of a `Vec`
    Optional(TokenStream),
    /// The node of a non-optional field
    Required(TokenStream),
}

/// The node of a field holding AST nodes, `None` for other fields.
fn generate_next_following_node(next_field: &FieldDef, schema: &Schema) -> Option<NextNode> {
    let next_field_type_def = next_field.type_def(schema);
    let next_field_name = format_ident!("{}", next_field.name());
    let next_field_accessor = quote! {
        self.inner.#next_field_name
    };

    let innermost_type = next_field_type_def.innermost_type(schema);
    if match innermost_type {
        TypeDef::Struct(struct_def) => !struct_def.visit.has_visitor(),
        TypeDef::Enum(enum_def) => !enum_def.visit.has_visitor(),
        _ => true,
    } {
        return None;
    }

    match next_field_type_def {
        TypeDef::Box(_) | TypeDef::Struct(_) | TypeDef::Enum(_) => {
            Some(NextNode::Required(next_field_accessor))
        }
        TypeDef::Option(option_def) => {
            let inner_type_call = if option_def.inner_type(schema).is_box() {
                quote! { as_deref() }
            } else {
                quote! { as_ref() }
            };
            Some(NextNode::Optional(quote! { #next_field_accessor.#inner_type_call }))
        }
        TypeDef::Vec(_) => Some(NextNode::Optional(quote! { #next_field_accessor.first() })),
        _ => None,
    }
}

fn generate_enum_impls(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_ident = enum_def.ident();
    let type_ty = enum_def.ty(schema);

    let variant_match_arms = enum_def.variants.iter().map(|variant| {
        let variant_name = &variant.ident();
        let field_type = variant.field_type(schema).unwrap();
        let is_box = field_type.is_box();
        let node_type_ident = field_type
            .maybe_inner_type(schema)
            .map_or_else(|| field_type.ident(), TypeDef::ident);

        let inner_expr = if is_box { quote! { s.as_ref() } } else { quote! { s } };

        let implementation = if has_kind(field_type, schema) {
            quote! { AstNodes::#node_type_ident(AstNode::new(#inner_expr)) }
        } else {
            // This panic might indicate a need for further refinement or configuration in your schema/generation
            quote! {
                panic!("No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`")
            }
        };
        quote! { #enum_ident::#variant_name(s) => #implementation, }
    });

    let inherits_match_arms = enum_def.inherits_enums(schema).map(|inherited_enum_def| {
        let inherits_snake_name = inherited_enum_def.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");
        let to_fn_ident = format_ident!("to_{inherits_snake_name}");

        quote! {
            it @ #match_ident!(#enum_ident) => AstNode::new(it.#to_fn_ident()).as_ast_nodes(),
        }
    });

    let node_type = get_node_type(&type_ty);
    quote! {
        ///@@line_break
        impl<'a> AsAstNodes<'a> for #node_type {
            #[inline]
            fn as_ast_nodes(&self) -> AstNodes<'a> {
                match self.inner {
                    #(#variant_match_arms)*
                    #(#inherits_match_arms)*
                }
            }
        }
    }
}

fn has_kind(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(struct_def) => struct_def.kind.has_kind,
        TypeDef::Box(box_def) => has_kind(box_def.inner_type(schema), schema),
        _ => false,
    }
}

fn is_copyable(devices: &[String]) -> bool {
    devices.contains(&"Copy".to_string())
}
