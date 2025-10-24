use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Derive macro for making a struct storable in the database
///
/// This macro implements the TableType trait.
/// Note: Structs must also derive serde::Serialize and serde::Deserialize
#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate a unique type name
    let type_name = name.to_string();

    // Validate that this is a struct
    match &input.data {
        Data::Struct(_) => {},
        _ => {
            return TokenStream::from(quote! {
                compile_error!("Table can only be derived for structs");
            });
        }
    }

    let expanded = quote! {
        // Implement the TableType trait
        impl #impl_generics ::struct_db::TableType for #name #ty_generics #where_clause {
            fn type_name() -> &'static str {
                #type_name
            }
        }
    };

    TokenStream::from(expanded)
}
