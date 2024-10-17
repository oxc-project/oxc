use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, Ident, LitFloat, LitInt, LitStr, Token};

use super::{
    declare_oxc_lint::rule_name_converter,
    util::{eat_comma, parse_assert},
};

pub struct SecretRuleMeta {
    struct_name: Ident,
    message: LitStr,
    entropy: Option<LitFloat>,
    min_len: Option<LitInt>,
    max_len: Option<LitInt>,
}

impl Parse for SecretRuleMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_name = input.parse()?;
        input.parse::<Token!(,)>()?;
        let description = input.parse()?;

        eat_comma(&input)?;

        let mut rule = SecretRuleMeta {
            struct_name,
            message: description,
            entropy: None,
            min_len: None,
            max_len: None,
        };

        while input.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            match ident.to_string().as_str() {
                "entropy" => {
                    input.parse::<Token!(=)>()?;
                    let entropy = input.parse::<LitFloat>()?;
                    parse_assert!(
                        entropy.base10_parse::<f32>()? >= 0.0,
                        entropy,
                        "Entropy must be greater than or equal to 0."
                    );
                    rule.entropy = Some(entropy);
                }
                "min_len" => {
                    input.parse::<Token!(=)>()?;
                    let min_len = input.parse::<LitInt>()?;
                    parse_assert!(
                        min_len.base10_parse::<u32>()? > 0,
                        min_len,
                        "Minimum length must be greater than or equal to 1."
                    );
                    rule.min_len = Some(min_len);
                }
                "max_len" => {
                    input.parse::<Token!(=)>()?;
                    let max_len = input.parse::<LitInt>()?;
                    parse_assert!(
                        max_len.base10_parse::<u32>()? > 0,
                        max_len,
                        "Maximum length cannot be zero."
                    );
                    rule.max_len = Some(max_len);
                }
                _ => parse_assert!(
                    false,
                    ident,
                    "Unexpected attribute. Only `entropy`, `min_len`, and `max_len` are allowed."
                ),
            }
            eat_comma(&input)?;
        }

        // Ignore the rest
        input.parse::<proc_macro2::TokenStream>()?;

        if let (Some(min), Some(max)) = (rule.min_len.as_ref(), &rule.max_len.as_ref()) {
            let min = min.base10_parse::<u32>()?;
            let max = max.base10_parse::<u32>()?;
            parse_assert!(
                min <= max,
                max,
                "Maximum length must be greater than or equal to minimum length."
            );
        }

        Ok(rule)
    }
}

pub fn declare_oxc_secret(meta: SecretRuleMeta) -> TokenStream {
    let SecretRuleMeta {
        //
        struct_name,
        message,
        entropy,
        min_len,
        max_len,
    } = meta;

    let rule_name = rule_name_converter().convert(struct_name.to_string());

    let min_len_fn = min_len.map(|min_len| {
        quote! {
            #[inline]
            fn min_len(&self) -> NonZeroU32 {
                // SAFETY: #min_len is a valid value for NonZeroU32
                unsafe { NonZeroU32::new_unchecked(#min_len) }
            }
        }
    });

    let max_len_fn = max_len.map(|max_len| {
        quote! {
            #[inline]
            fn max_len(&self) -> Option<NonZeroU32> {
                Some(unsafe { NonZeroU32::new_unchecked(#max_len) })
            }
        }
    });

    let entropy_fn = entropy.map(|entropy| {
        quote! {
            #[inline]
            fn min_entropy(&self) -> f32 {
                #entropy
            }
        }
    });

    let output = quote! {
        impl super::SecretScannerMeta for #struct_name {
            #[inline]
            fn rule_name(&self) -> &'static str {
                #rule_name
            }

            #[inline]
            fn message(&self) -> &'static str {
                #message
            }

            #min_len_fn

            #max_len_fn

            #entropy_fn
        }
    };

    TokenStream::from(output)
}
