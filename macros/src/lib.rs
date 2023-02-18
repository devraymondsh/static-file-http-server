extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::env;

#[proc_macro]
pub fn single_binary_producer_dir(_input: TokenStream) -> TokenStream {
    let dir = if env::var("CARGO_PUBLISH").is_err() {
        "$CARGO_MANIFEST_DIR/single-binary-producer"
    } else {
        "$CARGO_MANIFEST_DIR/../../../single-binary-producer"
    };

    quote! {
        include_directory::include_directory!(#dir)
    }
    .into()
}
