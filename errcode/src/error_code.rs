//! Contains the raw implementation of the error code API.

use crate::error_impl::ErrorInfoImpl;
use core::any::TypeId;
use core::fmt::{Debug, Formatter};

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
impl Debug for ErrorCodeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ErrorCodeInfo")
            .field("variant", &format_args!("{}::{}", self.type_name, self.value))
            .field("message", &self.message)
            .finish()
    }
}

/// A type that can be used as an error code for this crate.
pub trait ErrorCode: 'static + ErrorCodePrivate {}

/// The internal error code trait implementation.
pub trait ErrorCodePrivate: 'static {
    /// Helper type for constant time operations.
    ///
    /// contains: `const fn info(self) -> &'static ErrorCodeInfo;`
    type ConstHelper;

    /// The instance of `ConstHelper`.
    const CONST_HELPER_INSTANCE: Self::ConstHelper;

    /// Returns the internal error info code for this type.
    fn error_source(self) -> &'static ErrorInfoImpl;

    /// Returns true if the value matches this enum.
    fn is_value(self, value: u32) -> bool;

    /// Returns an enum value corresponding to this error code.
    ///
    /// This should *panic* if the value does not correspond to a known enum variant.
    fn from_value(value: u32) -> Self;
}
