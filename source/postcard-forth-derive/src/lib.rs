mod de;
mod ser;

/// Derive the `postcard::Schema` trait for a struct or enum.
#[proc_macro_derive(Serialize)]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ser::do_derive_serialize(item)
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    de::do_derive_deserialize(item)
}
