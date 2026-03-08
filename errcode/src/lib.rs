#![no_std]
extern crate alloc;

mod error;
mod error_code;
mod error_impl;
mod macros;
mod traits;

pub use errcode_derive::ErrorCode;
pub use error::{Error, ErrorInfo};
pub use error_code::ErrorCode;

/// A module containing helpful imports for using this crate.
pub mod prelude {
    use crate::Error;

    /// A convince wrapper over the [`Result`](`core::result::Result`) type.
    pub type Result<T> = core::result::Result<T, Error>;

    pub use crate::traits::{ConvertErrorHelper, IntoErrorHelper};
}

/// NOT PUBLIC API!
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::error::new_error_info;
    pub use crate::error_code::{ErrorCodeInfo, ErrorCodePrivate};
    pub use crate::error_impl::{DecodedLocation, ErrorInfoImpl, StaticMessageInfo};
    pub use crate::macros::{static_message, wrap_code, get_helper};
    pub use core;
    pub use core::option::Option::{None, Some};
}
