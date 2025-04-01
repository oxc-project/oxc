// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/to_rust.rs`

#![allow(clippy::redundant_closure_for_method_calls)]

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl ::oxc_quote_types::ToRust for Program<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Program",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("source_type", self.source_type.to_rust()),
                ("source_text", self.source_text.to_rust()),
                (
                    "comments",
                    ::oxc_quote_types::Node::Vec(
                        self.comments.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "hashbang",
                    ::oxc_quote_types::Node::Option(
                        self.hashbang.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "directives",
                    ::oxc_quote_types::Node::Vec(
                        self.directives.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Expression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Expression",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for IdentifierName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "IdentifierName",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for IdentifierReference<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "IdentifierReference",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                ("reference_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BindingIdentifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BindingIdentifier",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                ("symbol_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for LabelIdentifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "LabelIdentifier",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ThisExpression {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ThisExpression",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ArrayExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ArrayExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "elements",
                    ::oxc_quote_types::Node::Vec(
                        self.elements.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "trailing_comma",
                    ::oxc_quote_types::Node::Option(
                        self.trailing_comma.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ArrayExpressionElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::SpreadElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "SpreadElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Elision(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "Elision",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ArrayExpressionElement",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for Elision {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Elision",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ObjectExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ObjectExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "properties",
                    ::oxc_quote_types::Node::Vec(
                        self.properties.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "trailing_comma",
                    ::oxc_quote_types::Node::Option(
                        self.trailing_comma.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ObjectPropertyKind<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ObjectProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ObjectPropertyKind",
                    variant: "ObjectProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SpreadProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ObjectPropertyKind",
                    variant: "SpreadProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ObjectProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ObjectProperty",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                ("key", self.key.to_rust()),
                ("value", self.value.to_rust()),
                ("method", self.method.to_rust()),
                ("shorthand", self.shorthand.to_rust()),
                ("computed", self.computed.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for PropertyKey<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::StaticIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "StaticIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "PrivateIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKey",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for PropertyKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Init => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKind",
                    variant: "Init",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Get => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKind",
                    variant: "Get",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Set => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyKind",
                    variant: "Set",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TemplateLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TemplateLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "quasis",
                    ::oxc_quote_types::Node::Vec(self.quasis.iter().map(|v| v.to_rust()).collect())
                ),
                (
                    "expressions",
                    ::oxc_quote_types::Node::Vec(
                        self.expressions.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TaggedTemplateExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TaggedTemplateExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("tag", self.tag.to_rust()),
                ("quasi", self.quasi.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TemplateElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TemplateElement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("value", self.value.to_rust()),
                ("tail", self.tail.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TemplateElementValue<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TemplateElementValue",
            fields: ::std::vec![
                ("raw", self.raw.to_rust()),
                (
                    "cooked",
                    ::oxc_quote_types::Node::Option(
                        self.cooked.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for MemberExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MemberExpression",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MemberExpression",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MemberExpression",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ComputedMemberExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ComputedMemberExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object", self.object.to_rust()),
                ("expression", self.expression.to_rust()),
                ("optional", self.optional.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for StaticMemberExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "StaticMemberExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object", self.object.to_rust()),
                ("property", self.property.to_rust()),
                ("optional", self.optional.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for PrivateFieldExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "PrivateFieldExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object", self.object.to_rust()),
                ("field", self.field.to_rust()),
                ("optional", self.optional.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CallExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CallExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("callee", self.callee.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "arguments",
                    ::oxc_quote_types::Node::Vec(
                        self.arguments.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("optional", self.optional.to_rust()),
                ("pure", self.pure.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for NewExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "NewExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("callee", self.callee.to_rust()),
                (
                    "arguments",
                    ::oxc_quote_types::Node::Vec(
                        self.arguments.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("pure", self.pure.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for MetaProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "MetaProperty",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("meta", self.meta.to_rust()),
                ("property", self.property.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for SpreadElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "SpreadElement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Argument<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::SpreadElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "SpreadElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Argument",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for UpdateExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "UpdateExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("operator", self.operator.to_rust()),
                ("prefix", self.prefix.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for UnaryExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "UnaryExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("operator", self.operator.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BinaryExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BinaryExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("operator", self.operator.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for PrivateInExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "PrivateInExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for LogicalExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "LogicalExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("operator", self.operator.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ConditionalExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ConditionalExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("test", self.test.to_rust()),
                ("consequent", self.consequent.to_rust()),
                ("alternate", self.alternate.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("operator", self.operator.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTarget<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::AssignmentTargetIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "AssignmentTargetIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "ArrayAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTarget",
                    variant: "ObjectAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for SimpleAssignmentTarget<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::AssignmentTargetIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "AssignmentTargetIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "SimpleAssignmentTarget",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ArrayAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetPattern",
                    variant: "ArrayAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetPattern",
                    variant: "ObjectAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ArrayAssignmentTarget<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ArrayAssignmentTarget",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "elements",
                    ::oxc_quote_types::Node::Vec(
                        self.elements
                            .iter()
                            .map(|v| ::oxc_quote_types::Node::Option(
                                v.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                            ))
                            .collect()
                    )
                ),
                (
                    "rest",
                    ::oxc_quote_types::Node::Option(
                        self.rest.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "trailing_comma",
                    ::oxc_quote_types::Node::Option(
                        self.trailing_comma.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ObjectAssignmentTarget<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ObjectAssignmentTarget",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "properties",
                    ::oxc_quote_types::Node::Vec(
                        self.properties.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "rest",
                    ::oxc_quote_types::Node::Option(
                        self.rest.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetRest<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentTargetRest",
            fields: ::std::vec![("span", self.span.to_rust()), ("target", self.target.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetMaybeDefault<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::AssignmentTargetWithDefault(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "AssignmentTargetWithDefault",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentTargetIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "AssignmentTargetIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "ArrayAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetMaybeDefault",
                    variant: "ObjectAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetWithDefault<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentTargetWithDefault",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("binding", self.binding.to_rust()),
                ("init", self.init.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::AssignmentTargetPropertyIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetProperty",
                    variant: "AssignmentTargetPropertyIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentTargetPropertyProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentTargetProperty",
                    variant: "AssignmentTargetPropertyProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetPropertyIdentifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentTargetPropertyIdentifier",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("binding", self.binding.to_rust()),
                (
                    "init",
                    ::oxc_quote_types::Node::Option(
                        self.init.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AssignmentTargetPropertyProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentTargetPropertyProperty",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                ("binding", self.binding.to_rust()),
                ("computed", self.computed.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for SequenceExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "SequenceExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "expressions",
                    ::oxc_quote_types::Node::Vec(
                        self.expressions.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Super {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Super",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for AwaitExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AwaitExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ChainExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ChainExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ChainElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ChainElement",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ChainElement",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ChainElement",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ChainElement",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ChainElement",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ParenthesizedExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ParenthesizedExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Statement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::BlockStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "BlockStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BreakStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "BreakStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ContinueStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ContinueStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::DebuggerStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "DebuggerStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::DoWhileStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "DoWhileStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::EmptyStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "EmptyStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExpressionStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ExpressionStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ForInStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ForInStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ForOfStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ForOfStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ForStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ForStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IfStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "IfStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LabeledStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "LabeledStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ReturnStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ReturnStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SwitchStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "SwitchStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThrowStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ThrowStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TryStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TryStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::WhileStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "WhileStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::WithStatement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "WithStatement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::VariableDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "VariableDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "FunctionDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ClassDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAliasDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSTypeAliasDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInterfaceDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSInterfaceDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSEnumDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSEnumDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSModuleDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSModuleDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSImportEqualsDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSImportEqualsDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ImportDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportAllDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ExportAllDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportDefaultDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ExportDefaultDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportNamedDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "ExportNamedDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSExportAssignment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSExportAssignment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNamespaceExportDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Statement",
                    variant: "TSNamespaceExportDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for Directive<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Directive",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                ("directive", self.directive.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Hashbang<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Hashbang",
            fields: ::std::vec![("span", self.span.to_rust()), ("value", self.value.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BlockStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BlockStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Declaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::VariableDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "VariableDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "FunctionDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "ClassDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAliasDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "TSTypeAliasDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInterfaceDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "TSInterfaceDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSEnumDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "TSEnumDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSModuleDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "TSModuleDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSImportEqualsDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "Declaration",
                    variant: "TSImportEqualsDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for VariableDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "VariableDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                (
                    "declarations",
                    ::oxc_quote_types::Node::Vec(
                        self.declarations.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("declare", self.declare.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for VariableDeclarationKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Var => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "VariableDeclarationKind",
                    variant: "Var",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Let => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "VariableDeclarationKind",
                    variant: "Let",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Const => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "VariableDeclarationKind",
                    variant: "Const",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Using => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "VariableDeclarationKind",
                    variant: "Using",
                    field: ::std::option::Option::None,
                }))
            }
            Self::AwaitUsing => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "VariableDeclarationKind",
                    variant: "AwaitUsing",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for VariableDeclarator<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "VariableDeclarator",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "init",
                    ::oxc_quote_types::Node::Option(
                        self.init.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("definite", self.definite.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for EmptyStatement {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "EmptyStatement",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ExpressionStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ExpressionStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for IfStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "IfStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("test", self.test.to_rust()),
                ("consequent", self.consequent.to_rust()),
                (
                    "alternate",
                    ::oxc_quote_types::Node::Option(
                        self.alternate.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for DoWhileStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "DoWhileStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("body", self.body.to_rust()),
                ("test", self.test.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for WhileStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "WhileStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("test", self.test.to_rust()),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ForStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ForStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "init",
                    ::oxc_quote_types::Node::Option(
                        self.init.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "test",
                    ::oxc_quote_types::Node::Option(
                        self.test.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "update",
                    ::oxc_quote_types::Node::Option(
                        self.update.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("body", self.body.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ForStatementInit<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::VariableDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "VariableDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementInit",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ForInStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ForInStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust()),
                ("body", self.body.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ForStatementLeft<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::VariableDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "VariableDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentTargetIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "AssignmentTargetIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "ArrayAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectAssignmentTarget(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ForStatementLeft",
                    variant: "ObjectAssignmentTarget",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ForOfStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ForOfStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("await", self.r#await.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust()),
                ("body", self.body.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ContinueStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ContinueStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "label",
                    ::oxc_quote_types::Node::Option(
                        self.label.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BreakStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BreakStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "label",
                    ::oxc_quote_types::Node::Option(
                        self.label.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ReturnStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ReturnStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "argument",
                    ::oxc_quote_types::Node::Option(
                        self.argument.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for WithStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "WithStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object", self.object.to_rust()),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for SwitchStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "SwitchStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("discriminant", self.discriminant.to_rust()),
                (
                    "cases",
                    ::oxc_quote_types::Node::Vec(self.cases.iter().map(|v| v.to_rust()).collect())
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for SwitchCase<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "SwitchCase",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "test",
                    ::oxc_quote_types::Node::Option(
                        self.test.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "consequent",
                    ::oxc_quote_types::Node::Vec(
                        self.consequent.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for LabeledStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "LabeledStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("label", self.label.to_rust()),
                ("body", self.body.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ThrowStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ThrowStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TryStatement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TryStatement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "block",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.block.to_rust()))
                ),
                (
                    "handler",
                    ::oxc_quote_types::Node::Option(self.handler.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "finalizer",
                    ::oxc_quote_types::Node::Option(self.finalizer.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CatchClause<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CatchClause",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "param",
                    ::oxc_quote_types::Node::Option(
                        self.param.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("body", ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.body.to_rust()))),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CatchParameter<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "CatchParameter",
            fields: ::std::vec![("span", self.span.to_rust()), ("pattern", self.pattern.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for DebuggerStatement {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "DebuggerStatement",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BindingPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BindingPattern",
            fields: ::std::vec![
                ("kind", self.kind.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("optional", self.optional.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BindingPatternKind<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::BindingIdentifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BindingPatternKind",
                    variant: "BindingIdentifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectPattern(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BindingPatternKind",
                    variant: "ObjectPattern",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayPattern(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BindingPatternKind",
                    variant: "ArrayPattern",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentPattern(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BindingPatternKind",
                    variant: "AssignmentPattern",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AssignmentPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AssignmentPattern",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ObjectPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ObjectPattern",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "properties",
                    ::oxc_quote_types::Node::Vec(
                        self.properties.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "rest",
                    ::oxc_quote_types::Node::Option(self.rest.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BindingProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BindingProperty",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("key", self.key.to_rust()),
                ("value", self.value.to_rust()),
                ("shorthand", self.shorthand.to_rust()),
                ("computed", self.computed.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ArrayPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ArrayPattern",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "elements",
                    ::oxc_quote_types::Node::Vec(
                        self.elements
                            .iter()
                            .map(|v| ::oxc_quote_types::Node::Option(
                                v.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                            ))
                            .collect()
                    )
                ),
                (
                    "rest",
                    ::oxc_quote_types::Node::Option(self.rest.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BindingRestElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BindingRestElement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Function<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Function",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type", self.r#type.to_rust()),
                (
                    "id",
                    ::oxc_quote_types::Node::Option(
                        self.id.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("generator", self.generator.to_rust()),
                ("async", self.r#async.to_rust()),
                ("declare", self.declare.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "this_param",
                    ::oxc_quote_types::Node::Option(self.this_param.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Option(self.return_type.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "body",
                    ::oxc_quote_types::Node::Option(self.body.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption),
                ("pure", self.pure.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for FunctionType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::FunctionDeclaration => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FunctionType",
                    variant: "FunctionDeclaration",
                    field: ::std::option::Option::None,
                }))
            }
            Self::FunctionExpression => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FunctionType",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TSDeclareFunction => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FunctionType",
                    variant: "TSDeclareFunction",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TSEmptyBodyFunctionExpression => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FunctionType",
                    variant: "TSEmptyBodyFunctionExpression",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for FormalParameters<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "FormalParameters",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("kind", self.kind.to_rust()),
                (
                    "items",
                    ::oxc_quote_types::Node::Vec(self.items.iter().map(|v| v.to_rust()).collect())
                ),
                (
                    "rest",
                    ::oxc_quote_types::Node::Option(self.rest.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for FormalParameter<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "FormalParameter",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "decorators",
                    ::oxc_quote_types::Node::Vec(
                        self.decorators.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("pattern", self.pattern.to_rust()),
                (
                    "accessibility",
                    ::oxc_quote_types::Node::Option(
                        self.accessibility.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("readonly", self.readonly.to_rust()),
                ("override", self.r#override.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for FormalParameterKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::FormalParameter => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FormalParameterKind",
                    variant: "FormalParameter",
                    field: ::std::option::Option::None,
                }))
            }
            Self::UniqueFormalParameters => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FormalParameterKind",
                    variant: "UniqueFormalParameters",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ArrowFormalParameters => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FormalParameterKind",
                    variant: "ArrowFormalParameters",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Signature => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "FormalParameterKind",
                    variant: "Signature",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for FunctionBody<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "FunctionBody",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "directives",
                    ::oxc_quote_types::Node::Vec(
                        self.directives.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "statements",
                    ::oxc_quote_types::Node::Vec(
                        self.statements.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ArrowFunctionExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ArrowFunctionExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                ("async", self.r#async.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Option(self.return_type.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("body", ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.body.to_rust()))),
                ("scope_id", ::oxc_quote_types::Node::CellOption),
                ("pure", self.pure.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for YieldExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "YieldExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("delegate", self.delegate.to_rust()),
                (
                    "argument",
                    ::oxc_quote_types::Node::Option(
                        self.argument.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Class<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Class",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type", self.r#type.to_rust()),
                (
                    "decorators",
                    ::oxc_quote_types::Node::Vec(
                        self.decorators.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "id",
                    ::oxc_quote_types::Node::Option(
                        self.id.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "super_class",
                    ::oxc_quote_types::Node::Option(
                        self.super_class.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "super_type_arguments",
                    ::oxc_quote_types::Node::Option(self.super_type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "implements",
                    ::oxc_quote_types::Node::Option(self.implements.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Vec(
                            v.iter().map(|v| v.to_rust()).collect(),
                        ))
                    }))
                ),
                ("body", ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.body.to_rust()))),
                ("abstract", self.r#abstract.to_rust()),
                ("declare", self.declare.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ClassType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ClassDeclaration => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassType",
                    variant: "ClassDeclaration",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ClassExpression => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassType",
                    variant: "ClassExpression",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ClassBody<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ClassBody",
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

impl ::oxc_quote_types::ToRust for ClassElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::StaticBlock(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassElement",
                    variant: "StaticBlock",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MethodDefinition(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassElement",
                    variant: "MethodDefinition",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PropertyDefinition(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassElement",
                    variant: "PropertyDefinition",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AccessorProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassElement",
                    variant: "AccessorProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIndexSignature(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ClassElement",
                    variant: "TSIndexSignature",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for MethodDefinition<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "MethodDefinition",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type", self.r#type.to_rust()),
                (
                    "decorators",
                    ::oxc_quote_types::Node::Vec(
                        self.decorators.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("key", self.key.to_rust()),
                (
                    "value",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.value.to_rust()))
                ),
                ("kind", self.kind.to_rust()),
                ("computed", self.computed.to_rust()),
                ("static", self.r#static.to_rust()),
                ("override", self.r#override.to_rust()),
                ("optional", self.optional.to_rust()),
                (
                    "accessibility",
                    ::oxc_quote_types::Node::Option(
                        self.accessibility.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for MethodDefinitionType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::MethodDefinition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionType",
                    variant: "MethodDefinition",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TSAbstractMethodDefinition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionType",
                    variant: "TSAbstractMethodDefinition",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for PropertyDefinition<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "PropertyDefinition",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type", self.r#type.to_rust()),
                (
                    "decorators",
                    ::oxc_quote_types::Node::Vec(
                        self.decorators.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("key", self.key.to_rust()),
                (
                    "value",
                    ::oxc_quote_types::Node::Option(
                        self.value.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("computed", self.computed.to_rust()),
                ("static", self.r#static.to_rust()),
                ("declare", self.declare.to_rust()),
                ("override", self.r#override.to_rust()),
                ("optional", self.optional.to_rust()),
                ("definite", self.definite.to_rust()),
                ("readonly", self.readonly.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "accessibility",
                    ::oxc_quote_types::Node::Option(
                        self.accessibility.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for PropertyDefinitionType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::PropertyDefinition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyDefinitionType",
                    variant: "PropertyDefinition",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TSAbstractPropertyDefinition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "PropertyDefinitionType",
                    variant: "TSAbstractPropertyDefinition",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for MethodDefinitionKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Constructor => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionKind",
                    variant: "Constructor",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Method => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionKind",
                    variant: "Method",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Get => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionKind",
                    variant: "Get",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Set => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "MethodDefinitionKind",
                    variant: "Set",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for PrivateIdentifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "PrivateIdentifier",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for StaticBlock<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "StaticBlock",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ModuleDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ImportDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "ImportDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportAllDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "ExportAllDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportDefaultDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "ExportDefaultDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExportNamedDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "ExportNamedDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSExportAssignment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "TSExportAssignment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNamespaceExportDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleDeclaration",
                    variant: "TSNamespaceExportDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AccessorPropertyType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::AccessorProperty => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AccessorPropertyType",
                    variant: "AccessorProperty",
                    field: ::std::option::Option::None,
                }))
            }
            Self::TSAbstractAccessorProperty => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AccessorPropertyType",
                    variant: "TSAbstractAccessorProperty",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AccessorProperty<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "AccessorProperty",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type", self.r#type.to_rust()),
                (
                    "decorators",
                    ::oxc_quote_types::Node::Vec(
                        self.decorators.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("key", self.key.to_rust()),
                (
                    "value",
                    ::oxc_quote_types::Node::Option(
                        self.value.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("computed", self.computed.to_rust()),
                ("static", self.r#static.to_rust()),
                ("definite", self.definite.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "accessibility",
                    ::oxc_quote_types::Node::Option(
                        self.accessibility.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("source", self.source.to_rust()),
                (
                    "options",
                    ::oxc_quote_types::Node::Vec(
                        self.options.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "phase",
                    ::oxc_quote_types::Node::Option(
                        self.phase.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "specifiers",
                    ::oxc_quote_types::Node::Option(self.specifiers.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Vec(
                            v.iter().map(|v| v.to_rust()).collect(),
                        ))
                    }))
                ),
                ("source", self.source.to_rust()),
                (
                    "phase",
                    ::oxc_quote_types::Node::Option(
                        self.phase.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "with_clause",
                    ::oxc_quote_types::Node::Option(self.with_clause.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("import_kind", self.import_kind.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportPhase {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Source => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportPhase",
                    variant: "Source",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Defer => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportPhase",
                    variant: "Defer",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ImportDeclarationSpecifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ImportSpecifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportDeclarationSpecifier",
                    variant: "ImportSpecifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportDefaultSpecifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportDeclarationSpecifier",
                    variant: "ImportDefaultSpecifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportNamespaceSpecifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportDeclarationSpecifier",
                    variant: "ImportNamespaceSpecifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ImportSpecifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportSpecifier",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("imported", self.imported.to_rust()),
                ("local", self.local.to_rust()),
                ("import_kind", self.import_kind.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportDefaultSpecifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportDefaultSpecifier",
            fields: ::std::vec![("span", self.span.to_rust()), ("local", self.local.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportNamespaceSpecifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportNamespaceSpecifier",
            fields: ::std::vec![("span", self.span.to_rust()), ("local", self.local.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for WithClause<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "WithClause",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("attributes_keyword", self.attributes_keyword.to_rust()),
                (
                    "with_entries",
                    ::oxc_quote_types::Node::Vec(
                        self.with_entries.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportAttribute<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ImportAttribute",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("key", self.key.to_rust()),
                ("value", self.value.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportAttributeKey<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportAttributeKey",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportAttributeKey",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ExportNamedDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ExportNamedDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "declaration",
                    ::oxc_quote_types::Node::Option(
                        self.declaration.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "specifiers",
                    ::oxc_quote_types::Node::Vec(
                        self.specifiers.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "source",
                    ::oxc_quote_types::Node::Option(
                        self.source.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("export_kind", self.export_kind.to_rust()),
                (
                    "with_clause",
                    ::oxc_quote_types::Node::Option(self.with_clause.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ExportDefaultDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ExportDefaultDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("exported", self.exported.to_rust()),
                ("declaration", self.declaration.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ExportAllDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ExportAllDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "exported",
                    ::oxc_quote_types::Node::Option(
                        self.exported.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("source", self.source.to_rust()),
                (
                    "with_clause",
                    ::oxc_quote_types::Node::Option(self.with_clause.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("export_kind", self.export_kind.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ExportSpecifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "ExportSpecifier",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("local", self.local.to_rust()),
                ("exported", self.exported.to_rust()),
                ("export_kind", self.export_kind.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ExportDefaultDeclarationKind<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::FunctionDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "FunctionDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ClassDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInterfaceDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSInterfaceDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ExportDefaultDeclarationKind",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for ModuleExportName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::IdentifierName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleExportName",
                    variant: "IdentifierName",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleExportName",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ModuleExportName",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for V8IntrinsicExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "V8IntrinsicExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                (
                    "arguments",
                    ::oxc_quote_types::Node::Vec(
                        self.arguments.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BooleanLiteral {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BooleanLiteral",
            fields: ::std::vec![("span", self.span.to_rust()), ("value", self.value.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for NullLiteral {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "NullLiteral",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for NumericLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "NumericLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("value", self.value.to_rust()),
                (
                    "raw",
                    ::oxc_quote_types::Node::Option(
                        self.raw.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("base", self.base.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for StringLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "StringLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("value", self.value.to_rust()),
                (
                    "raw",
                    ::oxc_quote_types::Node::Option(
                        self.raw.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("lossy", self.lossy.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for BigIntLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "BigIntLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("raw", self.raw.to_rust()),
                ("base", self.base.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for RegExpLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "RegExpLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("regex", self.regex.to_rust()),
                (
                    "raw",
                    ::oxc_quote_types::Node::Option(
                        self.raw.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for RegExp<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "RegExp",
            fields: ::std::vec![
                ("pattern", self.pattern.to_rust()),
                ("flags", self.flags.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for RegExpPattern<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Raw(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "RegExpPattern",
                    variant: "Raw",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::Invalid(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "RegExpPattern",
                    variant: "Invalid",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::Pattern(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "RegExpPattern",
                    variant: "Pattern",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXElement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "opening_element",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.opening_element.to_rust()
                    ))
                ),
                (
                    "closing_element",
                    ::oxc_quote_types::Node::Option(self.closing_element.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "children",
                    ::oxc_quote_types::Node::Vec(
                        self.children.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXOpeningElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXOpeningElement",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("self_closing", self.self_closing.to_rust()),
                ("name", self.name.to_rust()),
                (
                    "attributes",
                    ::oxc_quote_types::Node::Vec(
                        self.attributes.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXClosingElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXClosingElement",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXFragment<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXFragment",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("opening_fragment", self.opening_fragment.to_rust()),
                ("closing_fragment", self.closing_fragment.to_rust()),
                (
                    "children",
                    ::oxc_quote_types::Node::Vec(
                        self.children.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXOpeningFragment {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXOpeningFragment",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXClosingFragment {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXClosingFragment",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXElementName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXElementName",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXElementName",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NamespacedName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXElementName",
                    variant: "NamespacedName",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXElementName",
                    variant: "MemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXElementName",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXNamespacedName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXNamespacedName",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("namespace", self.namespace.to_rust()),
                ("name", self.name.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXMemberExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXMemberExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object", self.object.to_rust()),
                ("property", self.property.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXMemberExpressionObject<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXMemberExpressionObject",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXMemberExpressionObject",
                    variant: "MemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXMemberExpressionObject",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXExpressionContainer<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXExpressionContainer",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::EmptyExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "EmptyExpression",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NullLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "NullLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::RegExpLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "RegExpLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::MetaProperty(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "MetaProperty",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Super(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "Super",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrayExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ArrayExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ArrowFunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ArrowFunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AssignmentExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "AssignmentExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::AwaitExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "AwaitExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BinaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "BinaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::CallExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "CallExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ChainExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ChainExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ClassExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ClassExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ConditionalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ConditionalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::FunctionExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "FunctionExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ImportExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ImportExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::LogicalExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "LogicalExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NewExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "NewExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ObjectExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ObjectExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ParenthesizedExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ParenthesizedExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SequenceExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "SequenceExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TaggedTemplateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TaggedTemplateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ThisExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ThisExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UpdateExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "UpdateExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::YieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "YieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateInExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "PrivateInExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXElement(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "JSXElement",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSXFragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "JSXFragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAsExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TSAsExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSatisfiesExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TSSatisfiesExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeAssertion(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TSTypeAssertion",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNonNullExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TSNonNullExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInstantiationExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "TSInstantiationExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::V8IntrinsicExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "V8IntrinsicExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ComputedMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "ComputedMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StaticMemberExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "StaticMemberExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::PrivateFieldExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXExpression",
                    variant: "PrivateFieldExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXEmptyExpression {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXEmptyExpression",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXAttributeItem<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Attribute(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeItem",
                    variant: "Attribute",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::SpreadAttribute(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeItem",
                    variant: "SpreadAttribute",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXAttribute<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXAttribute",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
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

impl ::oxc_quote_types::ToRust for JSXSpreadAttribute<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXSpreadAttribute",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXAttributeName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeName",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NamespacedName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeName",
                    variant: "NamespacedName",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXAttributeValue<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeValue",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExpressionContainer(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeValue",
                    variant: "ExpressionContainer",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Element(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeValue",
                    variant: "Element",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Fragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXAttributeValue",
                    variant: "Fragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXIdentifier<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXIdentifier",
            fields: ::std::vec![("span", self.span.to_rust()), ("name", self.name.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXChild<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Text(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXChild",
                    variant: "Text",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Element(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXChild",
                    variant: "Element",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Fragment(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXChild",
                    variant: "Fragment",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::ExpressionContainer(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXChild",
                    variant: "ExpressionContainer",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::Spread(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "JSXChild",
                    variant: "Spread",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSXSpreadChild<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXSpreadChild",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSXText<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSXText",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("value", self.value.to_rust()),
                (
                    "raw",
                    ::oxc_quote_types::Node::Option(
                        self.raw.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSThisParameter<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSThisParameter",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("this_span", self.this_span.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSEnumDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSEnumDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "members",
                    ::oxc_quote_types::Node::Vec(
                        self.members.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                ("const", self.r#const.to_rust()),
                ("declare", self.declare.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSEnumMember<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSEnumMember",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "initializer",
                    ::oxc_quote_types::Node::Option(
                        self.initializer.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSEnumMemberName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSEnumMemberName",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::String(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSEnumMemberName",
                    variant: "String",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSTypeAnnotation<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeAnnotation",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSLiteralType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSLiteralType",
            fields: ::std::vec![("span", self.span.to_rust()), ("literal", self.literal.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::BooleanLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "BooleanLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::NumericLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "NumericLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::BigIntLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "BigIntLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TemplateLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "TemplateLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::UnaryExpression(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSLiteral",
                    variant: "UnaryExpression",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::TSAnyKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSAnyKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSBigIntKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSBigIntKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSBooleanKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSBooleanKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIntrinsicKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSIntrinsicKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNeverKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSNeverKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNullKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSNullKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNumberKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSNumberKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSObjectKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSObjectKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSStringKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSStringKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSymbolKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSSymbolKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUndefinedKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSUndefinedKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUnknownKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSUnknownKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSVoidKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSVoidKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSArrayType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSArrayType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSConditionalType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSConditionalType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSConstructorType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSConstructorType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSFunctionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSFunctionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSImportType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSImportType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIndexedAccessType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSIndexedAccessType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInferType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSInferType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIntersectionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSIntersectionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSLiteralType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSLiteralType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSMappedType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSMappedType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNamedTupleMember(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSNamedTupleMember",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTemplateLiteralType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTemplateLiteralType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSThisType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSThisType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTupleType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTupleType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTypeLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeOperatorType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTypeOperatorType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypePredicate(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTypePredicate",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeQuery(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTypeQuery",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSTypeReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUnionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSUnionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSParenthesizedType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "TSParenthesizedType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocNullableType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "JSDocNullableType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocNonNullableType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "JSDocNonNullableType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocUnknownType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSType",
                    variant: "JSDocUnknownType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSConditionalType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSConditionalType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("check_type", self.check_type.to_rust()),
                ("extends_type", self.extends_type.to_rust()),
                ("true_type", self.true_type.to_rust()),
                ("false_type", self.false_type.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSUnionType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSUnionType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "types",
                    ::oxc_quote_types::Node::Vec(self.types.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSIntersectionType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSIntersectionType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "types",
                    ::oxc_quote_types::Node::Vec(self.types.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSParenthesizedType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSParenthesizedType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeOperator<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeOperator",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("operator", self.operator.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeOperatorOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Keyof => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeOperatorOperator",
                    variant: "Keyof",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Unique => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeOperatorOperator",
                    variant: "Unique",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Readonly => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeOperatorOperator",
                    variant: "Readonly",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSArrayType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSArrayType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("element_type", self.element_type.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSIndexedAccessType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSIndexedAccessType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("object_type", self.object_type.to_rust()),
                ("index_type", self.index_type.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTupleType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTupleType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "element_types",
                    ::oxc_quote_types::Node::Vec(
                        self.element_types.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNamedTupleMember<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNamedTupleMember",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("element_type", self.element_type.to_rust()),
                ("label", self.label.to_rust()),
                ("optional", self.optional.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSOptionalType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSOptionalType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSRestType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSRestType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTupleElement<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::TSOptionalType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSOptionalType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSRestType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSRestType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSAnyKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSAnyKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSBigIntKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSBigIntKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSBooleanKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSBooleanKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIntrinsicKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSIntrinsicKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNeverKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSNeverKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNullKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSNullKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNumberKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSNumberKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSObjectKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSObjectKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSStringKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSStringKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSSymbolKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSSymbolKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUndefinedKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSUndefinedKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUnknownKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSUnknownKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSVoidKeyword(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSVoidKeyword",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSArrayType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSArrayType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSConditionalType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSConditionalType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSConstructorType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSConstructorType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSFunctionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSFunctionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSImportType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSImportType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIndexedAccessType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSIndexedAccessType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSInferType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSInferType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSIntersectionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSIntersectionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSLiteralType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSLiteralType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSMappedType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSMappedType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSNamedTupleMember(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSNamedTupleMember",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTemplateLiteralType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTemplateLiteralType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSThisType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSThisType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTupleType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTupleType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTypeLiteral",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeOperatorType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTypeOperatorType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypePredicate(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTypePredicate",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeQuery(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTypeQuery",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSTypeReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSTypeReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSUnionType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSUnionType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSParenthesizedType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "TSParenthesizedType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocNullableType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "JSDocNullableType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocNonNullableType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "JSDocNonNullableType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::JSDocUnknownType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTupleElement",
                    variant: "JSDocUnknownType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSAnyKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSAnyKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSStringKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSStringKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSBooleanKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSBooleanKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNumberKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNumberKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNeverKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNeverKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSIntrinsicKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSIntrinsicKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSUnknownKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSUnknownKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNullKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNullKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSUndefinedKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSUndefinedKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSVoidKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSVoidKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSSymbolKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSSymbolKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSThisType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSThisType",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSObjectKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSObjectKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSBigIntKeyword {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSBigIntKeyword",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeReference<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeReference",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_name", self.type_name.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeName",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::QualifiedName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeName",
                    variant: "QualifiedName",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSQualifiedName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSQualifiedName",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("left", self.left.to_rust()),
                ("right", self.right.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeParameterInstantiation<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeParameterInstantiation",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "params",
                    ::oxc_quote_types::Node::Vec(self.params.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeParameter<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeParameter",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                (
                    "constraint",
                    ::oxc_quote_types::Node::Option(
                        self.constraint.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "default",
                    ::oxc_quote_types::Node::Option(
                        self.default.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("in", self.r#in.to_rust()),
                ("out", self.out.to_rust()),
                ("const", self.r#const.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeParameterDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeParameterDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "params",
                    ::oxc_quote_types::Node::Vec(self.params.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeAliasDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeAliasDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("type_annotation", self.type_annotation.to_rust()),
                ("declare", self.declare.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSAccessibility {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Private => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSAccessibility",
                    variant: "Private",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Protected => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSAccessibility",
                    variant: "Protected",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Public => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSAccessibility",
                    variant: "Public",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSClassImplements<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSClassImplements",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSInterfaceDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSInterfaceDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "extends",
                    ::oxc_quote_types::Node::Option(self.extends.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Vec(
                            v.iter().map(|v| v.to_rust()).collect(),
                        ))
                    }))
                ),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("body", ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.body.to_rust()))),
                ("declare", self.declare.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSInterfaceBody<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSInterfaceBody",
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

impl ::oxc_quote_types::ToRust for TSPropertySignature<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSPropertySignature",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("computed", self.computed.to_rust()),
                ("optional", self.optional.to_rust()),
                ("readonly", self.readonly.to_rust()),
                ("key", self.key.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSSignature<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::TSIndexSignature(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSSignature",
                    variant: "TSIndexSignature",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSPropertySignature(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSSignature",
                    variant: "TSPropertySignature",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSCallSignatureDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSSignature",
                    variant: "TSCallSignatureDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSConstructSignatureDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSSignature",
                    variant: "TSConstructSignatureDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSMethodSignature(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSSignature",
                    variant: "TSMethodSignature",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSIndexSignature<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSIndexSignature",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "parameters",
                    ::oxc_quote_types::Node::Vec(
                        self.parameters.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.type_annotation.to_rust()
                    ))
                ),
                ("readonly", self.readonly.to_rust()),
                ("static", self.r#static.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSCallSignatureDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSCallSignatureDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "this_param",
                    ::oxc_quote_types::Node::Option(
                        self.this_param.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Option(self.return_type.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSMethodSignatureKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Method => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMethodSignatureKind",
                    variant: "Method",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Get => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMethodSignatureKind",
                    variant: "Get",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Set => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMethodSignatureKind",
                    variant: "Set",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSMethodSignature<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSMethodSignature",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("key", self.key.to_rust()),
                ("computed", self.computed.to_rust()),
                ("optional", self.optional.to_rust()),
                ("kind", self.kind.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "this_param",
                    ::oxc_quote_types::Node::Option(self.this_param.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Option(self.return_type.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSConstructSignatureDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSConstructSignatureDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Option(self.return_type.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSIndexSignatureName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSIndexSignatureName",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("name", self.name.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.type_annotation.to_rust()
                    ))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSInterfaceHeritage<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSInterfaceHeritage",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypePredicate<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypePredicate",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("parameter_name", self.parameter_name.to_rust()),
                ("asserts", self.asserts.to_rust()),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(self.type_annotation.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypePredicateName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypePredicateName",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::This(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypePredicateName",
                    variant: "This",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSModuleDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSModuleDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                (
                    "body",
                    ::oxc_quote_types::Node::Option(
                        self.body.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("kind", self.kind.to_rust()),
                ("declare", self.declare.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSModuleDeclarationKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Global => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationKind",
                    variant: "Global",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Module => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationKind",
                    variant: "Module",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Namespace => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationKind",
                    variant: "Namespace",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSModuleDeclarationName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Identifier(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationName",
                    variant: "Identifier",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
            Self::StringLiteral(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationName",
                    variant: "StringLiteral",
                    field: ::std::option::Option::Some(item.to_rust()),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSModuleDeclarationBody<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::TSModuleDeclaration(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationBody",
                    variant: "TSModuleDeclaration",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::TSModuleBlock(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleDeclarationBody",
                    variant: "TSModuleBlock",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSModuleBlock<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSModuleBlock",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "directives",
                    ::oxc_quote_types::Node::Vec(
                        self.directives.iter().map(|v| v.to_rust()).collect()
                    )
                ),
                (
                    "body",
                    ::oxc_quote_types::Node::Vec(self.body.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeLiteral<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeLiteral",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "members",
                    ::oxc_quote_types::Node::Vec(
                        self.members.iter().map(|v| v.to_rust()).collect()
                    )
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSInferType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSInferType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "type_parameter",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.type_parameter.to_rust()
                    ))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeQuery<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeQuery",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expr_name", self.expr_name.to_rust()),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeQueryExprName<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::TSImportType(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeQueryExprName",
                    variant: "TSImportType",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeQueryExprName",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::QualifiedName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSTypeQueryExprName",
                    variant: "QualifiedName",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSImportType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSImportType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("argument", self.argument.to_rust()),
                (
                    "options",
                    ::oxc_quote_types::Node::Option(self.options.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "qualifier",
                    ::oxc_quote_types::Node::Option(
                        self.qualifier.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "type_arguments",
                    ::oxc_quote_types::Node::Option(self.type_arguments.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                ("is_type_of", self.is_type_of.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSFunctionType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSFunctionType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "this_param",
                    ::oxc_quote_types::Node::Option(self.this_param.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.return_type.to_rust()
                    ))
                ),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSConstructorType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSConstructorType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("abstract", self.r#abstract.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Option(self.type_parameters.as_ref().map(|v| {
                        ::std::boxed::Box::new(::oxc_quote_types::Node::Box(
                            ::std::boxed::Box::new(v.to_rust()),
                        ))
                    }))
                ),
                (
                    "params",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(self.params.to_rust()))
                ),
                (
                    "return_type",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.return_type.to_rust()
                    ))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSMappedType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSMappedType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "type_parameter",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.type_parameter.to_rust()
                    ))
                ),
                (
                    "name_type",
                    ::oxc_quote_types::Node::Option(
                        self.name_type.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                (
                    "type_annotation",
                    ::oxc_quote_types::Node::Option(
                        self.type_annotation.as_ref().map(|v| ::std::boxed::Box::new(v.to_rust()))
                    )
                ),
                ("optional", self.optional.to_rust()),
                ("readonly", self.readonly.to_rust()),
                ("scope_id", ::oxc_quote_types::Node::CellOption)
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSMappedTypeModifierOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::True => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMappedTypeModifierOperator",
                    variant: "True",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Plus => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMappedTypeModifierOperator",
                    variant: "Plus",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Minus => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMappedTypeModifierOperator",
                    variant: "Minus",
                    field: ::std::option::Option::None,
                }))
            }
            Self::None => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSMappedTypeModifierOperator",
                    variant: "None",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSTemplateLiteralType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTemplateLiteralType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                (
                    "quasis",
                    ::oxc_quote_types::Node::Vec(self.quasis.iter().map(|v| v.to_rust()).collect())
                ),
                (
                    "types",
                    ::oxc_quote_types::Node::Vec(self.types.iter().map(|v| v.to_rust()).collect())
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSAsExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSAsExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSSatisfiesExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSSatisfiesExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSTypeAssertion<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSTypeAssertion",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                ("type_annotation", self.type_annotation.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSImportEqualsDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSImportEqualsDeclaration",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("id", self.id.to_rust()),
                ("module_reference", self.module_reference.to_rust()),
                ("import_kind", self.import_kind.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSModuleReference<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::ExternalModuleReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleReference",
                    variant: "ExternalModuleReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::IdentifierReference(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleReference",
                    variant: "IdentifierReference",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
            Self::QualifiedName(item) => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "TSModuleReference",
                    variant: "QualifiedName",
                    field: ::std::option::Option::Some(::oxc_quote_types::Node::Box(
                        ::std::boxed::Box::new(item.to_rust()),
                    )),
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for TSExternalModuleReference<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSExternalModuleReference",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNonNullExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNonNullExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for Decorator<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Decorator",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSExportAssignment<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSExportAssignment",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSNamespaceExportDeclaration<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSNamespaceExportDeclaration",
            fields: ::std::vec![("span", self.span.to_rust()), ("id", self.id.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for TSInstantiationExpression<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "TSInstantiationExpression",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("expression", self.expression.to_rust()),
                (
                    "type_parameters",
                    ::oxc_quote_types::Node::Box(::std::boxed::Box::new(
                        self.type_parameters.to_rust()
                    ))
                )
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for ImportOrExportKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Value => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportOrExportKind",
                    variant: "Value",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Type => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "ImportOrExportKind",
                    variant: "Type",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for JSDocNullableType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSDocNullableType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust()),
                ("postfix", self.postfix.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSDocNonNullableType<'_> {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSDocNonNullableType",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("type_annotation", self.type_annotation.to_rust()),
                ("postfix", self.postfix.to_rust())
            ],
        }))
    }
}

impl ::oxc_quote_types::ToRust for JSDocUnknownType {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "JSDocUnknownType",
            fields: ::std::vec![("span", self.span.to_rust())],
        }))
    }
}

impl ::oxc_quote_types::ToRust for CommentKind {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Line => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentKind",
                    variant: "Line",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Block => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentKind",
                    variant: "Block",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for CommentPosition {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Leading => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentPosition",
                    variant: "Leading",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Trailing => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentPosition",
                    variant: "Trailing",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for CommentAnnotation {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::None => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "None",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Jsdoc => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "Jsdoc",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Legal => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "Legal",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Pure => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "Pure",
                    field: ::std::option::Option::None,
                }))
            }
            Self::NoSideEffects => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "NoSideEffects",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Webpack => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "Webpack",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Vite => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "CommentAnnotation",
                    variant: "Vite",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for Comment {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
            name: "Comment",
            fields: ::std::vec![
                ("span", self.span.to_rust()),
                ("attached_to", self.attached_to.to_rust()),
                ("kind", self.kind.to_rust()),
                ("position", self.position.to_rust()),
                ("preceded_by_newline", self.preceded_by_newline.to_rust()),
                ("followed_by_newline", self.followed_by_newline.to_rust()),
                ("annotation", self.annotation.to_rust())
            ],
        }))
    }
}
