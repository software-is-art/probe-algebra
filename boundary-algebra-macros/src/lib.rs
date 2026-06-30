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
use proc_macro2::Literal;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Fields, FnArg, Ident, Item, ItemFn, ItemMod,
    LitStr, ReturnType, Token, Type, Visibility,
};

/// `#[algebra(Marker, "display name")]` — generate a WHOLE discovery `Theory` from a module of
/// ordinary operator functions. No `theory!` block is written: the macro reads each PUBLIC function's
/// signature — arity → fixity, the value types → the sorts, the name → the symbol — and emits the
/// operator table, the sort(s), `sort_of`, identity `observe`, and the shadow-derived grid. So the
/// agent authors only the value objects and the operator functions; the algebra is what they mean.
///
/// MULTI-SORTED: operators may range over several `#[derive(Shaped)]` value types (e.g. `Date` and
/// `Duration`). The macro synthesises a `Value` sum over the sorts and the `sort_of` that tags it. The
/// module's PUBLIC functions are its operators; private functions are helpers, left untouched.
#[proc_macro_attribute]
pub fn algebra(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AlgebraArgs { marker, name } = parse_macro_input!(attr as AlgebraArgs);
    let mut module = parse_macro_input!(item as ItemMod);
    let content = match &mut module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(&module, "#[algebra] needs an inline module body")
                .to_compile_error()
                .into();
        }
    };

    // the module's PUBLIC functions are the operators; everything else (the value enums, private
    // helpers, `use`s) is left untouched.
    let mut ops: Vec<OpInfo> = Vec::new();
    for it in content.iter() {
        let Item::Fn(f) = it else { continue };
        if !matches!(f.vis, Visibility::Public(_)) {
            continue;
        }
        match operator_info(f) {
            Some(op) => ops.push(op),
            None => {
                return syn::Error::new_spanned(
                    &f.sig,
                    "#[algebra]: a public function must be an operator — every argument and the \
                     return must be a named value type (make helpers private)",
                )
                .to_compile_error()
                .into();
            }
        }
    }
    if ops.is_empty() {
        return syn::Error::new_spanned(&module, "#[algebra] found no public operator functions")
            .to_compile_error()
            .into();
    }

    // the sorts are the distinct value types appearing in the signatures, in first-seen order.
    let sorts = distinct_sorts(&ops);
    let generated = if sorts.len() == 1 {
        single_sort_impl(&marker, &name, &ops, &sorts[0])
    } else {
        multi_sort_impl(&marker, &name, &ops, &sorts)
    };

    let parsed: syn::File = syn::parse2(generated).expect("generated algebra items parse");
    content.extend(parsed.items);
    quote! { #module }.into()
}

/// One operator read off a function: its name, the (named) argument types, and the return type.
struct OpInfo {
    name: Ident,
    args: Vec<Type>,
    ret: Type,
}

/// `#[algebra(Marker, "display name")]`.
struct AlgebraArgs {
    marker: Ident,
    name: LitStr,
}
impl Parse for AlgebraArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let marker = input.parse()?;
        input.parse::<Token![,]>()?;
        let name = input.parse()?;
        Ok(AlgebraArgs { marker, name })
    }
}

/// Read an operator off a function: every argument and the return must be a NAMED (path) type — a
/// receiver or a non-path type means it is not an operator over value objects.
fn operator_info(f: &ItemFn) -> Option<OpInfo> {
    let ret = match &f.sig.output {
        ReturnType::Type(_, ty) => (**ty).clone(),
        ReturnType::Default => return None,
    };
    if !matches!(ret, Type::Path(_)) {
        return None;
    }
    let mut args = Vec::new();
    for arg in &f.sig.inputs {
        let FnArg::Typed(pt) = arg else {
            return None;
        };
        let ty = (*pt.ty).clone();
        if !matches!(ty, Type::Path(_)) {
            return None;
        }
        args.push(ty);
    }
    Some(OpInfo {
        name: f.sig.ident.clone(),
        args,
        ret,
    })
}

/// The distinct value types across every operator's signature, in first-seen order.
fn distinct_sorts(ops: &[OpInfo]) -> Vec<Type> {
    let mut seen: Vec<Type> = Vec::new();
    for op in ops {
        for ty in op.args.iter().chain(std::iter::once(&op.ret)) {
            if !seen.iter().any(|t| type_str(t) == type_str(ty)) {
                seen.push(ty.clone());
            }
        }
    }
    seen
}

fn type_str(t: &Type) -> String {
    quote! { #t }.to_string()
}

/// The variant identifier a value type contributes to the synthesised `Value`/`Sort` enums — its
/// last path segment (`crate::date::Duration` → `Duration`).
fn variant_of(ty: &Type) -> Ident {
    match ty {
        Type::Path(tp) => tp.path.segments.last().unwrap().ident.clone(),
        _ => format_ident!("Sort"),
    }
}

fn fixity_tokens(arity: usize) -> TokenStream2 {
    match arity {
        0 => quote! { Nullary },
        1 => quote! { Prefix },
        _ => quote! { Infix },
    }
}

/// SINGLE-SORT: the value type IS the engine `Value`; one sort, identity observation, shadow grid.
fn single_sort_impl(
    marker: &Ident,
    name: &LitStr,
    ops: &[OpInfo],
    value_ty: &Type,
) -> TokenStream2 {
    let sort = format_ident!("{}Sort", marker);
    let mut wrappers = Vec::new();
    let mut entries = Vec::new();
    for op in ops {
        let wname = format_ident!("__op_{}", op.name);
        let fname = &op.name;
        let arity = op.args.len();
        let clones: Vec<TokenStream2> = (0..arity)
            .map(|i| {
                let idx = Literal::usize_unsuffixed(i);
                quote! { __v[#idx].clone() }
            })
            .collect();
        wrappers.push(quote! {
            fn #wname(__v: &[#value_ty]) -> ::std::option::Option<#value_ty> {
                ::std::option::Option::Some(#fname(#(#clones),*))
            }
        });
        let fixity = fixity_tokens(arity);
        let inputs: Vec<TokenStream2> = (0..arity).map(|_| quote! { #sort::Only }).collect();
        let sym = op.name.to_string();
        entries.push(quote! {
            crate::discover::engine::Operator {
                name: #sym,
                symbol: #sym,
                fixity: crate::discover::engine::Fixity::#fixity,
                inputs: ::std::vec![ #(#inputs),* ],
                output: #sort::Only,
                eval: #wname,
            }
        });
    }
    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum #sort {
            Only,
        }
        pub struct #marker;
        impl crate::discover::engine::Theory for #marker {
            type Sort = #sort;
            type Value = #value_ty;
            type Obs = #value_ty;
            fn name() -> &'static str { #name }
            fn operators() -> ::std::vec::Vec<crate::discover::engine::Operator<Self>> {
                ::std::vec![ #(#entries),* ]
            }
            fn inhabitants(_sort: Self::Sort) -> ::std::vec::Vec<Self::Value> {
                crate::discover::engine::shadow_grid::<#value_ty>(24)
            }
            fn sort_of(_v: &Self::Value) -> Self::Sort { #sort::Only }
            fn observe(v: &Self::Value) -> Self::Obs { ::std::clone::Clone::clone(v) }
        }
        #(#wrappers)*
    }
}

/// MULTI-SORT: synthesise a `Value` SUM over the sorts (`Value::Date(Date) | Value::Duration(..)`)
/// and a matching `Sort` enum; each operator wrapper pattern-matches its argument sorts. The grid per
/// sort is the shadow grid of that type, wrapped into the sum.
fn multi_sort_impl(marker: &Ident, name: &LitStr, ops: &[OpInfo], sorts: &[Type]) -> TokenStream2 {
    let value_enum = format_ident!("{}Value", marker);
    let sort_enum = format_ident!("{}Sort", marker);
    let variants: Vec<Ident> = sorts.iter().map(variant_of).collect();

    let value_variants = sorts
        .iter()
        .zip(&variants)
        .map(|(ty, v)| quote! { #v(#ty) });
    let sort_of_arms = variants
        .iter()
        .map(|v| quote! { #value_enum::#v(_) => #sort_enum::#v });
    let inhab_arms = sorts.iter().zip(&variants).map(|(ty, v)| {
        quote! {
            #sort_enum::#v => crate::discover::engine::shadow_grid::<#ty>(12)
                .into_iter()
                .map(#value_enum::#v)
                .collect()
        }
    });

    let mut wrappers = Vec::new();
    let mut entries = Vec::new();
    for op in ops {
        let wname = format_ident!("__op_{}", op.name);
        let fname = &op.name;
        let arity = op.args.len();
        let ret_v = variant_of(&op.ret);
        let binds: Vec<Ident> = (0..arity).map(|i| format_ident!("__a{}", i)).collect();
        let pats = op.args.iter().zip(&binds).map(|(ty, b)| {
            let v = variant_of(ty);
            quote! { #value_enum::#v(#b) }
        });
        let calls = binds.iter().map(|b| quote! { #b.clone() });
        let body = if arity == 0 {
            quote! { ::std::option::Option::Some(#value_enum::#ret_v(#fname())) }
        } else {
            quote! {
                match __v {
                    [ #(#pats),* ] => ::std::option::Option::Some(
                        #value_enum::#ret_v(#fname( #(#calls),* ))
                    ),
                    _ => ::std::option::Option::None,
                }
            }
        };
        wrappers.push(quote! {
            fn #wname(__v: &[#value_enum]) -> ::std::option::Option<#value_enum> { #body }
        });
        let fixity = fixity_tokens(arity);
        let inputs = op.args.iter().map(|ty| {
            let v = variant_of(ty);
            quote! { #sort_enum::#v }
        });
        let sym = op.name.to_string();
        entries.push(quote! {
            crate::discover::engine::Operator {
                name: #sym,
                symbol: #sym,
                fixity: crate::discover::engine::Fixity::#fixity,
                inputs: ::std::vec![ #(#inputs),* ],
                output: #sort_enum::#ret_v,
                eval: #wname,
            }
        });
    }

    quote! {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum #value_enum { #(#value_variants),* }
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum #sort_enum { #(#variants),* }
        pub struct #marker;
        impl crate::discover::engine::Theory for #marker {
            type Sort = #sort_enum;
            type Value = #value_enum;
            type Obs = #value_enum;
            fn name() -> &'static str { #name }
            fn operators() -> ::std::vec::Vec<crate::discover::engine::Operator<Self>> {
                ::std::vec![ #(#entries),* ]
            }
            fn inhabitants(__sort: Self::Sort) -> ::std::vec::Vec<Self::Value> {
                match __sort { #(#inhab_arms),* }
            }
            fn sort_of(__v: &Self::Value) -> Self::Sort {
                match __v { #(#sort_of_arms),* }
            }
            fn observe(__v: &Self::Value) -> Self::Obs { ::std::clone::Clone::clone(__v) }
        }
        #(#wrappers)*
    }
}

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
