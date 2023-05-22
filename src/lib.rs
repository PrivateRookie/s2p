use darling::{FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Type};

#[derive(Default, FromMeta)]
#[darling(default)]
struct Opts {
    t: Option<Ident>,
    apply: Option<syn::Path>,
    col: Option<String>,
}

#[proc_macro_derive(Cast, attributes(cast))]
pub fn cast(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let helper = Ident::new(&format!("{}Collector", input.ident), input.ident.span());
    let data = match input.data {
        Data::Struct(data) => data,
        _ => {
            panic!("only support struct")
        }
    };
    let fields = match data.fields {
        syn::Fields::Named(fields) => fields,
        _ => {
            panic!("only support named field")
        }
    };
    let fields: Vec<(Ident, Type, Opts)> = fields
        .named
        .into_iter()
        .map(|f| {
            let opts = f
                .attrs
                .iter()
                .fold(None, |acc, attrs| {
                    if acc.is_none() && attrs.path().is_ident("cast") {
                        Some(Opts::from_meta(&attrs.meta).unwrap())
                    } else {
                        acc
                    }
                })
                .unwrap_or_default();

            (f.ident.unwrap(), f.ty, opts)
        })
        .collect();
    let fields_def: Vec<_> = fields
        .iter()
        .map(|(name, ty, opts)| match opts.t.clone() {
            Some(ty) => {
                quote!(#name: std::vec::Vec<#ty>)
            }
            None => {
                quote!(#name: std::vec::Vec<#ty>)
            }
        })
        .collect();
    let append_def: Vec<_> = fields
        .iter()
        .map(|(name, _, opts)| {
            if let Some(convert) = opts.apply.clone() {
                quote!(
                    self.#name.push(#convert(item.#name))
                )
            } else {
                quote!(self.#name.push(item.#name))
            }
        })
        .collect();

    let to_polars_def: Vec<_> = fields
        .iter()
        .map(|(name, _, opts)| {
            match opts.col.clone() {
                Some(col) => {
                    quote!(#col => self.#name)
                },
                None => {
                    quote!(stringify!(#name) => self.#name)
                }
            }
        })
        .collect();

    let helper_def = quote! {
        #[derive(Default)]
        struct #helper {
            #(#fields_def),*
        }

        impl #helper {
            fn append(&mut self, item: #ident) {
                #(#append_def);*
            }

            fn to_polars(self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
                use polars::prelude::*;

                polars::prelude::df! {
                    #(#to_polars_def),*
                }
            }
        }

        
    };
    quote!(
        impl #ident {
            pub fn to_polars(items: Vec<Self>) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
                #helper_def

                items.into_iter().fold(#helper::default(), |mut acc, item| {
                    acc.append(item);
                    acc
                }).to_polars()
                
            }
        }
    ).into()
}
