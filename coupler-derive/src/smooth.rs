use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, Ident, Path, Token};

use super::params::{gen_decode, parse_param, ParamAttr};

struct SmoothAttr {
    builder: Path,
    args: Punctuated<SmoothArg, Token![,]>,
}

impl Parse for SmoothAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let builder = input.parse::<Path>()?;

        let args = if input.peek(Token![,]) {
            let _ = input.parse::<Token![,]>()?;
            input.parse_terminated(SmoothArg::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        Ok(SmoothAttr { builder, args })
    }
}

struct SmoothArg {
    name: Ident,
    value: Expr,
}

impl Parse for SmoothArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse::<Expr>()?;

        Ok(SmoothArg { name, value })
    }
}

fn parse_smooth(field: &Field) -> Result<Option<SmoothAttr>> {
    let mut smooth = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("smooth") {
            continue;
        }

        if smooth.is_some() {
            return Err(Error::new_spanned(
                attr.path(),
                "duplicate `smooth` attribute",
            ));
        }

        smooth = Some(attr.parse_args::<SmoothAttr>()?);
    }

    Ok(smooth)
}

struct SmoothField<'a> {
    field: &'a Field,
    param: Option<ParamAttr>,
    smooth: Option<SmoothAttr>,
}

fn parse_fields(input: &DeriveInput) -> Result<Vec<SmoothField>> {
    let body = match &input.data {
        Data::Struct(body) => body,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Smooth)] can only be used on structs",
            ));
        }
    };

    let fields = match &body.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Smooth)] can only be used on structs with named fields",
            ));
        }
    };

    let mut smooth_fields = Vec::new();

    for field in &fields.named {
        let param = parse_param(field)?;
        let smooth = parse_smooth(field)?;

        if smooth.is_some() && !param.is_some() {
            return Err(Error::new_spanned(
                &field,
                "smooth attribute requires param attribute",
            ));
        }

        smooth_fields.push(SmoothField {
            field,
            param,
            smooth,
        });
    }

    Ok(smooth_fields)
}

pub fn expand_smooth(input: &DeriveInput) -> Result<TokenStream> {
    let fields = parse_fields(&input)?;

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let smooth_fields = fields.iter().map(|field| {
        let vis = &field.field.vis;
        let ident = &field.field.ident;
        let ty = &field.field.ty;

        let field_type = if let Some(smooth) = &field.smooth {
            let builder = &smooth.builder;
            quote! { <#builder as ::coupler::params::smooth::BuildSmoother<#ty>>::Smoother }
        } else {
            quote! { #ty }
        };

        quote! { #vis #ident: #field_type }
    });

    let smooth_fields_init = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;

        if let Some(smooth) = &field.smooth {
            let builder = &smooth.builder;

            let args = smooth.args.iter().map(|arg| {
                let name = &arg.name;
                let value = &arg.value;
                quote! { .#name(#value) }
            });

            quote! {
                #ident: <#builder as ::coupler::params::smooth::BuildSmoother<#ty>>::build(
                    <#builder as ::std::default::Default>::default() #(#args)*,
                    ::std::clone::Clone::clone(&self.#ident),
                    __sample_rate,
                )
            }
        } else {
            quote! { #ident: ::std::clone::Clone::clone(&self.#ident) }
        }
    });

    let set_cases = fields.iter().filter_map(|field| {
        let param = field.param.as_ref()?;

        let ident = &field.field.ident;
        let id = &param.id;

        let decode = gen_decode(&field.field, param, quote! { __value });

        if field.smooth.is_some() {
            Some(quote! {
                #id => {
                    ::coupler::params::smooth::Smoother::set(&mut self.#ident, #decode);
                }
            })
        } else {
            Some(quote! {
                #id => {
                    self.#ident = #decode;
                }
            })
        }
    });

    let reset_stmts = fields.iter().filter_map(|field| {
        let ident = &field.field.ident;

        if field.param.is_none() || field.smooth.is_none() {
            return None;
        }

        Some(quote! {
            ::coupler::params::smooth::Smoother::reset(&mut self.#ident);
        })
    });

    Ok(quote! {
        const _: () = {
            pub struct __Smooth #generics {
                #(#smooth_fields,)*
            }

            impl #impl_generics ::coupler::params::smooth::Smooth for #ident #ty_generics #where_clause {
                type Smoothed = __Smooth #ty_generics;

                fn smoothed(&self, __sample_rate: f64) -> Self::Smoothed {
                    __Smooth {
                        #(#smooth_fields_init,)*
                    }
                }
            }

            impl #impl_generics ::coupler::params::smooth::SmoothParams for __Smooth #ty_generics #where_clause {
                fn set_param(&mut self, __id: ParamId, __value: ParamValue) {
                    match __id {
                        #(#set_cases)*
                        _ => {}
                    }
                }

                fn reset(&mut self) {
                    #(#reset_stmts)*
                }
            }
        };
    })
}
