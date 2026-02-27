//! Contains the raw implementation of the error code API.

use crate::error_impl::ErrorSourceStatic;
use core::any::TypeId;

/// Represents the info underlying an error code.
pub struct ErrorCodeInfo {
    /// The type ID of this error code.
    pub tid: TypeId,

    /// The value of this error code.
    pub value: u32,

    /// The name of the type underlying this error code.
    pub type_name: &'static str,

    /// The name of this error code.
    pub variant_name: &'static str,

    /// The message this error code should be translated to.
    pub message: Option<&'static str>,
}
impl ErrorCodeInfo {
    pub fn is_value<T: ErrorCodePrivate>(&self, val: T) -> bool {
        self.tid == TypeId::of::<T>() && val.is_value(self.value)
    }

    pub fn decode_value<T: ErrorCodePrivate>(&self) -> Option<T> {
        if self.tid == TypeId::of::<T>() {
            Some(T::from_value(self.value))
        } else {
            None
        }
    }
}

/// A type that can be used as an error code for this crate.
pub trait ErrorCode: 'static + Copy + Eq + ErrorCodePrivate {}

/// The internal error code trait implementation.
pub trait ErrorCodePrivate: 'static + Copy {
    /// Returns the internal error info code for this type.
    fn info(self) -> &'static ErrorCodeInfo;

    /// Returns the internal error info code for this type.
    fn error_source(self) -> &'static ErrorSourceStatic;

    /// Returns true if the value matches this enum.
    fn is_value(self, value: u32) -> bool;

    /// Returns an enum value corresponding to this error code.
    ///
    /// This should *panic* if the value does not correspond to a known enum variant.
    fn from_value(value: u32) -> Self;
}
