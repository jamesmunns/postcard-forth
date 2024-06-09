use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, ImplGenerics, TypeGenerics, WhereClause,
};

pub fn do_derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let span = input.span();
    let name = input.ident;

    // Add a bound `T: Schema` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = generate_type(
        &input.data,
        span,
        name.to_string(),
        name.clone(),
        impl_generics,
        ty_generics,
        where_clause,
    )
    .unwrap_or_else(syn::Error::into_compile_error);

    expanded.into()
}

fn generate_type(
    data: &Data,
    span: Span,
    _name: String,
    tyident: syn::Ident,
    impl_generics: ImplGenerics,
    ty_generics: TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> Result<TokenStream, syn::Error> {
    match data {
        Data::Struct(data) => {
            let ty = generate_struct(tyident.clone(), &data.fields);
            Ok(quote! {
                unsafe impl #impl_generics ::postcard_forth::Serialize for #tyident #ty_generics #where_clause {
                    const FIELDS: &'static [::postcard_forth::SerField] = &[
                        #ty
                    ];
                }
            })
        }
        Data::Enum(data) => {
            let serfunc_name = format!("ser_{}", tyident);
            let sername_ident = syn::Ident::new(&serfunc_name, tyident.span());
            let mut arms = TokenStream::new();
            for (i, var) in data.variants.iter().enumerate() {
                let ident = &var.ident;
                let fields = generate_arm(&var.fields, tyident.clone(), ident, i as u32);
                arms.extend(quote! {
                    #fields
                });
            }

            let out = quote! {
                #[allow(non_snake_case)]
                #[inline]
                pub unsafe fn #sername_ident(stream: &mut ::postcard_forth::SerStream, base: core::ptr::NonNull<()>) -> Result<(), ()> {
                    let eref = base.cast::<#tyident>().as_ref();
                    match eref {
                        #arms
                    }
                }

                unsafe impl ::postcard_forth::Serialize for #tyident {
                    const FIELDS: &'static [::postcard_forth::SerField] = &[::postcard_forth::SerField {
                        offset: 0,
                        func: #sername_ident,
                    }];
                }
            };
            Ok(out)
        }
        Data::Union(_) => Err(syn::Error::new(
            span,
            "unions are not supported by `postcard::experimental::schema`",
        )),
    }
}

fn generate_struct(tyname: syn::Ident, fields: &Fields) -> TokenStream {
    let mut out = TokenStream::new();

    match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let ty = &f.ty;
                let name = &f.ident;
                let tystr = quote!( #ty ).to_string();

                // This is probably not sound. Users could shadow real names with other types, breaking
                // the safety guarantees.
                let serf = match tystr.as_str() {
                    // "u8" => quote!(::postcard_forth::impls::ser_u8),
                    // "u16" => quote!(::postcard_forth::impls::ser_u16),
                    // "u32" => quote!(::postcard_forth::impls::ser_u32),
                    // "u64" => quote!(::postcard_forth::impls::ser_u64),
                    // "u128" => quote!(::postcard_forth::impls::ser_u128),
                    // "usize" => quote!(::postcard_forth::impls::ser_usize),
                    // "i8" => quote!(::postcard_forth::impls::ser_i8),
                    // "i16" => quote!(::postcard_forth::impls::ser_i16),
                    // "i32" => quote!(::postcard_forth::impls::ser_i32),
                    // "i64" => quote!(::postcard_forth::impls::ser_i64),
                    // "i128" => quote!(::postcard_forth::impls::ser_i128),
                    // "isize" => quote!(::postcard_forth::impls::ser_isize),
                    // "String" => quote!(::postcard_forth::impls::ser_string),
                    _other => {
                        quote!(::postcard_forth::ser_fields::<#ty>)
                    },
                };
                let out = quote_spanned!(f.span() => ::postcard_forth::SerField { offset: ::core::mem::offset_of!(#tyname, #name), func: #serf });
                out
            });
            out.extend(quote! {
                #( #fields ),*
            });
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                // let name = &f.ident;
                let tystr = quote!( #ty ).to_string();

                // This is probably not sound. Users could shadow real names with other types, breaking
                // the safety guarantees.
                let serf = match tystr.as_str() {
                    // "u8" => quote!(::postcard_forth::impls::ser_u8),
                    // "u16" => quote!(::postcard_forth::impls::ser_u16),
                    // "u32" => quote!(::postcard_forth::impls::ser_u32),
                    // "u64" => quote!(::postcard_forth::impls::ser_u64),
                    // "u128" => quote!(::postcard_forth::impls::ser_u128),
                    // "usize" => quote!(::postcard_forth::impls::ser_usize),
                    // "i8" => quote!(::postcard_forth::impls::ser_i8),
                    // "i16" => quote!(::postcard_forth::impls::ser_i16),
                    // "i32" => quote!(::postcard_forth::impls::ser_i32),
                    // "i64" => quote!(::postcard_forth::impls::ser_i64),
                    // "i128" => quote!(::postcard_forth::impls::ser_i128),
                    // "isize" => quote!(::postcard_forth::impls::ser_isize),
                    // "String" => quote!(::postcard_forth::impls::ser_string),
                    _other => {
                        quote!(::postcard_forth::ser_fields::<#ty>)
                    },
                };
                let tupidx = syn::Index::from(i);
                let out = quote_spanned!(f.span() => ::postcard_forth::SerField { offset: ::core::mem::offset_of!(#tyname, #tupidx), func: #serf });
                out
            });
            out.extend(quote! {
                #( #fields ),*
            });
        }
        syn::Fields::Unit => {}
    }
    out
}

fn generate_arm(
    fields: &Fields,
    tyident: syn::Ident,
    varident: &syn::Ident,
    idx: u32,
) -> TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let just_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

            let just_names = just_names.as_slice();

            quote! {
                #tyident :: #varident { #(#just_names),* } => {
                    // serialize the discriminant as a u32
                    let var: u32 = #idx;
                    if ::postcard_forth::impls::ser_u32(stream, core::ptr::NonNull::from(&var).cast()).is_err() {
                        return Err(());
                    }

                    // Serialize the payload
                    #(
                        if ::postcard_forth::ser_fields_ref(stream, #just_names).is_err() {
                            return Err(());
                        }
                    )*

                    Ok(())

                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let names = b"abcdefghijklmnopqrstuvwxyz";

            let just_names: Vec<_> = fields
                .unnamed
                .iter()
                .zip(names.iter())
                .map(|(f, c)| {
                    let ch = &[*c];
                    let name = syn::Ident::new(core::str::from_utf8(ch).unwrap(), f.span());
                    quote_spanned! {f.span() => #name}
                })
                .collect();

            let just_names = just_names.as_slice();

            quote! {
                #tyident :: #varident ( #(#just_names),* ) => {
                    // serialize the discriminant as a u32
                    let var: u32 = #idx;
                    if ::postcard_forth::impls::ser_u32(stream, core::ptr::NonNull::from(&var).cast()).is_err() {
                        return Err(());
                    }

                    // Serialize the payload
                    #(
                        if ::postcard_forth::ser_fields_ref(stream, #just_names).is_err() {
                            return Err(());
                        }
                    )*

                    Ok(())

                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                #tyident :: #varident => {
                    let var: u32 = #idx;
                    ::postcard_forth::impls::ser_u32(stream, core::ptr::NonNull::from(&var).cast())
                }
            }
        }
    }
}

/// Add a bound `T: MaxSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::postcard_forth::Serialize));
        }
    }
    generics
}
