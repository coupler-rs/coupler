use proc_macro::TokenStream;

#[proc_macro_derive(Params)]
pub fn derive_params(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
