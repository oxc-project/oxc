//! # `jsquote!` and `jsquote_expr!` for OXC
//! **Don't use this crate directly;** use `oxc_quote` instead.
#![cfg_attr(oxc_quote_is_nightly, feature(proc_macro_diagnostic, proc_macro_span))]

extern crate proc_macro;

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::ParseOptions;
use oxc_quote_types::{Enum, Node, Struct, ToRust};
use oxc_span::SourceType;
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;
use syn::{
    Expr, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
    token::{Brace, Paren},
};

const PLACEHOLDER: &str = "__OXC__QUOTE__PLACEHOLDER____";

fn translate_ast_to_rust(
    node: Node,
    allocator: &Expr,
    span_stream: &Expr,
    stream: &mut TokenStream,
) {
    macro_rules! extend {
        ($($tt:tt)*) => {
            stream.extend(quote!{$($tt)*})
        }
    }

    match node {
        Node::U8(v) => extend! { #v },
        Node::U16(v) => extend! { #v },
        Node::U32(v) => extend! { #v },
        Node::U64(v) => extend! { #v },
        Node::U128(v) => extend! { #v },
        Node::I8(v) => extend! { #v },
        Node::I16(v) => extend! { #v },
        Node::I32(v) => extend! { #v },
        Node::I64(v) => extend! { #v },
        Node::I128(v) => extend! { #v },
        Node::F32(v) => extend! { #v },
        Node::F64(v) => extend! { #v },
        Node::Usize(v) => extend! { #v },
        Node::Isize(v) => extend! { #v },
        Node::Bool(v) => extend! { #v },
        Node::String(v) => extend! { #v },
        Node::Span(_) => {
            // It might seem strange, but we don't use the spans here at all.
            // We just substitute in the span expression.
            extend! { #span_stream };
        }
        Node::Atom(s) => {
            extend! { ::oxc_quote::private::Atom::from(#s) }
        }
        Node::Vec(v) => {
            let mut child_stream = TokenStream::new();
            let mut is_first = true;
            for node in v {
                let mut inner_stream = TokenStream::new();

                translate_ast_to_rust(node, allocator, span_stream, &mut inner_stream);

                if is_first {
                    child_stream.extend(quote! { {
                        use ::oxc_quote::private::ToChainIter;
                        (#inner_stream).to_chain().into_iter()
                    }});
                } else {
                    child_stream.extend(quote! { .chain({
                        use ::oxc_quote::private::ToChainIter;
                        (#inner_stream).to_chain().into_iter()
                    }) });
                }

                is_first = false;
            }

            if is_first {
                // No elements were there.
                extend! { ::oxc_quote::private::Vec::new_in(#allocator) };
            } else {
                extend! { ::oxc_quote::private::Vec::from_iter_in(#child_stream, #allocator) };
            }
        }
        Node::Box(v) => {
            extend! { ::oxc_quote::private::Box::new_in };
            let mut child_stream = TokenStream::new();
            translate_ast_to_rust(*v, allocator, span_stream, &mut child_stream);
            child_stream.extend(quote! { , #allocator });
            stream.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, child_stream))]);
        }
        Node::TryIntoUnwrap(v) => {
            let mut child_stream = TokenStream::new();
            translate_ast_to_rust(*v, allocator, span_stream, &mut child_stream);
            stream.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, child_stream))]);
            extend! { .try_into().unwrap() };
        }
        Node::Option(o) => {
            if let Some(v) = o {
                extend! { ::std::option::Option::Some };
                let mut child_stream = TokenStream::new();
                translate_ast_to_rust(*v, allocator, span_stream, &mut child_stream);
                stream.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, child_stream))]);
            } else {
                extend! { ::std::option::Option::None };
            }
        }
        Node::CellOption => {
            extend! { ::std::cell::Cell::new(None) };
        }
        Node::Cell(v) => {
            extend! { ::std::cell::Cell::new };
            let mut child_stream = TokenStream::new();
            translate_ast_to_rust(*v, allocator, span_stream, &mut child_stream);
            stream.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, child_stream))]);
        }
        Node::Enum(enm) => {
            // Special case: we have to search for `IdentifierReference`
            // structures that contain placeholder values and substitute
            // them back out with simple identifiers.
            if let Some(placeholder) = enm.to_placeholder(allocator) {
                extend! { #placeholder };
            } else {
                let Enum { name, variant, field } = *enm;

                let name = Ident::new(name, Span::call_site());
                let variant = Ident::new(variant, Span::call_site());

                extend! { ::oxc_quote::private::#name::#variant };

                if let Some(field) = field {
                    let mut child_stream = TokenStream::new();
                    translate_ast_to_rust(field, allocator, span_stream, &mut child_stream);
                    stream.extend([TokenTree::Group(Group::new(
                        Delimiter::Parenthesis,
                        child_stream,
                    ))]);
                }
            }
        }
        Node::Struct(strukt) => {
            // Special case: we have to search for `IdentifierReference`
            // structures that contain placeholder values and substitute
            // them back out with simple identifiers.
            if let Some(placeholder) = strukt.to_placeholder(allocator) {
                extend! { #placeholder };
            } else {
                let Struct { name, fields } = *strukt;
                let name = Ident::new(name, Span::call_site());
                extend! { ::oxc_quote::private::#name };

                if !fields.is_empty() {
                    let mut child_stream = TokenStream::new();

                    for (field_name, field) in fields {
                        let field_name = syn::Ident::new_raw(field_name, Span::call_site());

                        child_stream.extend(quote! { #field_name : });

                        translate_ast_to_rust(field, allocator, span_stream, &mut child_stream);

                        child_stream.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
                    }

                    stream.extend([TokenTree::Group(Group::new(Delimiter::Brace, child_stream))]);
                }
            }
        }
    }
}

trait ToPlaceholder {
    fn to_placeholder(&self, allocator: &Expr) -> Option<TokenStream>;
}

impl ToPlaceholder for Struct {
    fn to_placeholder(&self, allocator: &Expr) -> Option<TokenStream> {
        match self.name {
            "IdentifierReference" => {
                let (_, Node::Atom(name)) = self.fields.iter().find(|(name, _)| *name == "name")?
                else {
                    unreachable!();
                };

                let name = name.strip_prefix(PLACEHOLDER)?;
                let ident = Ident::new(name, Span::call_site());

                Some(quote! { #ident.clone_in(#allocator) })
            }
            "ExpressionStatement" => {
                let (_, Node::Enum(enm)) =
                    self.fields.iter().find(|(name, _)| *name == "expression")?
                else {
                    unreachable!();
                };

                enm.to_placeholder(allocator)
            }
            _ => None,
        }
    }
}

impl ToPlaceholder for Enum {
    fn to_placeholder(&self, allocator: &Expr) -> Option<TokenStream> {
        match (self.name, self.variant) {
            ("Expression", "Identifier") | ("Statement", "ExpressionStatement") => {
                if let Some(Node::Box(boxed)) = &self.field {
                    if let Node::Struct(ref strukt) = **boxed {
                        return strukt.to_placeholder(allocator);
                    }
                }
            }
            // This might be a naive assumption. But it works for now.
            (_, "Identifier") => {
                if let Some(Node::Box(boxed)) = &self.field {
                    if let Node::Struct(ref strukt) = **boxed {
                        let p = strukt.to_placeholder(allocator);
                        return Some(quote! { ::oxc_quote::private::Argument::from(#p) });
                    }
                }
            }
            (_, _) => {}
        }

        None
    }
}

struct InputArgs {
    alloc_expr: Expr,
    _comma: Token![,],
    span_expr: Expr,
    _comma2: Token![,],
    _block: Brace,
    body: TokenStream,
}

struct Input {
    _quote_macro: Ident,
    _bang: Token![!],
    _parens: Paren,
    args: InputArgs,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _quote_macro: input.parse()?,
            _bang: input.parse()?,
            _parens: parenthesized!(content in input),
            args: content.parse()?,
        })
    }
}

impl Parse for InputArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        Ok(Self {
            alloc_expr: input.parse()?,
            _comma: input.parse()?,
            span_expr: input.parse()?,
            _comma2: input.parse()?,
            _block: braced!(content in input),
            body: content.parse()?,
        })
    }
}

#[proc_macro]
pub fn jsquote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let InputArgs { body, .. } = syn::parse_macro_input!(input as InputArgs);

    match quotejs_inner(&body, false) {
        Ok(r) => r.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn jsquote_expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let InputArgs { body, .. } = syn::parse_macro_input!(input as InputArgs);

    match quotejs_inner(&body, true) {
        Ok(r) => r.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn quotejs_inner(tokens: &TokenStream, expression: bool) -> syn::Result<TokenStream> {
    // This is highly discouraged, and is only done here since it's absolutely necessary.
    // Proc-macros are meant only to work on "rusty" tokens, which mostly align with javascript's --
    // except for the allowed characters in identifiers, which in javascript can contain `$`, which
    // breaks Rust's tokenizer.
    let original_text = Span::call_site().source_text().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "cannot use `quotejs!` from within macros (source_text was empty)",
        )
    })?;

    if original_text.contains(PLACEHOLDER) {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("`quotejs!` input cannot contain the text '{PLACEHOLDER}'"),
        ));
    }

    let Input { args, .. } = syn::parse_str(&original_text)?;
    let InputArgs { alloc_expr, span_expr, .. } = args;

    let original_text = {
        // We use the hacked version because the Span accessor methods have been
        // in limbo for what feels like a decade, and under stable even with proc_macro2
        // they report zeroes. I'm tired of waiting.
        let base_offset = proc_macro::Span::call_site().hack_byte_range().start;

        let iter = tokens.clone().into_iter();
        let Some(start) =
            iter.clone().next().map(|t| t.span().unwrap().hack_byte_range().start - base_offset)
        else {
            return Err(syn::Error::new(
                Span::call_site(),
                "javascript expression cannot be empty",
            ));
        };
        let end = iter.last().unwrap().span().unwrap().hack_byte_range().end - base_offset;

        &original_text[start..end]
    };

    // Replace any placeholders.
    let mut iter = original_text.chars();
    let mut source = String::with_capacity(original_text.len() + (original_text.len() / 2));
    let mut offset = 0;
    while let Some(ch) = iter.next() {
        if ch == '#' {
            let next_ident = iter
                .clone()
                .take_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
                .collect::<String>();

            for _ in 0..next_ident.len() {
                iter.next().unwrap();
            }

            if next_ident.is_empty() {
                return Err(syn::Error::new(
                    get_nearest_span(tokens, offset),
                    "`#` token must be followed by a Rust-ey placeholder identifier",
                ));
            }

            source.push_str(&format!("{PLACEHOLDER}{next_ident}"));
            offset += next_ident.len() + 1;
        } else {
            offset += 1;
            source.push(ch);
        }
    }

    let alloc = Allocator::default();

    let parser =
        oxc_parser::Parser::new(&alloc, &source, SourceType::tsx()).with_options(ParseOptions {
            allow_return_outside_function: true,
            parse_regular_expression: true,
            preserve_parens: false,
            allow_v8_intrinsics: true,
        });

    let nodes = if expression {
        let parsed = match parser.parse_expression() {
            Ok(t) => t,
            Err(errs) => {
                check_parse_errors(tokens, &errs)?;
                // Just in case for some reason OXC returns an `Err()`
                // with no errors.
                return Err(syn::Error::new(Span::call_site(), "failed to parse Javascript"));
            }
        };

        parsed.to_rust()
    } else {
        let parsed = parser.parse();

        check_parse_errors(tokens, &parsed.errors)?;

        Node::Vec(parsed.program.body.iter().map(ToRust::to_rust).collect())
    };

    let mut stream = TokenStream::new();
    translate_ast_to_rust(nodes, &alloc_expr, &span_expr, &mut stream);
    Ok(stream)
}

// Emits errors during parsing; if `nightly` is enabled,
// uses the `Diagnostic` API.
fn check_parse_errors(tokens: &TokenStream, errors: &[OxcDiagnostic]) -> syn::Result<()> {
    if !errors.is_empty() {
        #[cfg(oxc_quote_is_nightly)]
        {
            for err in errors {
                let span = err
                    .labels
                    .as_ref()
                    .and_then(|l| l.first().map(oxc_diagnostics::LabeledSpan::offset))
                    .map_or_else(Span::call_site, |off| get_nearest_span(tokens, off))
                    .unwrap();

                let mut diag = proc_macro::Diagnostic::spanned(
                    span,
                    match err.severity {
                        oxc_diagnostics::Severity::Error => proc_macro::Level::Error,
                        oxc_diagnostics::Severity::Warning => proc_macro::Level::Warning,
                        oxc_diagnostics::Severity::Advice => proc_macro::Level::Note,
                    },
                    err.message.clone(),
                );

                if let Some(help) = &err.help {
                    diag = diag.help(help.to_string());
                }

                diag.emit();
            }

            // We do this to abort further operations; we have to return _something_.
            return Err(syn::Error::new(Span::call_site(), "failed to parse Javascript"));
        }

        #[cfg(not(oxc_quote_is_nightly))]
        {
            return Err(errors
                .iter()
                .map(|err| {
                    let span = err
                        .labels
                        .as_ref()
                        .and_then(|l| l.first().map(oxc_diagnostics::LabeledSpan::offset))
                        .map_or_else(Span::call_site, |off| get_nearest_span(tokens, off))
                        .unwrap();

                    syn::Error::new(span, err.message.clone())
                })
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .unwrap());
        }
    }
    Ok(())
}

/// **Very slow** function that should only be used in error situations.
///
/// Attempts to find the closest [`Span`] for which a given offset occurs.
/// Returns [`Span::call_site`] if no span could be found.
#[cold]
fn get_nearest_span(tokens: &TokenStream, mut offset: usize) -> Span {
    let mut iter = tokens.clone().into_iter();
    let Some(first) = iter.next() else {
        return Span::call_site();
    };
    offset += first.span().unwrap().hack_byte_range().start;

    let mut tkn = first;

    loop {
        let range = tkn.span().unwrap().hack_byte_range();

        if range.contains(&offset) {
            return tkn.span();
        }

        let Some(next) = iter.next() else {
            return Span::call_site();
        };

        tkn = next;
    }
}

// This is a nasty, nasty hack.
// Sorry :( Please don't use this in your own code.
// It exists to work around the lack of useful, developer-friendly
// accessor methods in spans, put off by a seemingly unending river
// of bikeshedding and arguing. It's been years, so this is what
// it ultimately comes down to.
trait SpanHack {
    fn hack_byte_range(&self) -> core::ops::Range<usize>;
}

impl SpanHack for proc_macro::Span {
    #[cfg_attr(oxc_quote_is_nightly, inline)]
    fn hack_byte_range(&self) -> core::ops::Range<usize> {
        #[cfg(oxc_quote_is_nightly)]
        {
            self.byte_range()
        }

        #[cfg(not(oxc_quote_is_nightly))]
        {
            // I am so, so sorry. But rust compiler members, PLEASE
            // finally do something about this. It's getting ridiculous how
            // inaccessible some of the interesting parts of the proc macro
            // world are. People are already doing cursed shit with them,
            // please just let us do it cleanly :(
            let formatted = format!("{self:?}");
            let (start, end) = formatted
                .split_once('(')
                .unwrap()
                .1
                .split_once(')')
                .unwrap()
                .0
                .split_once("..")
                .unwrap();
            let start: usize = start.parse().unwrap();
            let end: usize = end.parse().unwrap();
            start..end
        }
    }
}
