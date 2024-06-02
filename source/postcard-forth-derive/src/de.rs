use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, ImplGenerics, TypeGenerics, WhereClause,
};

pub fn do_derive_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let span = input.span();
    let name = input.ident;

    // Add a bound `T: Schema` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = generate_type(&input.data, span, name.to_string(), name.clone(), impl_generics, ty_generics, where_clause)
        .unwrap_or_else(syn::Error::into_compile_error);

    // let expanded = quote! {
    //     unsafe impl #impl_generics ::postcard_forth::Deserialize for #name #ty_generics #where_clause {
    //         const FIELDS: &'static [::postcard_forth::DeserField] = &[
    //             #ty
    //         ];
    //     }
    // };

    // eprintln!("{expanded}");

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

            let expanded = quote! {
                unsafe impl #impl_generics ::postcard_forth::Deserialize for #tyident #ty_generics #where_clause {
                    const FIELDS: &'static [::postcard_forth::DeserField] = &[
                        #ty
                    ];
                }
            };

            Ok(expanded)
        },
        Data::Enum(data) => {
            let deserfunc_name = format!("deser_{}", tyident);
            let desername_ident = syn::Ident::new(&deserfunc_name, tyident.span());
            let mut arms = TokenStream::new();
            for (i, var) in data.variants.iter().enumerate() {
                let ident = &var.ident;
                let fields = generate_arm(&var.fields, tyident.clone(), ident, i as u32);
                arms.extend(quote! {
                    #fields
                });
            }

            let out = quote! {
                #[inline]
                pub unsafe fn #desername_ident(stream: &mut ::postcard_forth::DeserStream, base: core::ptr::NonNull<()>) -> Result<(), ()> {
                    let mut variant = core::mem::MaybeUninit::<u32>::uninit();
                    ::postcard_forth::impls::deser_u32(stream, core::ptr::NonNull::from(&mut variant).cast())?;
                    let variant = variant.assume_init();
                    match variant {
                        #arms
                        _ => return Err(()),
                    }
                    Ok(())
                }

                unsafe impl ::postcard_forth::Deserialize for #tyident {
                    const FIELDS: &'static [::postcard_forth::DeserField] = &[::postcard_forth::DeserField {
                        offset: 0,
                        func: #desername_ident,
                    }];
                }
            };
            Ok(out)
        }
        Data::Union(_) => {
            Err(syn::Error::new(
                span,
                "unions are not supported by `postcard::experimental::schema`",
            ))
        }
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
                let deserf = match tystr.as_str() {
                    "u8" => quote!(::postcard_forth::impls::deser_u8),
                    "u16" => quote!(::postcard_forth::impls::deser_u16),
                    "u32" => quote!(::postcard_forth::impls::deser_u32),
                    "u64" => quote!(::postcard_forth::impls::deser_u64),
                    "u128" => quote!(::postcard_forth::impls::deser_u128),
                    "usize" => quote!(::postcard_forth::impls::deser_usize),
                    "i8" => quote!(::postcard_forth::impls::deser_i8),
                    "i16" => quote!(::postcard_forth::impls::deser_i16),
                    "i32" => quote!(::postcard_forth::impls::deser_i32),
                    "i64" => quote!(::postcard_forth::impls::deser_i64),
                    "i128" => quote!(::postcard_forth::impls::deser_i128),
                    "isize" => quote!(::postcard_forth::impls::deser_isize),
                    "String" => quote!(::postcard_forth::impls::deser_string),
                    _other => {
                        quote!(::postcard_forth::deser_fields::<#ty>)
                    },
                };
                let out = quote_spanned!(f.span() => ::postcard_forth::DeserField { offset: ::core::mem::offset_of!(#tyname, #name), func: #deserf });
                out
            });
            out.extend(quote! {
                #( #fields ),*
            });
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                let tystr = quote!( #ty ).to_string();
                let deserf = match tystr.as_str() {
                    "u8" => quote!(::postcard_forth::impls::deser_u8),
                    "u16" => quote!(::postcard_forth::impls::deser_u16),
                    "u32" => quote!(::postcard_forth::impls::deser_u32),
                    "u64" => quote!(::postcard_forth::impls::deser_u64),
                    "u128" => quote!(::postcard_forth::impls::deser_u128),
                    "usize" => quote!(::postcard_forth::impls::deser_usize),
                    "i8" => quote!(::postcard_forth::impls::deser_i8),
                    "i16" => quote!(::postcard_forth::impls::deser_i16),
                    "i32" => quote!(::postcard_forth::impls::deser_i32),
                    "i64" => quote!(::postcard_forth::impls::deser_i64),
                    "i128" => quote!(::postcard_forth::impls::deser_i128),
                    "isize" => quote!(::postcard_forth::impls::deser_isize),
                    "String" => quote!(::postcard_forth::impls::deser_string),
                    _other => {
                        quote!(::postcard_forth::deser_fields::<#ty>)
                    },
                };
                let out = quote_spanned!(f.span() => ::postcard_forth::DeserField { offset: ::core::mem::offset_of!(#tyname, #i), func: #deserf });
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

            let just_tys: Vec<_> = fields.named.iter().map(|f| {
                &f.ty
            }).collect();
            let just_tys = just_tys.as_slice();

            quote! {
                #idx => {
                    // Deserialize the payload
                    #(
                        let mut #just_names = core::mem::MaybeUninit::<#just_tys>::uninit();
                        if ::postcard_forth::deser_fields_ref(stream, &mut #just_names).is_err() {
                            return Err(());
                        }
                    )*

                    base.cast::<#tyident>().as_ptr().write(#tyident :: #varident {
                        #(
                            #just_names: #just_names.assume_init(),
                        )*
                    });
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

            let just_tys: Vec<_> = fields.unnamed.iter().map(|f| {
                &f.ty
            }).collect();
            let just_tys = just_tys.as_slice();

            quote! {
                #idx => {
                    // Deserialize the payload
                    #(
                        let mut #just_names = core::mem::MaybeUninit::<#just_tys>::uninit();
                        if ::postcard_forth::deser_fields_ref(stream, &mut #just_names).is_err() {
                            return Err(());
                        }
                    )*

                    base.cast::<#tyident>().as_ptr().write(#tyident :: #varident (
                        #(
                            #just_names.assume_init(),
                        )*
                    ));

                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                #idx => {
                    base.cast::<#tyident>().as_ptr().write(#tyident :: #varident);
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
