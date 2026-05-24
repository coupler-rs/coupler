use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, LitInt, LitStr};

pub struct ParamAttr {
    pub id: LitInt,
    pub name: LitStr,
    pub range: TokenStream,
    pub format: TokenStream,
}

pub fn parse_param(field: &Field) -> Result<Option<ParamAttr>, Error> {
    let mut is_param = false;

    let mut id = None;
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
            if ident == "id" {
                if id.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `id`",
                    ));
                }

                id = Some(meta.value()?.parse::<LitInt>()?);
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

    let id = if let Some(id) = id {
        id
    } else {
        return Err(Error::new_spanned(field, "missing `id` attribute"));
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

    Ok(Some(ParamAttr {
        id,
        name,
        range,
        format,
    }))
}

struct ParamField<'a> {
    field: &'a Field,
    param: ParamAttr,
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

    let param_info = fields.iter().map(|field| {
        let ident = field.field.ident.as_ref().unwrap();
        let ty = &field.field.ty;
        let id = &field.param.id;
        let name = &field.param.name;
        let range = &field.param.range;

        quote! {
            ::coupler::params::ParamInfo {
                id: #id,
                name: #name,
                default: ::coupler::params::Range::<#ty>::encode(&(#range), &__default.#ident),
                steps: ::coupler::params::Range::<#ty>::steps(&(#range)),
            }
        }
    });

    let set_cases = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let id = &field.param.id;
        let range = &field.param.range;

        quote! {
            #id => {
                self.#ident = ::coupler::params::Range::<#ty>::decode(&(#range), __value);
            }
        }
    });

    let get_cases = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let id = &field.param.id;
        let range = &field.param.range;

        quote! {
            #id => ::coupler::params::Range::<#ty>::encode(&(#range), &self.#ident),
        }
    });

    let parse_cases = fields.iter().map(|field| {
        let ty = &field.field.ty;
        let id = &field.param.id;
        let range = &field.param.range;
        let format = &field.param.format;

        quote! {
            #id => match ::coupler::params::Format::<#ty>::parse(&(#format), __text) {
                ::std::option::Option::Some(__value) => ::std::option::Option::Some(
                    ::coupler::params::Range::<#ty>::encode(&(#range), &__value),
                ),
                _ => ::std::option::Option::None,
            }
        }
    });

    let display_cases = fields.iter().map(|field| {
        let id = &field.param.id;
        let ty = &field.field.ty;
        let range = &field.param.range;
        let format = &field.param.format;

        quote! {
            #id => ::coupler::params::Format::<#ty>::display(
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
                   #(.param(#param_info))*;
            }

            fn set_param(&mut self, __id: ::std::primitive::u32, __value: ::std::primitive::f64) {
                match __id {
                    #(#set_cases)*
                    _ => {}
                }
            }

            fn get_param(&self, __id: ::std::primitive::u32) -> ::std::primitive::f64 {
                match __id {
                    #(#get_cases)*
                    _ => 0.0,
                }
            }

            fn parse_param(&self, __id: ::std::primitive::u32, __text: &::std::primitive::str) -> ::std::option::Option<::std::primitive::f64> {
                match __id {
                    #(#parse_cases)*
                    _ => ::std::option::Option::None
                }
            }

            fn display_param(
                &self,
                __id: ::std::primitive::u32,
                __value: ::std::primitive::f64,
                __write: impl fmt::Write,
            ) -> ::std::result::Result<(), ::std::fmt::Error> {
                match __id {
                    #(#display_cases)*
                    _ => Ok(())
                }
            }
        }
    })
}
