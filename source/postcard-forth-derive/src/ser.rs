use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics,
};

pub fn do_derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let span = input.span();
    let name = input.ident;

    // Add a bound `T: Schema` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = generate_type(&input.data, span, name.to_string(), name.clone())
        .unwrap_or_else(syn::Error::into_compile_error);

    let expanded = quote! {
        unsafe impl #impl_generics ::postcard_forth::Serialize for #name #ty_generics #where_clause {
            const FIELDS: &'static [::postcard_forth::SerField] = &[
                #ty
            ];
        }
    };

    expanded.into()
}

fn generate_type(data: &Data, span: Span, _name: String, tyident: syn::Ident) -> Result<TokenStream, syn::Error> {
    let ty = match data {
        Data::Struct(data) => generate_struct(tyident, &data.fields),
        Data::Enum(_data) => {
            // let name = data.variants.iter().map(|v| v.ident.to_string());
            // let ty = data.variants.iter().map(|v| generate_variants(&v.fields));

            // quote! {
            //     &::postcard::experimental::schema::SdmTy::Enum(&[
            //         #( &::postcard::experimental::schema::NamedVariant { name: #name, ty: #ty } ),*
            //     ])
            // }
            todo!()
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                span,
                "unions are not supported by `postcard::experimental::schema`",
            ))
        }
    };

    Ok(quote! {
        #ty
    })
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
                    "u8" => quote!(::postcard_forth::impls::ser_u8),
                    "u16" => quote!(::postcard_forth::impls::ser_u16),
                    "u32" => quote!(::postcard_forth::impls::ser_u32),
                    "u64" => quote!(::postcard_forth::impls::ser_u64),
                    "u128" => quote!(::postcard_forth::impls::ser_u128),
                    "usize" => quote!(::postcard_forth::impls::ser_usize),
                    "i8" => quote!(::postcard_forth::impls::ser_i8),
                    "i16" => quote!(::postcard_forth::impls::ser_i16),
                    "i32" => quote!(::postcard_forth::impls::ser_i32),
                    "i64" => quote!(::postcard_forth::impls::ser_i64),
                    "i128" => quote!(::postcard_forth::impls::ser_i128),
                    "isize" => quote!(::postcard_forth::impls::ser_isize),
                    "String" => quote!(::postcard_forth::impls::ser_string),
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
                    "u8" => quote!(::postcard_forth::impls::ser_u8),
                    "u16" => quote!(::postcard_forth::impls::ser_u16),
                    "u32" => quote!(::postcard_forth::impls::ser_u32),
                    "u64" => quote!(::postcard_forth::impls::ser_u64),
                    "u128" => quote!(::postcard_forth::impls::ser_u128),
                    "usize" => quote!(::postcard_forth::impls::ser_usize),
                    "i8" => quote!(::postcard_forth::impls::ser_i8),
                    "i16" => quote!(::postcard_forth::impls::ser_i16),
                    "i32" => quote!(::postcard_forth::impls::ser_i32),
                    "i64" => quote!(::postcard_forth::impls::ser_i64),
                    "i128" => quote!(::postcard_forth::impls::ser_i128),
                    "isize" => quote!(::postcard_forth::impls::ser_isize),
                    "String" => quote!(::postcard_forth::impls::ser_string),
                    _other => {
                        quote!(::postcard_forth::ser_fields::<#ty>)
                    },
                };
                let out = quote_spanned!(f.span() => ::postcard_forth::SerField { offset: ::core::mem::offset_of!(#tyname, #i), func: #serf });
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

fn _generate_variants(_fields: &Fields) -> TokenStream {
    // match fields {
    //     syn::Fields::Named(fields) => {
    //         let fields = fields.named.iter().map(|f| {
    //             let ty = &f.ty;
    //             let name = f.ident.as_ref().unwrap().to_string();
    //             quote_spanned!(f.span() => &::postcard::experimental::schema::NamedValue { name: #name, ty: <#ty as ::postcard::experimental::schema::Schema>::SCHEMA })
    //         });
    //         quote! { &::postcard::experimental::schema::SdmTy::StructVariant(&[
    //             #( #fields ),*
    //         ]) }
    //     }
    //     syn::Fields::Unnamed(fields) => {
    //         let fields = fields.unnamed.iter().map(|f| {
    //             let ty = &f.ty;
    //             quote_spanned!(f.span() => <#ty as ::postcard::experimental::schema::Schema>::SCHEMA)
    //         });
    //         quote! { &::postcard::experimental::schema::SdmTy::TupleVariant(&[
    //             #( #fields ),*
    //         ]) }
    //     }
    //     syn::Fields::Unit => {
    //         quote! { &::postcard::experimental::schema::SdmTy::UnitVariant }
    //     }
    // }
    quote! { }
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
