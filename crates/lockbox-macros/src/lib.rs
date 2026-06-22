#[allow(unused_extern_crates)]
extern crate self as lockbox_macros;

mod http;

use proc_macro::TokenStream;


#[proc_macro_derive(RequestDTO)]
pub fn derive_request_dto(input: TokenStream) -> TokenStream {
    http::derive_request_dto_impl(input)
}

#[proc_macro_derive(ResponseDTO, attributes(response))]
pub fn derive_response_dto(input: TokenStream) -> TokenStream {
    http::derive_response_dto_impl(input)
}
