use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, LitStr};

struct ParamAttrs {
    key: LitStr,
    name: LitStr,
    range: TokenStream,
    format: TokenStream,
}

fn parse_param(field: &Field) -> Result<Option<ParamAttrs>, Error> {
    let mut is_param = false;

    let mut key = None;
    let mut name = None;
    let mut range = None;
    let mut format = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("param") {
            continue;
        }

        is_param = true;

        attr.parse_nested_meta(|meta| {
            let ident = meta.path.get_ident().ok_or_else(|| {
                Error::new_spanned(&meta.path, "expected this path to be an identifier")
            })?;
            if ident == "key" {
                if key.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `key`",
                    ));
                }

                key = Some(meta.value()?.parse::<LitStr>()?);
            } else if ident == "name" {
                if name.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `name`",
                    ));
                }

                name = Some(meta.value()?.parse::<LitStr>()?);
            } else if ident == "range" {
                if range.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `range`",
                    ));
                }

                range = Some(meta.value()?.parse::<Expr>()?);
            } else if ident == "format" {
                if format.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `format`",
                    ));
                }

                format = Some(meta.value()?.parse::<Expr>()?);
            } else {
                return Err(Error::new_spanned(
                    &meta.path,
                    format!("unknown param attribute `{}`", ident),
                ));
            }

            Ok(())
        })?;
    }

    if !is_param {
        return Ok(None);
    }

    let key = if let Some(key) = key {
        key
    } else {
        let ident = field.ident.as_ref().unwrap();
        LitStr::new(&ident.to_string(), ident.span())
    };

    let name = if let Some(name) = name {
        name.clone()
    } else {
        let ident = field.ident.as_ref().unwrap();
        LitStr::new(&ident.to_string(), ident.span())
    };

    let range = if let Some(range) = range {
        quote! { #range }
    } else {
        quote! { ::coupler::params::DefaultRange }
    };

    let format = if let Some(format) = format {
        quote! { #format }
    } else {
        quote! { ::coupler::params::DefaultFormat }
    };

    Ok(Some(ParamAttrs {
        key,
        name,
        range,
        format,
    }))
}

struct ParamField<'a> {
    field: &'a Field,
    param: ParamAttrs,
}

fn parse_fields(input: &DeriveInput) -> Result<Vec<ParamField<'_>>, Error> {
    let body = match &input.data {
        Data::Struct(body) => body,
        _ => {
            return Err(Error::new_spanned(
                input,
                "#[derive(Params)] can only be used on structs",
            ));
        }
    };

    let fields = match &body.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(Error::new_spanned(
                input,
                "#[derive(Params)] can only be used on structs with named fields",
            ));
        }
    };

    let mut param_fields = Vec::new();

    for field in &fields.named {
        if let Some(param) = parse_param(field)? {
            param_fields.push(ParamField { field, param });
        }
    }

    Ok(param_fields)
}

pub fn expand_params(input: &DeriveInput) -> Result<TokenStream, Error> {
    let fields = parse_fields(input)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let param_keys = fields.iter().map(|field| {
        let key = &field.param.key;

        quote! { #key }
    });

    let param_infos = fields.iter().map(|field| {
        let ident = field.field.ident.as_ref().unwrap();
        let ty = &field.field.ty;
        let name = &field.param.name;
        let range = &field.param.range;

        quote! {
            ::coupler::params::ParamInfo {
                name: #name,
                default: ::coupler::params::Range::<#ty>::encode(&(#range), &__default.#ident),
                steps: ::coupler::params::Range::<#ty>::steps(&(#range)),
            }
        }
    });

    let set_cases = fields.iter().enumerate().map(|(index, field)| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let range = &field.param.range;

        quote! {
            #index => {
                self.#ident = ::coupler::params::Range::<#ty>::decode(&(#range), __value);
            }
        }
    });

    let get_cases = fields.iter().enumerate().map(|(index, field)| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let range = &field.param.range;

        quote! {
            #index => ::coupler::params::Range::<#ty>::encode(&(#range), &self.#ident),
        }
    });

    let parse_cases = fields.iter().enumerate().map(|(index, field)| {
        let ty = &field.field.ty;
        let range = &field.param.range;
        let format = &field.param.format;

        quote! {
            #index => match ::coupler::params::Format::<#ty>::parse(&(#format), __text) {
                ::std::option::Option::Some(__value) => ::std::option::Option::Some(
                    ::coupler::params::Range::<#ty>::encode(&(#range), &__value),
                ),
                _ => ::std::option::Option::None,
            }
        }
    });

    let display_cases = fields.iter().enumerate().map(|(index, field)| {
        let ty = &field.field.ty;
        let range = &field.param.range;
        let format = &field.param.format;

        quote! {
            #index => ::coupler::params::Format::<#ty>::display(
                &(#format),
                ::coupler::params::Range::<#ty>::decode(&(#range), __value),
                __write,
            ),
        }
    });

    Ok(quote! {
        impl #impl_generics ::coupler::params::Params for #ident #ty_generics #where_clause {
            fn params(&self, __build: impl ::coupler::params::BuildParams) {
                let __default: #ident #ty_generics = ::std::default::Default::default();

                __build
                    #(.param(#param_keys, #param_infos))*;
            }

            fn set_param(&mut self, __index: ::std::primitive::usize, __value: ::std::primitive::f64) {
                match __index {
                    #(#set_cases)*
                    _ => {}
                }
            }

            fn get_param(&self, __index: ::std::primitive::usize) -> ::std::primitive::f64 {
                match __index {
                    #(#get_cases)*
                    _ => 0.0,
                }
            }

            fn parse_param(&self, __index: ::std::primitive::usize, __text: &::std::primitive::str) -> ::std::option::Option<::std::primitive::f64> {
                match __index {
                    #(#parse_cases)*
                    _ => ::std::option::Option::None
                }
            }

            fn display_param(
                &self,
                __index: ::std::primitive::usize,
                __value: ::std::primitive::f64,
                __write: impl ::std::fmt::Write,
            ) -> ::std::result::Result<(), ::std::fmt::Error> {
                match __index {
                    #(#display_cases)*
                    _ => Ok(())
                }
            }
        }
    })
}
