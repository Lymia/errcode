#![no_std]
extern crate alloc;

mod error_code;
mod error_impl;
mod error_ty;
mod macros;
mod traits;

pub use errcode_derive::ErrorCode;
pub use error_code::ErrorCode;
pub use error_ty::{Error, ErrorFrame, ErrorFrameIter, ErrorInfo};

/// A module containing helpful imports for using this crate.
pub mod prelude {
    use crate::Error;

    /// A convince wrapper over the [`Result`](`core::result::Result`) type.
    pub type Result<T> = core::result::Result<T, Error>;

    pub use crate::traits::{ConvertErrorHelper, IntoErrorHelper};

    pub use crate::{bail, ensure, error, error_info};
}

/// NOT PUBLIC API!
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::error_code::{ErrorCodeInfo, ErrorCodePrivate};
    pub use crate::error_impl::{DecodedLocation, ErrorInfoImpl, StaticMessageInfo};
    pub use crate::error_ty::new_error_info;
    pub use crate::macros::{get_helper, static_message, wrap_code};
    pub use core;
    pub use core::option::Option::{None, Some};
}
