use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, LitInt, LitStr};

pub struct ParamAttr {
    pub id: LitInt,
    pub name: Option<LitStr>,
    pub range: Option<Expr>,
    pub parse: Option<Expr>,
    pub display: Option<Expr>,
    pub format: Option<LitStr>,
}

pub fn parse_param(field: &Field) -> Result<Option<ParamAttr>, Error> {
    let mut is_param = false;

    let mut id = None;
    let mut name = None;
    let mut range = None;
    let mut parse = None;
    let mut display = None;
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
            } else if ident == "parse" {
                if parse.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `parse`",
                    ));
                }

                parse = Some(meta.value()?.parse::<Expr>()?);
            } else if ident == "display" {
                if display.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `display`",
                    ));
                }

                if format.is_some() {
                    return Err(Error::new_spanned(
                        &ident,
                        "`format` attribute cannot be used with `display`",
                    ));
                }

                display = Some(meta.value()?.parse::<Expr>()?);
            } else if ident == "format" {
                if format.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate param attribute `format`",
                    ));
                }

                if display.is_some() {
                    return Err(Error::new_spanned(
                        &ident,
                        "`format` attribute cannot be used with `display`",
                    ));
                }

                format = Some(meta.value()?.parse::<LitStr>()?);
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
        return Err(Error::new_spanned(&field, "missing `id` attribute"));
    };

    Ok(Some(ParamAttr {
        id,
        name,
        range,
        parse,
        display,
        format,
    }))
}

pub fn gen_range(field: &Field, param: &ParamAttr) -> TokenStream {
    if let Some(range) = &param.range {
        range.to_token_stream()
    } else {
        let ty = &field.ty;
        quote! { <#ty as ::coupler::params::DefaultRange>::default_range() }
    }
}

struct ParamField<'a> {
    field: &'a Field,
    param: ParamAttr,
    range: TokenStream,
}

fn parse_fields(input: &DeriveInput) -> Result<Vec<ParamField>, Error> {
    let body = match &input.data {
        Data::Struct(body) => body,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Params)] can only be used on structs",
            ));
        }
    };

    let fields = match &body.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Params)] can only be used on structs with named fields",
            ));
        }
    };

    let mut param_fields = Vec::new();

    for field in &fields.named {
        if let Some(param) = parse_param(field)? {
            let range = gen_range(field, &param);
            param_fields.push(ParamField {
                field,
                param,
                range,
            });
        }
    }

    Ok(param_fields)
}

pub fn expand_params(input: &DeriveInput) -> Result<TokenStream, Error> {
    let fields = parse_fields(&input)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let param_info = fields.iter().map(|field| {
        let ident = field.field.ident.as_ref().unwrap();
        let ty = &field.field.ty;
        let id = &field.param.id;
        let range = &field.range;

        let name = if let Some(name) = &field.param.name {
            name.clone()
        } else {
            LitStr::new(&ident.to_string(), ident.span())
        };

        let encode = quote! { ::coupler::params::Range::<#ty>::encode(&(#range), __value) };
        let parse = if let Some(parse) = &field.param.parse {
            quote! {
                match (#parse)(__str) {
                    ::std::option::Option::Some(__value) => ::std::option::Option::Some(#encode),
                    _ => ::std::option::Option::None,
                }
            }
        } else {
            quote! {
                match <#ty as ::std::str::FromStr>::from_str(__str) {
                    ::std::result::Result::Ok(__value) => ::std::option::Option::Some(#encode),
                    _ => ::std::option::Option::None,
                }
            }
        };

        let decode = quote! { ::coupler::params::Range::<#ty>::decode(&(#range), __value) };
        let display = if let Some(display) = &field.param.display {
            quote! { (#display)(#decode, __formatter) }
        } else if let Some(format) = &field.param.format {
            quote! { write!(__formatter, #format, #decode) }
        } else {
            quote! { write!(__formatter, "{}", #decode) }
        };

        quote! {
            ::coupler::params::ParamInfo {
                id: #id,
                name: ::std::string::ToString::to_string(#name),
                default: ::coupler::params::Range::<#ty>::encode(&(#range), __default.#ident),
                steps: ::coupler::params::Range::<#ty>::steps(&(#range)),
                parse: ::std::boxed::Box::new(|__str| #parse),
                display: ::std::boxed::Box::new(|__value, __formatter| #display),
            }
        }
    });

    let set_cases = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let id = &field.param.id;
        let range = &field.range;

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
        let range = &field.range;

        quote! {
            #id => {
                ::coupler::params::Range::<#ty>::encode(&(#range), self.#ident)
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::coupler::params::Params for #ident #ty_generics #where_clause {
            fn params() -> ::std::vec::Vec<::coupler::params::ParamInfo> {
                let __default: #ident #ty_generics = ::std::default::Default::default();

                ::std::vec![
                    #(#param_info,)*
                ]
            }

            fn set_param(&mut self, __id: ::coupler::params::ParamId, __value: ::coupler::params::ParamValue) {
                match __id {
                    #(#set_cases)*
                    _ => {}
                }
            }

            fn get_param(&self, __id: ::coupler::params::ParamId) -> ::coupler::params::ParamValue {
                match __id {
                    #(#get_cases)*
                    _ => 0.0,
                }
            }
        }
    })
}
