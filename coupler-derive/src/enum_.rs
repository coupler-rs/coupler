use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Error, Fields, LitStr};

fn parse_names(body: &DataEnum) -> Result<Vec<LitStr>, Error> {
    let mut names = Vec::new();

    for variant in &body.variants {
        let mut name = None;

        for attr in &variant.attrs {
            if attr.path().is_ident("name") {
                if name.is_some() {
                    return Err(Error::new_spanned(
                        attr.path(),
                        "duplicate `name` attribute",
                    ));
                }

                name = Some(attr.parse_args::<LitStr>()?);
            }
        }

        let name = if let Some(name) = name {
            name
        } else {
            LitStr::new(&variant.ident.to_string(), variant.ident.span())
        };

        names.push(name);
    }

    Ok(names)
}

pub fn expand_enum(input: &DeriveInput) -> Result<TokenStream, Error> {
    let body = match &input.data {
        Data::Enum(body) => body,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Enum)] can only be used on enums",
            ));
        }
    };

    if body.variants.is_empty() {
        return Err(Error::new_spanned(
            &input,
            "#[derive(Enum)] cannot be used on empty enums",
        ));
    }

    for variant in &body.variants {
        match &variant.fields {
            Fields::Unit => {}
            _ => {
                return Err(Error::new_spanned(
                    &variant,
                    "#[derive(Enum)] can only be used on fieldless enums",
                ));
            }
        }
    }

    let names = parse_names(&body)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let count = body.variants.len() as u32;

    let encode_cases = body.variants.iter().enumerate().map(|(index, variant)| {
        let index = index as u32;
        let variant = &variant.ident;
        quote! {
            #ident::#variant => (#index as ::std::primitive::f64 + 0.5) / #count as ::std::primitive::f64,
        }
    });

    let decode_cases = body.variants.iter().enumerate().map(|(index, variant)| {
        let index = index as u32;
        let variant = &variant.ident;
        quote! {
            #index => #ident::#variant,
        }
    });
    let last_variant = &body.variants.last().as_ref().unwrap().ident;

    let from_str_cases = body.variants.iter().zip(names.iter()).map(|(variant, name)| {
        let variant = &variant.ident;
        quote! {
            #name => ::std::result::Result::Ok(#ident::#variant),
        }
    });

    let fmt_cases = body.variants.iter().zip(names.iter()).map(|(variant, name)| {
        let variant = &variant.ident;
        quote! {
            #ident::#variant => #name,
        }
    });

    Ok(quote! {
        impl #impl_generics ::coupler::params::Encode for #ident #ty_generics #where_clause {
            fn steps() -> Option<u32> {
                Some(#count)
            }

            fn encode(self) -> ParamValue {
                match self {
                    #(#encode_cases)*
                }
            }

            fn decode(__value: ParamValue) -> Self {
                match (__value * #count as ::std::primitive::f64) as ::std::primitive::u32 {
                    #(#decode_cases)*
                    _ => #ident::#last_variant,
                }
            }
        }

        impl #impl_generics ::std::str::FromStr for #ident #ty_generics #where_clause {
            type Err = ();

            fn from_str(__str: &str) -> Result<Self, Self::Err> {
                match __str {
                    #(#from_str_cases)*
                    _ => ::std::result::Result::Err(()),
                }
            }
        }

        impl #impl_generics ::std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, __formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                ::std::fmt::Formatter::write_str(__formatter, match self {
                    #(#fmt_cases)*
                })
            }
        }

        impl #impl_generics ::coupler::params::Enum for #ident #ty_generics #where_clause {}
    })
}
