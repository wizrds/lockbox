#[allow(unused_extern_crates)]
extern crate self as lockbox_macros;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream};
use syn::Error;


#[proc_macro_derive(RequestDTO)]
pub fn derive_request_dto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl<S> axum::extract::FromRequest<S> for #name
        where
            S: Send + Sync,
            axum::extract::Json<#name>: axum::extract::FromRequest<S>,
        {
            type Rejection = <axum::extract::Json<#name> as axum::extract::FromRequest<S>>::Rejection;

            async fn from_request(
                req: axum::http::Request<axum::body::Body>,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                Ok(axum::extract::Json::<#name>::from_request(req, state).await?.0)
            }
        }
    };

    TokenStream::from(expanded)
}


struct ResponseDTOMacroArgs {
    status_code: Option<Expr>,
}

impl ResponseDTOMacroArgs {
    fn from_attributes(attrs: &[syn::Attribute]) -> Option<ResponseDTOMacroArgs> {
        for attr in attrs.iter().filter(|a| a.path().is_ident("response")) {
            if let Ok(args) = attr.parse_args::<ResponseDTOMacroArgs>() {
                return Some(args);
            }
        }

        None
    }
}

impl Parse for ResponseDTOMacroArgs {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut status_code = None;
    
        let args = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;
        
        for arg in args {
            if let syn::Meta::NameValue(nv) = arg {
                if nv.path.is_ident("status_code") {
                    status_code = Some(nv.value);
                }
            }
        }

        Ok(ResponseDTOMacroArgs { status_code })
    }
}


#[proc_macro_derive(ResponseDTO, attributes(response))]
pub fn derive_response_dto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let args = ResponseDTOMacroArgs::from_attributes(&input.attrs);
    let status_expr = args
        .and_then(|args| args.status_code)
        .unwrap_or_else(|| syn::parse_quote! { axum::http::StatusCode::OK });

    let expanded = quote! {
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                (#status_expr, axum::response::Json(self)).into_response()
            }
        }
    };

    TokenStream::from(expanded)
}
