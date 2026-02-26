//! Contains the raw implementation of the error code API.

use std::any::TypeId;
use std::fmt::Debug;

/// Represents the info underlying an error code.
#[non_exhaustive]
pub struct ErrorCodeInfo {
    /// The type ID of this error code.
    pub tid: TypeId,

    /// The value of this error code.
    pub value: u32,

    /// The name of this error code.
    pub name: &'static str,

    /// The message this error code should be translated to.
    pub message: &'static str,
}

/// Types that can be used as error codes.
pub trait ErrorCode: Copy + Clone + Eq + Debug {
    fn info(self) -> ErrorCodeInfo;
    fn from_value(value: u32) -> Self;
}