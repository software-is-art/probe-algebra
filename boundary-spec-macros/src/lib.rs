//! boundary-spec-macros — the derive side of the grammar.
//!
//! `#[derive(Shaped)]` reads a value object's STRUCTURE and emits its probe surface, so the
//! "structure supplies the test surface" thesis is mechanized rather than hand-written:
//!
//!   - `inhabitant()` — one canonical seed: for an enum, the first LEAF variant (the first whose
//!     field types don't mention the enum itself, so direct self-recursion bottoms out; indirect
//!     recursion through another type is not detected — see `enum_body`); for a struct, the
//!     constructor filled with each field's own inhabitant; and
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
///
/// DECLARED EXPECTATIONS (optional third argument): `#[algebra(CreditMeter, "credit meter",
/// expects(commutative(grant), identity(grant, zero), bias_later(renew)))]` additionally
/// generates the `discover::expect::Expected` impl — the theory's declared laws, each a ratified
/// catalog shape applied to operator symbols (an identifier per operator function; a string
/// literal is also accepted). `Distance::of::<CreditMeter>()` then reports the gap between the
/// declaration and what discovery finds; a shape name outside the catalog fails loudly, by name,
/// the first time the expectations are read. No `expects`, no impl — nothing else changes.
#[proc_macro_attribute]
pub fn algebra(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AlgebraArgs {
        marker,
        name,
        expects,
    } = parse_macro_input!(attr as AlgebraArgs);
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
    // Two operators sharing a name would emit two `__op_<name>` wrappers — surface the clash
    // here, at the second function, rather than as a duplicate-definition error in generated code.
    for (i, a) in ops.iter().enumerate() {
        if let Some(b) = ops[i + 1..].iter().find(|b| b.name == a.name) {
            return syn::Error::new_spanned(
                &b.name,
                format!(
                    "#[algebra]: duplicate operator `{}` — operator functions must have \
                     distinct names",
                    a.name
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    // the sorts are the distinct value types appearing in the signatures, in first-seen order.
    let sorts = distinct_sorts(&ops);
    // DISTINCT sorts (dedup'd by FULL path) become variants named by their LAST segment, so
    // `foo::Date` and `bar::Date` would collide inside the synthesised enums — an opaque error
    // deep in generated code. Name the clash at the signature that introduced it instead.
    if let Some(err) = sort_name_collision(&sorts) {
        return err.to_compile_error().into();
    }
    let mut generated = if sorts.len() == 1 {
        single_sort_impl(&marker, &name, &ops, &sorts[0])
    } else {
        multi_sort_impl(&marker, &name, &ops, &sorts)
    };
    if let Some(decls) = &expects {
        generated.extend(expected_impl(&marker, decls));
    }

    let parsed: syn::File = match syn::parse2(generated) {
        Ok(file) => file,
        Err(e) => {
            // an internal macro bug, but reported as a diagnostic rather than a panic — the
            // user sees WHERE and WHY instead of a proc-macro abort.
            return syn::Error::new(
                e.span(),
                format!("#[algebra]: internal error — the generated items failed to parse: {e}"),
            )
            .to_compile_error()
            .into();
        }
    };
    content.extend(parsed.items);
    quote! { #module }.into()
}

/// The last-segment clash among DISTINCT sorts, if any: `distinct_sorts` dedups by the FULL type
/// path, but the synthesised `Value`/`Sort` enums name variants by the LAST segment only
/// (`variant_of`), so two different sorts ending in the same identifier cannot coexist.
fn sort_name_collision(sorts: &[Type]) -> Option<syn::Error> {
    for (i, a) in sorts.iter().enumerate() {
        for b in &sorts[i + 1..] {
            if variant_of(a) == variant_of(b) {
                return Some(syn::Error::new_spanned(
                    b,
                    format!(
                        "#[algebra]: sorts `{}` and `{}` collide on the variant name `{}` — the \
                         synthesised Value/Sort enums need distinct last path segments (rename \
                         one type, or introduce a type alias)",
                        type_str(a).replace(' ', ""),
                        type_str(b).replace(' ', ""),
                        variant_of(a),
                    ),
                ));
            }
        }
    }
    None
}

/// One operator read off a function: its name, the (named) argument types, and the return type.
struct OpInfo {
    name: Ident,
    args: Vec<Type>,
    ret: Type,
}

/// `#[algebra(Marker, "display name")]`, optionally
/// `#[algebra(Marker, "display name", expects(shape(op, ...), ...))]`.
struct AlgebraArgs {
    marker: Ident,
    name: LitStr,
    expects: Option<Vec<ExpectDecl>>,
}
impl Parse for AlgebraArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let marker = input.parse()?;
        input.parse::<Token![,]>()?;
        let name = input.parse()?;
        let mut expects = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                let keyword: Ident = input.parse()?;
                if keyword != "expects" {
                    return Err(syn::Error::new_spanned(
                        keyword,
                        "#[algebra]: after the display name, the only further argument is \
                         `expects(shape(op, ...), ...)`",
                    ));
                }
                let inner;
                syn::parenthesized!(inner in input);
                let decls = inner.parse_terminated(ExpectDecl::parse, Token![,])?;
                expects = Some(decls.into_iter().collect());
            }
        }
        Ok(AlgebraArgs {
            marker,
            name,
            expects,
        })
    }
}

/// One declared law inside `expects(...)`: a catalog shape key applied to operator symbols —
/// `identity(grant, zero)`. Shape-name validity is the LIBRARY's call (its `Expectation::of`
/// panics by name on an unratified shape), so this crate never duplicates the catalog.
struct ExpectDecl {
    shape: Ident,
    ops: Vec<String>,
}
impl Parse for ExpectDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let shape: Ident = input.parse()?;
        let inner;
        syn::parenthesized!(inner in input);
        let ops = inner
            .parse_terminated(ExpectOp::parse, Token![,])?
            .into_iter()
            .map(|op| op.0)
            .collect();
        Ok(ExpectDecl { shape, ops })
    }
}

/// An operator symbol in a declared law: the operator function's identifier (`grant`), or a
/// string literal for a symbol no identifier can spell.
struct ExpectOp(String);
impl Parse for ExpectOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(ExpectOp(input.parse::<LitStr>()?.value()))
        } else {
            Ok(ExpectOp(input.parse::<Ident>()?.to_string()))
        }
    }
}

/// The `expects(...)` expansion: the marker's `Expected` impl, one `Expectation` per declared
/// law. Symbols are the operator function names — exactly what the synthesised operator table
/// uses as `symbol`, so declared and discovered speak the same names.
fn expected_impl(marker: &Ident, decls: &[ExpectDecl]) -> TokenStream2 {
    let entries = decls.iter().map(|decl| {
        let shape = decl.shape.to_string();
        let ops = decl.ops.iter();
        quote! {
            ::boundary_spec::discover::expect::Expectation::of(
                #shape,
                ::std::vec![ #( #ops ),* ],
            )
        }
    });
    quote! {
        impl ::boundary_spec::discover::expect::Expected for #marker {
            fn expectations() -> ::std::vec::Vec<::boundary_spec::discover::expect::Expectation> {
                ::std::vec![ #( #entries ),* ]
            }
        }
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
            ::boundary_spec::discover::engine::Operator {
                name: #sym,
                symbol: #sym,
                fixity: ::boundary_spec::discover::engine::Fixity::#fixity,
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
        impl ::boundary_spec::discover::engine::Theory for #marker {
            type Sort = #sort;
            type Value = #value_ty;
            type Obs = #value_ty;
            fn name() -> &'static str { #name }
            fn operators() -> ::std::vec::Vec<::boundary_spec::discover::engine::Operator<Self>> {
                ::std::vec![ #(#entries),* ]
            }
            fn inhabitants(_sort: Self::Sort) -> ::std::vec::Vec<Self::Value> {
                ::boundary_spec::discover::engine::shadow_grid::<#value_ty>(24)
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
            #sort_enum::#v => ::boundary_spec::discover::engine::shadow_grid::<#ty>(12)
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
            ::boundary_spec::discover::engine::Operator {
                name: #sym,
                symbol: #sym,
                fixity: ::boundary_spec::discover::engine::Fixity::#fixity,
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
        impl ::boundary_spec::discover::engine::Theory for #marker {
            type Sort = #sort_enum;
            type Value = #value_enum;
            type Obs = #value_enum;
            fn name() -> &'static str { #name }
            fn operators() -> ::std::vec::Vec<::boundary_spec::discover::engine::Operator<Self>> {
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
        Data::Enum(data) => match enum_body(name, data) {
            Ok(parts) => parts,
            Err(e) => return e.to_compile_error().into(),
        },
        Data::Struct(data) => struct_body(name, &data.fields),
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "Shaped cannot be derived for a union")
                .to_compile_error()
                .into();
        }
    };
    // The STRUCTURAL slice: variant swaps (here and, threaded through fields, anywhere
    // below) — the type-level-finite partition `shadow_grid` closes over exhaustively
    // before spending budget on value neighbours.
    let structural = match &input.data {
        Data::Enum(data) => enum_structural(name, data),
        Data::Struct(data) => struct_structural(name, &data.fields),
        Data::Union(_) => unreachable!("unions were rejected above"),
    };
    // The DERIVED degree-of-freedom set: one `Field<Self, I>` per variant (enum) or field
    // (struct), folded into a `DofCons` list. So `HasDofs` is mechanical and `Complete<Self>`
    // covers it by construction — completeness cannot be under-specified.
    let dof_count = match &input.data {
        Data::Enum(data) => data.variants.len(),
        Data::Struct(data) => data.fields.len(),
        Data::Union(_) => 0,
    };
    let mut dofs = quote! { ::boundary_spec::boundary::DofNil };
    for i in (0..dof_count).rev() {
        let idx = proc_macro2::Literal::usize_unsuffixed(i);
        dofs = quote! { ::boundary_spec::boundary::DofCons<::boundary_spec::boundary::Field<#name, #idx>, #dofs> };
    }
    quote! {
        impl ::boundary_spec::boundary::Shaped for #name {
            fn inhabitant() -> Self {
                #inhabitant
            }
            fn perturbation_classes(&self) -> ::std::vec::Vec<::std::vec::Vec<Self>> {
                let mut __classes: ::std::vec::Vec<::std::vec::Vec<Self>> = ::std::vec::Vec::new();
                #classes
                __classes
            }
            fn structural_perturbations(&self) -> ::std::vec::Vec<Self> {
                let mut __structural: ::std::vec::Vec<Self> = ::std::vec::Vec::new();
                #structural
                __structural
            }
        }
        impl ::boundary_spec::boundary::HasDofs for #name {
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
                quote! { <#ty as ::boundary_spec::boundary::Shaped>::inhabitant() }
            });
            quote! { #path( #(#vals),* ) }
        }
        Fields::Named(f) => {
            let vals = f.named.iter().map(|field| {
                let id = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! { #id: <#ty as ::boundary_spec::boundary::Shaped>::inhabitant() }
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
                for __n in ::boundary_spec::boundary::Shaped::all_perturbations(#id) {
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

/// The per-field STRUCTURAL pushes: the constructor with one field replaced by each of that
/// field's own structural perturbations (siblings cloned) — how a variant swap anywhere
/// BELOW threads back up through this constructor. The value-side sibling of
/// `field_class_pushes`.
fn structural_field_pushes(path: &TokenStream2, fields: &Fields, ids: &[Ident]) -> TokenStream2 {
    let pushes = ids.iter().enumerate().map(|(i, id)| {
        let rebuilt = rebuild(path, fields, ids, i);
        quote! {
            for __n in ::boundary_spec::boundary::Shaped::structural_perturbations(#id) {
                __structural.push(#rebuilt);
            }
        }
    });
    quote! { #(#pushes)* }
}

/// A struct's structural perturbations: no variant of its own to swap, so exactly the
/// field-threaded swaps from below.
fn struct_structural(name: &Ident, fields: &Fields) -> TokenStream2 {
    let path = quote! { #name };
    let (ids, pat) = binders(fields);
    let pushes = structural_field_pushes(&path, fields, &ids);
    match fields {
        Fields::Unit => quote! {},
        _ => quote! {
            let Self #pat = self;
            #pushes
        },
    }
}

/// An enum's structural perturbations: every OTHER variant's inhabitant (the swap at this
/// level), plus the current variant rebuilt around each field's own structural
/// perturbations (swaps below, threaded up).
fn enum_structural(name: &Ident, data: &DataEnum) -> TokenStream2 {
    let all_variants = data.variants.iter().map(|v| {
        let vid = &v.ident;
        inhabitant_ctor(&quote! { #name::#vid }, &v.fields)
    });
    let arms = data.variants.iter().map(|v| {
        let vid = &v.ident;
        let path = quote! { #name::#vid };
        let (ids, pat) = binders(&v.fields);
        let pushes = structural_field_pushes(&path, &v.fields, &ids);
        quote! { #name::#vid #pat => { #pushes } }
    });
    quote! {
        let __variants: ::std::vec::Vec<Self> = ::std::vec![ #(#all_variants),* ];
        for __v in __variants {
            if &__v != self {
                __structural.push(__v);
            }
        }
        match self {
            #(#arms)*
        }
    }
}

/// True iff `ty` mentions `name` anywhere in its tokens — a SYNTACTIC recursion probe. It sees
/// through wrappers (`Box<E>`, `Vec<E>`, `(E, u8)`), but it cannot see INDIRECT recursion through
/// another type (`Rec(Wrapper)` where `Wrapper` privately holds an `E`) — that residual case
/// still recurses at runtime, and only a semantic (post-resolution) analysis could catch it.
fn mentions(ty: &Type, name: &Ident) -> bool {
    fn walk(ts: TokenStream2, name: &str) -> bool {
        ts.into_iter().any(|tt| match tt {
            proc_macro2::TokenTree::Ident(i) => i == name,
            proc_macro2::TokenTree::Group(g) => walk(g.stream(), name),
            _ => false,
        })
    }
    walk(quote! { #ty }, &name.to_string())
}

fn enum_body(name: &Ident, data: &DataEnum) -> syn::Result<(TokenStream2, TokenStream2)> {
    // The seed must come from a LEAF variant or `inhabitant()` never bottoms out: on
    // `enum E { Rec(Box<E>), Leaf }` the FIRST variant's inhabitant would need an `E` to build an
    // `E` — infinite recursion. So pick the first variant none of whose field types mention the
    // enum itself (see `mentions` for the syntactic limit), falling back to the first variant
    // when every variant self-references (that enum has no finite inhabitant anyway).
    let seed = data
        .variants
        .iter()
        .find(|v| v.fields.iter().all(|f| !mentions(&f.ty, name)))
        .or_else(|| data.variants.first())
        .ok_or_else(|| {
            syn::Error::new(
                name.span(),
                "Shaped cannot be derived for an empty enum — it has no inhabitant",
            )
        })?;
    let seed_path = {
        let v = &seed.ident;
        quote! { #name::#v }
    };
    let inhabitant = inhabitant_ctor(&seed_path, &seed.fields);

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
    Ok((inhabitant, classes))
}

/// `#[mutate]` / `#[mutate("label")]` — MUTANT SCHEMATA: every operator-flip mutant of
/// this function is compiled into the binary behind a runtime selector, so the whole
/// mutant population costs ONE build and each verdict is a test run, not a rebuild.
///
/// Each `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||` in the body becomes
/// `if Schemata::active("<site>") { <negated> } else { <original> }`, each `!`
/// gains a deletion mutant, and the function gains one DEAFNESS mutant per
/// constructible form of its return type (`Result<A, B>` → `Ok(default)` and
/// `Err(default)` where the payloads are recognisable, `Option` → `None`, `bool` →
/// both constants, `Vec`/`String` → empty) — the whole-body replacement class, read
/// from the return TYPE syntax; an unrecognised type simply grows no deaf form.
/// Every site registers itself into the census slice
/// (`discover::schemata::MUTANT_SITES`) the schemata lock freezes. Site ids are
/// `<label>:<index>: <op> -> <flip>` in body visit order (deaf forms:
/// `<label>:deaf -> <value>`); the label defaults to the function name — pass one
/// wherever the bare name is ambiguous (three types have a `judge`). Evaluation
/// order is preserved: the left operand binds once (so `&&`/`||` stay lazy in the
/// right operand and left-leaning chains grow linearly, not exponentially),
/// comparison operands bind by reference. Nested items are not descended into (a
/// const's comparison must stay const), and macro invocations (`matches!`,
/// `assert!`) are opaque tokens — their interiors stay uninstrumented, disclosed
/// here rather than silently.
#[proc_macro_attribute]
pub fn mutate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let label: Option<LitStr> = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as LitStr))
    };
    // a free fn, an impl method, or a WHOLE impl block (every method instrumented,
    // labelled `<Type>::<method>` — coverage grows per block, not per function).
    // DUAL EMISSION: the instrumented copy exists only under `feature = "schemata"`;
    // every other build carries the original item, byte for byte — zero overhead,
    // which is what lets hot paths (the engine) be instrumented at all. The sweep
    // builds once with the feature and pays the branches there alone.
    if let Ok(original) = syn::parse::<ItemFn>(item.clone()) {
        let mut f = original.clone();
        let label = label
            .map(|l| l.value())
            .unwrap_or_else(|| f.sig.ident.to_string());
        let sites = mutate_body(&label, &f.sig.output.clone(), &mut f.block);
        let regs = registrations(&sites);
        quote! {
            #regs
            #[cfg(feature = "schemata")]
            #f
            #[cfg(not(feature = "schemata"))]
            #original
        }
        .into()
    } else if let Ok(original) = syn::parse::<syn::ImplItemFn>(item.clone()) {
        // a lone method cannot host module-scope statics, so its registrations stay
        // in-body — which is only census-stable for NON-generic methods; instrument a
        // generic method through its impl block instead.
        let mut f = original.clone();
        let label = label
            .map(|l| l.value())
            .unwrap_or_else(|| f.sig.ident.to_string());
        let sites = mutate_body(&label, &f.sig.output.clone(), &mut f.block);
        for (i, site) in sites.iter().enumerate() {
            let ident = format_ident!("__MUTANT_SITE_{i}");
            let registration: syn::Stmt = syn::parse_quote! {
                #[::linkme::distributed_slice(crate::discover::schemata::MUTANT_SITES)]
                static #ident: &'static str =
                    ::core::concat!(::core::module_path!(), "::", #site);
            };
            f.block.stmts.insert(i, registration);
        }
        quote! {
            #[cfg(feature = "schemata")]
            #f
            #[cfg(not(feature = "schemata"))]
            #original
        }
        .into()
    } else if let Ok(original) = syn::parse::<syn::ItemImpl>(item) {
        let mut block = original.clone();
        let type_label = label.map(|l| l.value()).unwrap_or_else(|| {
            let ty = &block.self_ty;
            let tokens = quote!(#ty).to_string();
            tokens.replace(' ', "")
        });
        let mut sites = Vec::new();
        for item in &mut block.items {
            if let syn::ImplItem::Fn(method) = item {
                let label = format!("{type_label}::{}", method.sig.ident);
                sites.extend(mutate_body(
                    &label,
                    &method.sig.output.clone(),
                    &mut method.block,
                ));
            }
        }
        let regs = registrations(&sites);
        quote! {
            #regs
            #[cfg(feature = "schemata")]
            #block
            #[cfg(not(feature = "schemata"))]
            #original
        }
        .into()
    } else {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[mutate] applies to a function, a method, or an impl block",
        )
        .to_compile_error()
        .into()
    }
}

/// Rewrite a function body's flip sites and prepend the deafness prologue (one early
/// return per constructible form of the return type). Returns the site ids; the
/// CALLER emits their census registrations at MODULE scope — a static inside a
/// generic fn only materialises when the fn is instantiated, which would make the
/// census depend on which binary happened to monomorphise what.
fn mutate_body(label: &str, output: &ReturnType, block: &mut syn::Block) -> Vec<String> {
    let mut mutator = FlipMutator {
        label: label.to_string(),
        count: 0,
        sites: Vec::new(),
    };
    syn::visit_mut::VisitMut::visit_block_mut(&mut mutator, block);
    let mut sites = mutator.sites;
    let mut prologue: Vec<syn::Stmt> = Vec::new();
    if let ReturnType::Type(_, ty) = output {
        for (desc, value) in deaf_values(ty) {
            let site = format!("{label}:deaf -> {desc}");
            sites.push(site.clone());
            prologue.push(syn::parse_quote! {
                if crate::discover::schemata::Schemata::active(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    #site
                )) {
                    return #value;
                }
            });
        }
    }
    for stmt in prologue.into_iter().rev() {
        block.stmts.insert(0, stmt);
    }
    sites
}

/// The module-scope census registrations for one expansion's sites — names carry a
/// content hash so two impl blocks of the same type in one module cannot collide.
fn registrations(sites: &[String]) -> TokenStream2 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    sites.hash(&mut hasher);
    let tag = hasher.finish();
    let statics = sites.iter().enumerate().map(|(i, site)| {
        let ident = format_ident!("__MUTANT_SITE_{tag:016x}_{i}");
        quote! {
            #[cfg(feature = "schemata")]
            #[::linkme::distributed_slice(crate::discover::schemata::MUTANT_SITES)]
            static #ident: &'static str =
                ::core::concat!(::core::module_path!(), "::", #site);
        }
    });
    quote! { #(#statics)* }
}

/// The constructible deafness values of a return type, by SYNTAX — the same move the
/// whole-body replacement class of source-level mutation makes, done at expansion.
/// Unrecognised types return no forms: a deaf mutant that cannot be constructed is
/// not emitted, never guessed.
fn deaf_values(ty: &Type) -> Vec<(String, TokenStream2)> {
    let Type::Path(path) = ty else {
        return Vec::new();
    };
    let Some(segment) = path.path.segments.last() else {
        return Vec::new();
    };
    let type_args = || -> Vec<&Type> {
        match &segment.arguments {
            syn::PathArguments::AngleBracketed(a) => a
                .args
                .iter()
                .filter_map(|g| match g {
                    syn::GenericArgument::Type(t) => Some(t),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    };
    match segment.ident.to_string().as_str() {
        "bool" => vec![
            ("true".to_string(), quote!(true)),
            ("false".to_string(), quote!(false)),
        ],
        "String" => vec![(
            "String::new()".to_string(),
            quote!(::std::string::String::new()),
        )],
        "Vec" => vec![("vec![]".to_string(), quote!(::std::vec::Vec::new()))],
        "Option" => vec![("None".to_string(), quote!(::std::option::Option::None))],
        "Result" => {
            let args = type_args();
            let mut out = Vec::new();
            if let Some(ok_ty) = args.first() {
                if let Some((desc, value)) = deaf_values(ok_ty).into_iter().next() {
                    out.push((
                        format!("Ok({desc})"),
                        quote!(::std::result::Result::Ok(#value)),
                    ));
                }
            }
            if let Some(err_ty) = args.get(1) {
                if let Some((desc, value)) = deaf_values(err_ty).into_iter().next() {
                    out.push((
                        format!("Err({desc})"),
                        quote!(::std::result::Result::Err(#value)),
                    ));
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

/// The expression folder: children first (so a chain's inner sites are numbered
/// before the outer), then the four flips. Nested items are deliberately skipped.
struct FlipMutator {
    label: String,
    count: usize,
    sites: Vec<String>,
}

impl syn::visit_mut::VisitMut for FlipMutator {
    fn visit_item_mut(&mut self, _: &mut Item) {
        // a nested item (const, fn, static) keeps its original text: a comparison in
        // a const context cannot become a runtime branch.
    }

    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);

        // `!`-DELETION: the negation drops, its operand (already folded) stands bare.
        if let syn::Expr::Unary(unary) = expr {
            if matches!(unary.op, syn::UnOp::Not(_)) {
                let site = format!("{}:{}: ! -> (deleted)", self.label, self.count);
                self.count += 1;
                self.sites.push(site.clone());
                let inner = (*unary.expr).clone();
                // bind the NEGATED value once (normalising `&bool` operands through
                // `Not`); deletion is then a second negation — both arms one type.
                *expr = syn::parse_quote!({
                    let __mutant_v = !(#inner);
                    if crate::discover::schemata::Schemata::active(::core::concat!(
                        ::core::module_path!(),
                        "::",
                        #site
                    )) {
                        !__mutant_v
                    } else {
                        __mutant_v
                    }
                });
                return;
            }
        }

        let syn::Expr::Binary(binary) = expr else {
            return;
        };
        // each operator's single strongest alternate: the NEGATION for equality and
        // ordering (`<` -> `>=` disturbs both the strict/inclusive edge and the
        // direction), the dual for the lazy connectives.
        let (orig, flip): (&str, &str) = match binary.op {
            syn::BinOp::Eq(_) => ("==", "!="),
            syn::BinOp::Ne(_) => ("!=", "=="),
            syn::BinOp::Lt(_) => ("<", ">="),
            syn::BinOp::Le(_) => ("<=", ">"),
            syn::BinOp::Gt(_) => (">", "<="),
            syn::BinOp::Ge(_) => (">=", "<"),
            syn::BinOp::And(_) => ("&&", "||"),
            syn::BinOp::Or(_) => ("||", "&&"),
            _ => return,
        };
        let site = format!("{}:{}: {} -> {}", self.label, self.count, orig, flip);
        self.count += 1;
        self.sites.push(site.clone());
        let left = (*binary.left).clone();
        let right = (*binary.right).clone();
        let op = binary.op;
        let flipped: syn::BinOp = match binary.op {
            syn::BinOp::Eq(_) => syn::parse_quote!(!=),
            syn::BinOp::Ne(_) => syn::parse_quote!(==),
            syn::BinOp::Lt(_) => syn::parse_quote!(>=),
            syn::BinOp::Le(_) => syn::parse_quote!(>),
            syn::BinOp::Gt(_) => syn::parse_quote!(<=),
            syn::BinOp::Ge(_) => syn::parse_quote!(<),
            syn::BinOp::And(_) => syn::parse_quote!(||),
            _ => syn::parse_quote!(&&),
        };
        *expr = match binary.op {
            // lazy connectives: the left operand was eager anyway, so it binds once;
            // the right stays a lazy expression in both branches.
            syn::BinOp::And(_) | syn::BinOp::Or(_) => syn::parse_quote!({
                let __mutant_l = #left;
                if crate::discover::schemata::Schemata::active(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    #site
                )) {
                    __mutant_l #flipped (#right)
                } else {
                    __mutant_l #op (#right)
                }
            }),
            // equality and ordering: bind BOTH operands by reference, once — `&A op
            // &B` derefs to the underlying comparison, and neither side is moved or
            // re-evaluated.
            _ => syn::parse_quote!({
                let __mutant_l = &(#left);
                let __mutant_r = &(#right);
                if crate::discover::schemata::Schemata::active(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    #site
                )) {
                    __mutant_l #flipped __mutant_r
                } else {
                    __mutant_l #op __mutant_r
                }
            }),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enum_data(di: &DeriveInput) -> &DataEnum {
        match &di.data {
            Data::Enum(data) => data,
            _ => panic!("test input must be an enum"),
        }
    }

    /// A FIRST-recursive enum must seed from its leaf variant, or the generated `inhabitant()`
    /// recurses forever (`E::Rec` needs an `E` to build an `E`).
    #[test]
    fn a_first_recursive_enum_seeds_from_a_leaf_variant() {
        let di: DeriveInput = syn::parse_quote! { enum E { Rec(Box<E>), Leaf } };
        let (inhabitant, _) = enum_body(&di.ident, enum_data(&di)).expect("derivable");
        let src = inhabitant.to_string();
        assert!(src.contains("Leaf"), "must seed from the leaf, got: {src}");
        assert!(!src.contains("Rec"), "must not seed recursively: {src}");
    }

    /// A non-recursive enum keeps the ORIGINAL behaviour: the first variant is the seed.
    #[test]
    fn a_plain_enum_still_seeds_from_its_first_variant() {
        let di: DeriveInput = syn::parse_quote! { enum Op { Add, Mul, Lt } };
        let (inhabitant, _) = enum_body(&di.ident, enum_data(&di)).expect("derivable");
        assert!(inhabitant.to_string().contains("Add"));
    }

    /// The syntactic probe looks INSIDE wrappers and named-field variants, not just at the
    /// outermost type: `Rec { next: Vec<Box<E>> }` is recursive, `Wrap(Vec<u8>)` is not.
    #[test]
    fn the_recursion_probe_sees_through_nesting() {
        let di: DeriveInput = syn::parse_quote! {
            enum E { Rec { next: Vec<Box<E>> }, Wrap(Vec<u8>) }
        };
        let (inhabitant, _) = enum_body(&di.ident, enum_data(&di)).expect("derivable");
        assert!(inhabitant.to_string().contains("Wrap"));
    }

    /// Every variant self-referencing falls BACK to the first variant (documented residual: such
    /// an enum has no finite inhabitant, and indirect recursion is likewise not detected).
    #[test]
    fn an_all_recursive_enum_falls_back_to_the_first_variant() {
        let di: DeriveInput = syn::parse_quote! { enum E { A(Box<E>), B(Box<E>) } };
        let (inhabitant, _) = enum_body(&di.ident, enum_data(&di)).expect("derivable");
        assert!(inhabitant.to_string().contains("A"));
    }

    /// An EMPTY enum is a clean diagnostic, not a proc-macro panic.
    #[test]
    fn an_empty_enum_is_a_diagnostic_not_a_panic() {
        let di: DeriveInput = syn::parse_quote! { enum Never {} };
        let err = enum_body(&di.ident, enum_data(&di)).expect_err("empty enums are rejected");
        assert!(err.to_string().contains("empty enum"), "got: {err}");
    }

    /// Two distinct sorts sharing a LAST path segment are named as a clash (the synthesised
    /// enums would otherwise emit duplicate variants — an opaque error in generated code).
    #[test]
    fn colliding_sort_names_are_reported() {
        let sorts: Vec<Type> = vec![
            syn::parse_quote! { foo::Date },
            syn::parse_quote! { Duration },
            syn::parse_quote! { bar::Date },
        ];
        let err = sort_name_collision(&sorts).expect("the Date/Date clash must be caught");
        let msg = err.to_string();
        assert!(
            msg.contains("foo::Date") && msg.contains("bar::Date") && msg.contains("`Date`"),
            "the clash must name both sorts and the shared variant, got: {msg}"
        );
        assert!(
            sort_name_collision(&sorts[..2]).is_none(),
            "distinct last segments must not be flagged"
        );
    }
}
