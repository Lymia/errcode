#![no_std]
extern crate alloc;

mod error;
mod error_code;
mod error_impl;
mod macros;

pub use error::{Error, ErrorInfo};
pub use error_code::ErrorCode;
