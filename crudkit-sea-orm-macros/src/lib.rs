use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

mod derives;

/// Derives a `read_view` module containing a copy of the annotated struct with the
/// `pub has_validation_errors: bool` field added to it.
#[proc_macro_derive(ReadView, attributes(read_view))]
pub fn derive_migration_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_read_view(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
