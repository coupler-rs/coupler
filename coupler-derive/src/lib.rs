use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod params;
mod smooth;

use params::expand_params;
use smooth::expand_smooth;

#[proc_macro_derive(Params, attributes(param))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    match expand_params(&input) {
        Ok(params) => params.into(),
        Err(err) => return err.into_compile_error().into(),
    }
}

#[proc_macro_derive(Smooth, attributes(param, smooth))]
pub fn derive_smooth(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    match expand_smooth(&input) {
        Ok(smooth) => smooth.into(),
        Err(err) => return err.into_compile_error().into(),
    }
}
