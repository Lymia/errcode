//! Contains the raw implementation of the error code API.

use crate::r#impl::ErrorSourceStatic;
use core::any::TypeId;

/// Represents the info underlying an error code.
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

/// A type that can be used as an error code for this crate.
pub trait ErrorCode: 'static + Copy + Eq + ErrorCodePrivate {}

/// The internal error code trait implementation.
pub trait ErrorCodePrivate: 'static + Copy {
    /// Returns the internal error info code for this type.
    fn info(self) -> &'static ErrorCodeInfo;

    /// Returns the internal error info code for this type.
    fn error_source(self) -> &'static ErrorSourceStatic;

    /// Returns an enum value corresponding to this error code.
    ///
    /// This should *panic* if the value does not correspond to a known enum variant.
    fn from_value(value: u32) -> Self;
}
