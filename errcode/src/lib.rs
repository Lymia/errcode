#![no_std]
extern crate alloc;

mod error;
mod error_code;
mod error_impl;

pub use error::Error;
pub use error_code::ErrorCode;
