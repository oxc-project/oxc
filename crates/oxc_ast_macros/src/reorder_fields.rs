use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemStruct};

const DUMMY: &str = "DUMMY";

#[inline]
pub fn reorder_fields(ty: &mut ItemStruct, data: &[u8]) {
    let (Fields::Named(FieldsNamed { named: fields, .. })
    | Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })) = &mut ty.fields
    else {
        debug_assert!(false, "Entered unreachable code!");
        // SAFETY: We don't generate any ordering data for empty structs, And the debug assertions
        // are on in CI runs; The debug assertion above would ensure any possible mistake gets caught
        // by tests early on in the PR's life span. This allows us to avoid a branch here.
        unsafe { std::hint::unreachable_unchecked() }
    };

    // TODO: We can replace this with uninitialized memory, It might be faster if we use one byte
    // to check if a field is placeholder or not and keep the rest of the bytes uninitialized as we
    // never read them. I'm not sure if it is safe with a mutable reference or not but I believe it
    // would be safe with exclusive ownership of the field.
    let mut pick = Field {
        attrs: Vec::default(),
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: Some(format_ident!("{DUMMY}")),
        colon_token: None,
        ty: syn::Type::Verbatim(TokenStream::default()),
    };
    // TODO: use bit array here?
    let mut is_ordered = vec![false; fields.len()];
    let mut target;
    // Best case O(n), Worst case O(2n)
    for i in 0..fields.len() {
        if is_ordered[i] {
            continue;
        }

        let field = &mut fields[i];
        // `pick` the first unordered field
        pick = std::mem::replace(field, pick);
        // capture its ideal position
        target = data[i];

        // while we have something in our `pick`
        while pick.ident.as_ref().is_some_and(|it| it != DUMMY) {
            // select the slot of target position
            let field = &mut fields[target as usize];
            // put the picked field in the target slot and pick the previous item
            pick = std::mem::replace(field, pick);
            // mark the field as ordered
            is_ordered[target as usize] = true;
            // capture the ideal position of our new pick
            target = data[target as usize];
        }
    }
}
