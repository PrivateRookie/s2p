use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

#[derive(Default, FromMeta)]
#[darling(default)]
struct Opts {
    apply: Option<syn::Path>,
    col: Option<Ident>,
}

#[proc_macro_derive(Cast, attributes(cast))]
pub fn cast(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
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
    let mut left_fields = vec![];
    let mut init_tuple = vec![];
    let mut trans = vec![];
    let mut to_df = vec![];
    for (idx, f) in fields.named.iter().enumerate() {
        let idx = syn::Index::from(idx);
        let ident = f.ident.clone().unwrap();
        let opt = f.attrs.iter().fold(None, |acc, attrs| {
            if acc.is_none() && attrs.path().is_ident("cast") {
                Some(Opts::from_meta(&attrs.meta).unwrap())
            } else {
                acc
            }
        });
        if let Some(opt) = opt {
            if let Some(apply) = opt.apply {
                trans.push(quote!( (acc.#idx).push(#apply(item.#ident)) ));
            } else {
                trans.push(quote!( (acc.#idx).push(item.#ident) ));
            }
            if let Some(col) = opt.col {
                to_df.push(quote!(stringify!(#col) => #ident));
            } else {
                to_df.push(quote!(stringify!(#ident) => #ident));
            }
        } else {
            trans.push(quote!((acc.#idx).push(item.#ident)));
            to_df.push(quote!(stringify!(#ident) => #ident));
        }
        left_fields.push(ident);
        init_tuple.push(quote!(vec![]));
    }

    quote!(
        impl #ident {
            pub fn to_polars(items: Vec<Self>) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
                use polars::prelude::*;
                let (#(#left_fields),*) = items.into_iter().fold((#(#init_tuple),*), |mut acc, item| {
                    #(#trans;)*
                    acc
                });

                df!(#(#to_df),*)
            }
        }
    ).into()
}
