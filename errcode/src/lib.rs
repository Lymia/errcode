#![no_std]
extern crate alloc;

mod error;
mod error_code;
mod error_impl;
mod macros;

pub use error::{Error, ErrorInfo};
pub use error_code::ErrorCode;

/// NOT PUBLIC API!
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::error::new_error_info;
    pub use crate::error_code::ErrorCodePrivate;
    pub use crate::error_impl::{DecodedLocation, ErrorSourceStatic};
    pub use core;
    pub use core::option::Option::{None, Some};
}
