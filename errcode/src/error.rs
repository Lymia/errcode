use crate::error_code::ErrorCode;
use crate::error_impl::{ErrorImpl, ErrorImplFunctions, ErrorOrigin, ErrorSourceStatic};
use core::any::type_name;

#[derive(Clone)]
#[repr(transparent)]
pub struct Error {
    underlying: ErrorImpl,
}
impl Error {
    /// Returns whether this error has a given error code.
    pub fn is_code<T: ErrorCode>(&self, value: T) -> bool {
        if let Some(code) = self.underlying.code() {
            code.is_value(value)
        } else {
            false
        }
    }
}
impl<T: core::error::Error> From<T> for Error {
    #[inline(never)]
    #[track_caller]
    fn from(value: T) -> Self {
        Error {
            underlying: ErrorImpl::new(
                ErrorOrigin::TypeOrigin(type_name::<T>(), error_code_for_error(value)),
                None,
            ),
        }
    }
}

#[inline(never)]
fn error_code_for_error<T>(value: T) -> Option<&'static ErrorSourceStatic> {
    None
}
