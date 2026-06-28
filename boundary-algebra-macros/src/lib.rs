//! boundary-algebra-macros — the derive side of the grammar.
//!
//! `#[derive(Shaped)]` reads a value object's STRUCTURE and emits its probe surface, so the
//! "structure supplies the test surface" thesis is mechanized rather than hand-written:
//!
//!   - `inhabitant()` — one canonical seed, the first variant / the struct built from each
//!     field's own inhabitant (recursion bottoms out because a leaf variant is chosen); and
//!   - `perturbation_classes()` — one neighbour-GROUP per degree of freedom: the
//!     variant-swap group (STRUCTURAL — every other constructor) and one group per field
//!     (VALUE / deep, the field's own perturbations threaded back through the constructor).
//!
//! Feeding those classes to the fused `sensitive_to_all` probe in `boundary` gives a single
//! derived operator sensitive to structure, value, AND (through the recursive field groups)
//! semantics — the universal probe. Leaves with smart-constructor invariants (`Int`,
//! `Ident`) impl `Shaped` by hand; everything composite is derived.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident};

#[proc_macro_derive(Shaped)]
pub fn derive_shaped(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (inhabitant, classes) = match &input.data {
        Data::Enum(data) => enum_body(name, data),
        Data::Struct(data) => struct_body(name, &data.fields),
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "Shaped cannot be derived for a union")
                .to_compile_error()
                .into();
        }
    };
    // The DERIVED degree-of-freedom set: one `Field<Self, I>` per variant (enum) or field
    // (struct), folded into a `DofCons` list. So `HasDofs` is mechanical and `Complete<Self>`
    // covers it by construction — completeness cannot be under-specified.
    let dof_count = match &input.data {
        Data::Enum(data) => data.variants.len(),
        Data::Struct(data) => data.fields.len(),
        Data::Union(_) => 0,
    };
    let mut dofs = quote! { crate::boundary::DofNil };
    for i in (0..dof_count).rev() {
        let idx = proc_macro2::Literal::usize_unsuffixed(i);
        dofs = quote! { crate::boundary::DofCons<crate::boundary::Field<#name, #idx>, #dofs> };
    }
    quote! {
        impl crate::boundary::Shaped for #name {
            fn inhabitant() -> Self {
                #inhabitant
            }
            fn perturbation_classes(&self) -> ::std::vec::Vec<::std::vec::Vec<Self>> {
                let mut __classes: ::std::vec::Vec<::std::vec::Vec<Self>> = ::std::vec::Vec::new();
                #classes
                __classes
            }
        }
        impl crate::boundary::HasDofs for #name {
            type Dofs = #dofs;
        }
    }
    .into()
}

/// Build a constructor that fills every field with `<FieldTy>::inhabitant()`.
fn inhabitant_ctor(path: &TokenStream2, fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Unit => quote! { #path },
        Fields::Unnamed(f) => {
            let vals = f.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote! { <#ty as crate::boundary::Shaped>::inhabitant() }
            });
            quote! { #path( #(#vals),* ) }
        }
        Fields::Named(f) => {
            let vals = f.named.iter().map(|field| {
                let id = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! { #id: <#ty as crate::boundary::Shaped>::inhabitant() }
            });
            quote! { #path { #(#vals),* } }
        }
    }
}

/// The binding identifiers for a field set (the field's own name, or `f0`, `f1`, ... for a
/// tuple), and the pattern that binds them by reference.
fn binders(fields: &Fields) -> (Vec<Ident>, TokenStream2) {
    match fields {
        Fields::Unit => (vec![], quote! {}),
        Fields::Unnamed(f) => {
            let ids: Vec<Ident> = (0..f.unnamed.len())
                .map(|i| format_ident!("f{}", i))
                .collect();
            let pat = quote! { ( #(ref #ids),* ) };
            (ids, pat)
        }
        Fields::Named(f) => {
            let ids: Vec<Ident> = f
                .named
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .collect();
            let pat = quote! { { #(ref #ids),* } };
            (ids, pat)
        }
    }
}

/// Rebuild a constructor where field `replace` is `__n` and every other field is cloned
/// from its binding.
fn rebuild(path: &TokenStream2, fields: &Fields, ids: &[Ident], replace: usize) -> TokenStream2 {
    match fields {
        Fields::Unit => quote! { #path },
        Fields::Unnamed(_) => {
            let args = ids.iter().enumerate().map(|(i, id)| {
                if i == replace {
                    quote! { __n.clone() }
                } else {
                    quote! { #id.clone() }
                }
            });
            quote! { #path( #(#args),* ) }
        }
        Fields::Named(_) => {
            let args = ids.iter().enumerate().map(|(i, id)| {
                if i == replace {
                    quote! { #id: __n.clone() }
                } else {
                    quote! { #id: #id.clone() }
                }
            });
            quote! { #path { #(#args),* } }
        }
    }
}

/// The per-field perturbation classes: for each field, the constructor with that field
/// replaced by each of its own perturbations (siblings cloned). Reused for structs and for
/// each enum variant arm.
fn field_class_pushes(path: &TokenStream2, fields: &Fields, ids: &[Ident]) -> TokenStream2 {
    let pushes = ids.iter().enumerate().map(|(i, id)| {
        let rebuilt = rebuild(path, fields, ids, i);
        quote! {
            {
                let mut __c: ::std::vec::Vec<Self> = ::std::vec::Vec::new();
                for __n in crate::boundary::Shaped::all_perturbations(#id) {
                    __c.push(#rebuilt);
                }
                __classes.push(__c);
            }
        }
    });
    quote! { #(#pushes)* }
}

fn struct_body(name: &Ident, fields: &Fields) -> (TokenStream2, TokenStream2) {
    let path = quote! { #name };
    let inhabitant = inhabitant_ctor(&path, fields);
    let (ids, pat) = binders(fields);
    let pushes = field_class_pushes(&path, fields, &ids);
    let classes = match fields {
        Fields::Unit => quote! {},
        _ => quote! {
            let Self #pat = self;
            #pushes
        },
    };
    (inhabitant, classes)
}

fn enum_body(name: &Ident, data: &DataEnum) -> (TokenStream2, TokenStream2) {
    let first = data
        .variants
        .first()
        .expect("Shaped cannot be derived for an empty enum");
    let first_path = {
        let v = &first.ident;
        quote! { #name::#v }
    };
    let inhabitant = inhabitant_ctor(&first_path, &first.fields);

    // The variant-swap class: every variant's inhabitant, minus the one equal to `self`.
    let all_variants = data.variants.iter().map(|v| {
        let vid = &v.ident;
        inhabitant_ctor(&quote! { #name::#vid }, &v.fields)
    });

    // Per-variant arms producing the field classes for whichever variant `self` is.
    let arms = data.variants.iter().map(|v| {
        let vid = &v.ident;
        let path = quote! { #name::#vid };
        let (ids, pat) = binders(&v.fields);
        let pushes = field_class_pushes(&path, &v.fields, &ids);
        quote! { #name::#vid #pat => { #pushes } }
    });

    let classes = quote! {
        let __variants: ::std::vec::Vec<Self> = ::std::vec![ #(#all_variants),* ];
        __classes.push(__variants.into_iter().filter(|__v| __v != self).collect());
        match self {
            #(#arms)*
        }
    };
    (inhabitant, classes)
}
