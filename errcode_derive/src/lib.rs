#![recursion_limit = "256"]

mod enum_info;
mod generate;

extern crate proc_macro;

use proc_macro2::TokenStream;
use venial::Error;

#[proc_macro_derive(ErrorCode, attributes(errmsg))]
pub fn derive_error_code(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let result = derive_error_code_0(input.into());
    result.unwrap_or_else(|err| err.to_compile_error()).into()
}
fn derive_error_code_0(input: TokenStream) -> Result<TokenStream, Error> {
    let item = venial::parse_item(input)?;
    if let Some(item) = item.as_enum() {
        let info = enum_info::parse(item)?;
        Ok(generate::generate(info))
    } else {
        Err(Error::new("#[derive(ErrorCode)] is only supported on enums."))
    }
}
