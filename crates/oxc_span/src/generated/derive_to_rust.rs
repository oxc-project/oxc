// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/to_rust.rs`

#![allow(clippy::redundant_closure_for_method_calls)]

use crate::source_type::*;

impl ::oxc_quote_types::ToRust for SourceType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "SourceType",
            fields: ::std::vec![
                ("language", self.language.to_rust()),
                ("module_kind", self.module_kind.to_rust()),
                ("variant", self.variant.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Language {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::JavaScript => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Language",
                    variant: "JavaScript",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TypeScript => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Language",
                    variant: "TypeScript",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TypeScriptDefinition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Language",
                    variant: "TypeScriptDefinition",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ModuleKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Script => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleKind",
                    variant: "Script",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Module => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleKind",
                    variant: "Module",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Unambiguous => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleKind",
                    variant: "Unambiguous",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for LanguageVariant {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Standard => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LanguageVariant",
                    variant: "Standard",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Jsx => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LanguageVariant",
                    variant: "Jsx",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}
