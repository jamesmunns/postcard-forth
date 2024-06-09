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
            if data.fields.is_empty() {
                Ok(quote! {
                    unsafe impl #impl_generics ::postcard_forth::Serialize for #tyident #ty_generics #where_clause {
                        const NODE: ::postcard_forth::SerNode = ::postcard_forth::SerNode::new_custom(::postcard_forth::impls::ser_nothing);
                    }
                })
            } else {
                let ty = generate_struct(tyident.clone(), &data.fields);
                Ok(quote! {
                    impl #impl_generics #tyident #ty_generics #where_clause {
                        const SER_FIELDS: &'static [::postcard_forth::SerField] = &[
                            #ty
                        ];
                    }

                    unsafe impl #impl_generics ::postcard_forth::Serialize for #tyident #ty_generics #where_clause {
                        const NODE: ::postcard_forth::SerNode = ::postcard_forth::SerNode::new_arry(Self::SER_FIELDS);
                    }
                })
            }
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
                pub unsafe fn #sername_ident(stream: &mut ::postcard_forth::SerStream, base: ::core::ptr::NonNull<()>) -> Result<(), ()> {
                    let eref = base.cast::<#tyident>().as_ref();
                    match eref {
                        #arms
                    }
                }

                unsafe impl ::postcard_forth::Serialize for #tyident {
                    const NODE: ::postcard_forth::SerNode = ::postcard_forth::SerNode::new_custom(#sername_ident);
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
                let out = quote_spanned!(f.span() => ::postcard_forth::SerField { offset: ::core::mem::offset_of!(#tyname, #name), node: &<#ty as ::postcard_forth::Serialize>::NODE });
                out
            });
            out.extend(quote! {
                #( #fields ),*
            });
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                let tupidx = syn::Index::from(i);
                let out = quote_spanned!(f.span() => ::postcard_forth::SerField { offset: ::core::mem::offset_of!(#tyname, #tupidx), node: &<#ty as ::postcard_forth::Serialize>::NODE });
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
            let just_tys: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

            let just_names = just_names.as_slice();
            let just_tys = just_tys.as_slice();

            quote! {
                #tyident :: #varident { #(#just_names),* } => {
                    // serialize the discriminant as a u32
                    let var: u32 = #idx;
                    if ::postcard_forth::impls::ser_u32(stream, ::core::ptr::NonNull::from(&var).cast()).is_err() {
                        return Err(());
                    }

                    // Serialize the payload
                    #(
                        let base: ::core::ptr::NonNull<#just_tys> = ::core::ptr::NonNull::from(#just_names);
                        if ::postcard_forth::ser_node(stream, base.cast(), &<#just_tys as ::postcard_forth::Serialize>::NODE).is_err() {
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
            let just_tys: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

            let just_names = just_names.as_slice();
            let just_tys = just_tys.as_slice();


            quote! {
                #tyident :: #varident ( #(#just_names),* ) => {
                    // serialize the discriminant as a u32
                    let var: u32 = #idx;
                    if ::postcard_forth::impls::ser_u32(stream, ::core::ptr::NonNull::from(&var).cast()).is_err() {
                        return Err(());
                    }

                    // Serialize the payload
                    #(
                        let base: ::core::ptr::NonNull<#just_tys> = ::core::ptr::NonNull::from(#just_names);
                        if ::postcard_forth::ser_node(stream, base.cast(), &<#just_tys as ::postcard_forth::Serialize>::NODE).is_err() {
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
                    ::postcard_forth::impls::ser_u32(stream, ::core::ptr::NonNull::from(&var).cast())
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
