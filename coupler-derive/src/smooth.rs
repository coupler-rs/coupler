use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, Type};

use super::params::{gen_range, parse_param, ParamAttr};

struct SmoothAttr {
    ty: Type,
    args: Option<Expr>,
}

fn parse_smooth(field: &Field) -> Result<Option<SmoothAttr>, Error> {
    let mut is_smooth = false;

    let mut ty = None;
    let mut args = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("smooth") {
            continue;
        }

        is_smooth = true;

        attr.parse_nested_meta(|meta| {
            let ident = meta.path.get_ident().ok_or_else(|| {
                Error::new_spanned(&meta.path, "expected this path to be an identifier")
            })?;
            if ident == "type" {
                if ty.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate smooth attribute `type`",
                    ));
                }

                ty = Some(meta.value()?.parse::<Type>()?);
            } else if ident == "args" {
                if args.is_some() {
                    return Err(Error::new_spanned(
                        &meta.path,
                        "duplicate smooth attribute `args`",
                    ));
                }

                args = Some(meta.value()?.parse::<Expr>()?);
            } else {
                return Err(Error::new_spanned(
                    &meta.path,
                    format!("unknown smooth attribute `{}`", ident),
                ));
            }

            Ok(())
        })?;
    }

    if !is_smooth {
        return Ok(None);
    }

    let ty = if let Some(ty) = ty {
        ty
    } else {
        return Err(Error::new_spanned(&field, "missing `type` attribute"));
    };

    Ok(Some(SmoothAttr { ty, args }))
}

struct SmoothField<'a> {
    field: &'a Field,
    param: Option<ParamAttr>,
    smooth: Option<SmoothAttr>,
}

fn parse_fields(input: &DeriveInput) -> Result<Vec<SmoothField>, Error> {
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

pub fn expand_smooth(input: &DeriveInput) -> Result<TokenStream, Error> {
    let fields = parse_fields(&input)?;

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let smooth_fields = fields.iter().map(|field| {
        let mut smooth_field = field.field.clone();

        smooth_field.attrs = Vec::new();

        if let Some(smooth) = &field.smooth {
            smooth_field.ty = smooth.ty.clone();
        }

        smooth_field
    });

    let smooth_fields_init = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;

        if let Some(smooth) = &field.smooth {
            let smoother_type = &smooth.ty;

            let args = if let Some(args) = &smooth.args {
                quote! { ::std::convert::From::from(#args) }
            } else {
                quote! { ::std::default::Default::default() }
            };

            quote! {
                #ident: <#smoother_type as ::coupler::params::smooth::Smoother<#ty>>::build(
                    self.#ident,
                    #args,
                    __sample_rate,
                )
            }
        } else {
            quote! { #ident: self.#ident }
        }
    });

    let set_cases = fields.iter().filter_map(|field| {
        let param = field.param.as_ref()?;
        let range = gen_range(&field.field, param);

        let ident = &field.field.ident;
        let ty = &field.field.ty;
        let id = &param.id;

        if field.smooth.is_some() {
            Some(quote! {
                #id => {
                    ::coupler::params::smooth::Smoother::set(
                        &mut self.#ident,
                        ::coupler::params::Range::<#ty>::decode(&(#range), __value),
                    );
                }
            })
        } else {
            Some(quote! {
                #id => {
                    self.#ident = ::coupler::params::Range::<#ty>::decode(&(#range), __value);
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
