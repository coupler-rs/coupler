use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Error, Expr, Fields, Ident, LitInt, LitStr, Type};

struct ParamInfo {
    ident: Ident,
    ty: Type,
    id: LitInt,
    name: Option<LitStr>,
    range: Option<Expr>,
    parse: Option<Expr>,
    display: Option<Expr>,
    format: Option<LitStr>,
}

fn parse_struct(input: &DeriveInput) -> Result<Vec<ParamInfo>, Error> {
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

    let mut params = Vec::new();

    for field in &fields.named {
        let mut param_info = None;

        for attr in &field.attrs {
            if !attr.path().is_ident("param") {
                continue;
            }

            if param_info.is_some() {
                return Err(Error::new_spanned(&attr, "duplicate `param` attribute"));
            }

            let mut id = None;
            let mut name = None;
            let mut range = None;
            let mut parse = None;
            let mut display = None;
            let mut format = None;

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

                    display = Some(meta.value()?.parse::<Expr>()?);
                } else if ident == "format" {
                    if format.is_some() {
                        return Err(Error::new_spanned(
                            &meta.path,
                            "duplicate param attribute `format`",
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

            let id = if let Some(id) = id {
                id
            } else {
                return Err(Error::new_spanned(&attr, "missing `id` attribute"));
            };

            if display.is_some() && format.is_some() {
                return Err(Error::new_spanned(
                    &attr,
                    "`format` attribute cannot be used with `display`",
                ));
            }

            param_info = Some(ParamInfo {
                ident: field.ident.clone().unwrap(),
                ty: field.ty.clone(),
                id,
                name,
                range,
                parse,
                display,
                format,
            });
        }

        if let Some(param_info) = param_info {
            params.push(param_info);
        }
    }

    Ok(params)
}

#[proc_macro_derive(Params, attributes(param))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let params = match parse_struct(&input) {
        Ok(params) => params,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let ranges: Vec<_> = params
        .iter()
        .map(|param| {
            if let Some(range) = &param.range {
                range.to_token_stream()
            } else {
                let ty = &param.ty;
                quote! { <#ty as ::coupler::params::DefaultRange>::default_range() }
            }
        })
        .collect();

    let params_expanded = params.iter().zip(&ranges).map(|(param, range)| {
        let ident = &param.ident;
        let ty = &param.ty;
        let id = &param.id;

        let name = if let Some(name) = &param.name {
            name.clone()
        } else {
            LitStr::new(&param.ident.to_string(), param.ident.span())
        };

        let encode = quote! { ::coupler::params::Range::<#ty>::encode(&(#range), __value) };
        let parse = if let Some(parse) = &param.parse {
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
        let display = if let Some(display) = &param.display {
            quote! { (#display)(#decode, __formatter) }
        } else if let Some(format) = &param.format {
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

    let set_cases = params.iter().zip(&ranges).map(|(param, range)| {
        let ident = &param.ident;
        let ty = &param.ty;
        let id = &param.id;

        quote! {
            #id => {
                self.#ident = ::coupler::params::Range::<#ty>::decode(&(#range), __value);
            }
        }
    });

    let get_cases = params.iter().zip(&ranges).map(|(param, range)| {
        let ident = &param.ident;
        let ty = &param.ty;
        let id = &param.id;

        quote! {
            #id => {
                ::coupler::params::Range::<#ty>::encode(&(#range), self.#ident)
            }
        }
    });

    let expanded = quote! {
        impl #impl_generics ::coupler::params::Params for #ident #ty_generics #where_clause {
            fn params() -> ::std::vec::Vec<::coupler::params::ParamInfo> {
                let __default: #ident #ty_generics = ::std::default::Default::default();

                ::std::vec![
                    #(#params_expanded,)*
                ]
            }

            fn set_param(&mut self, __id: ::coupler::ParamId, __value: ::coupler::ParamValue) {
                match __id {
                    #(#set_cases)*
                    _ => {}
                }
            }

            fn get_param(&self, __id: ::coupler::ParamId) -> ::coupler::ParamValue {
                match __id {
                    #(#get_cases)*
                    _ => 0.0,
                }
            }
        }
    };

    expanded.into()
}
