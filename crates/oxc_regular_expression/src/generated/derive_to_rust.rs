// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/to_rust.rs`

#![allow(clippy::redundant_closure_for_method_calls)]

use crate::ast::*;

impl ::oxc_quote_types::ToRust for Pattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Pattern",
            fields: ::std::vec![("span", self.span.to_rust()), ("body", self.body.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Disjunction<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Disjunction",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Alternative<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Alternative",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Term<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::BoundaryAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "BoundaryAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LookAroundAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "LookAroundAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Quantifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "Quantifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Character(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "Character",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Dot(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "Dot",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::CharacterClassEscape(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "CharacterClassEscape",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnicodePropertyEscape(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "UnicodePropertyEscape",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CharacterClass(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "CharacterClass",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CapturingGroup(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "CapturingGroup",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IgnoreGroup(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "IgnoreGroup",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IndexedReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "IndexedReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NamedReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Term",
                    variant: "NamedReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for BoundaryAssertion {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BoundaryAssertion",
            fields: ::std::vec![("span", self.span.to_rust()), ("kind", self.kind.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BoundaryAssertionKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Start => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BoundaryAssertionKind",
                    variant: "Start",
                    field: ::std::option::Option::None,
                }))
            }
            Self::End => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BoundaryAssertionKind",
                    variant: "End",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Boundary => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BoundaryAssertionKind",
                    variant: "Boundary",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeBoundary => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BoundaryAssertionKind",
                    variant: "NegativeBoundary",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for LookAroundAssertion<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "LookAroundAssertion",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for LookAroundAssertionKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Lookahead => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LookAroundAssertionKind",
                    variant: "Lookahead",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeLookahead => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LookAroundAssertionKind",
                    variant: "NegativeLookahead",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Lookbehind => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LookAroundAssertionKind",
                    variant: "Lookbehind",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeLookbehind => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LookAroundAssertionKind",
                    variant: "NegativeLookbehind",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for Quantifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Quantifier",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("min", self.min.to_rust()),
                (
                    "max",
                    ::oxc_quote_types::Node::Option(
                        self.max.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("greedy", self.greedy.to_rust()),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Character {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Character",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                ("value", self.value.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CharacterKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ControlLetter => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "ControlLetter",
                    field: ::std::option::Option::None,
                }))
            }
            Self::HexadecimalEscape => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "HexadecimalEscape",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Identifier => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Identifier",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Null => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Null",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Octal1 => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Octal1",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Octal2 => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Octal2",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Octal3 => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Octal3",
                    field: ::std::option::Option::None,
                }))
            }
            Self::SingleEscape => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "SingleEscape",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Symbol => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "Symbol",
                    field: ::std::option::Option::None,
                }))
            }
            Self::UnicodeEscape => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterKind",
                    variant: "UnicodeEscape",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for CharacterClassEscape {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CharacterClassEscape",
            fields: ::std::vec![("span", self.span.to_rust()), ("kind", self.kind.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CharacterClassEscapeKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::D => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "D",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeD => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "NegativeD",
                    field: ::std::option::Option::None,
                }))
            }
            Self::S => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "S",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeS => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "NegativeS",
                    field: ::std::option::Option::None,
                }))
            }
            Self::W => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "W",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NegativeW => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassEscapeKind",
                    variant: "NegativeW",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for UnicodePropertyEscape<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "UnicodePropertyEscape",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("negative", self.negative.to_rust()),
                ("strings", self.strings.to_rust()),
                ("name", self.name.to_rust()),
                (
                    "value",
                    ::oxc_quote_types::Node::Option(
                        self.value.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Dot {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Dot",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CharacterClass<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CharacterClass",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("negative", self.negative.to_rust()),
                ("strings", self.strings.to_rust()),
                ("kind", self.kind.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CharacterClassContentsKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Union => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContentsKind",
                    variant: "Union",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Intersection => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContentsKind",
                    variant: "Intersection",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Subtraction => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContentsKind",
                    variant: "Subtraction",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for CharacterClassContents<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::CharacterClassRange(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "CharacterClassRange",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CharacterClassEscape(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "CharacterClassEscape",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnicodePropertyEscape(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "UnicodePropertyEscape",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Character(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "Character",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NestedCharacterClass(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "NestedCharacterClass",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassStringDisjunction(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CharacterClassContents",
                    variant: "ClassStringDisjunction",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for CharacterClassRange {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CharacterClassRange",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("min", self.min.to_rust()),
                ("max", self.max.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ClassStringDisjunction<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ClassStringDisjunction",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("strings", self.strings.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ClassString<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ClassString",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("strings", self.strings.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CapturingGroup<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CapturingGroup",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "name",
                    ::oxc_quote_types::Node::Option(
                        self.name.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for IgnoreGroup<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "IgnoreGroup",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "modifiers",
                    ::oxc_quote_types::Node::Option(
                        self.modifiers.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Modifiers {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Modifiers",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "enabling",
                    ::oxc_quote_types::Node::Option(
                        self.enabling.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "disabling",
                    ::oxc_quote_types::Node::Option(
                        self.disabling.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Modifier {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Modifier",
            fields: ::std::vec![
                ("ignore_case", self.ignore_case.to_rust()),
                ("multiline", self.multiline.to_rust()),
                ("sticky", self.sticky.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for IndexedReference {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "IndexedReference",
            fields: ::std::vec![("span", self.span.to_rust()), ("index", self.index.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for NamedReference<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "NamedReference",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}
