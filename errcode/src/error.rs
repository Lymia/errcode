use crate::error_code::ErrorCode;
use crate::error_impl::{ErrorImpl, ErrorImplFunctions, ErrorOrigin, ErrorSourceStatic};
use core::any::{TypeId, type_name};
use core::fmt::{Arguments, Display, Formatter};

#[derive(Clone)]
#[repr(transparent)]
pub struct Error {
    underlying: ErrorImpl,
}
impl Error {
    #[inline(never)]
    #[track_caller]
    pub fn from_info(info: ErrorInfo) -> Self {
        Error {
            underlying: ErrorImpl::new(
                ErrorOrigin::StaticOrigin(info.info),
                info.arguments.as_ref(),
            ),
        }
    }

    #[inline(never)]
    #[track_caller]
    pub fn from_code<T: ErrorCode>(code: T) -> Self {
        Error {
            underlying: ErrorImpl::new(ErrorOrigin::StaticOrigin(T::error_source(code)), None),
        }
    }

    /// Returns whether this error has an error code.
    #[inline(always)]
    pub fn has_code(&self) -> bool {
        self.underlying.code().is_some()
    }

    /// Returns whether this error has a given error code.
    #[inline(always)]
    pub fn is<T: ErrorCode>(&self, value: T) -> bool {
        if let Some(code) = self.underlying.code() {
            code.is_value(value)
        } else {
            false
        }
    }

    /// Returns whether this error has an error code of the given type.
    #[inline(always)]
    pub fn is_type<T: ErrorCode>(&self) -> bool {
        if let Some(code) = self.underlying.code() {
            code.tid == TypeId::of::<T>()
        } else {
            false
        }
    }

    /// Downcasts the error code to a given type if possible.
    #[inline(always)]
    pub fn downcast_code<T: ErrorCode>(&self) -> Option<T> {
        if let Some(code) = self.underlying.code() {
            if code.tid == TypeId::of::<T>() {
                Some(T::from_value(code.value))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Adds a new context frame to this error type.
    #[inline(never)]
    #[track_caller]
    pub fn with_context(
        mut self,
        info: &'static ErrorInfo,
        format: Option<&Arguments<'_>>,
    ) -> Self {
        self.underlying.push_context(&info.info, format);
        self
    }
}
impl<T: core::error::Error> From<T> for Error {
    #[inline(never)]
    #[track_caller]
    fn from(value: T) -> Self {
        let code = error_code_for_error(&value);
        Error {
            underlying: ErrorImpl::new(
                ErrorOrigin::TypeOrigin(type_name::<T>(), code),
                Some(&format_args!("{value}")),
            ),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Error trace:")?;
        for frame in self.underlying.iter() {
            write!(f, "\n  {frame}")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct ErrorInfo<'a> {
    info: &'static ErrorSourceStatic,
    arguments: Option<Arguments<'a>>,
}

#[inline(never)]
fn error_code_for_error<T>(_value: &T) -> Option<&'static ErrorSourceStatic> {
    None
}

pub const fn new_error_info<'a>(
    info: &'static ErrorSourceStatic,
    arguments: Option<Arguments<'a>>,
) -> ErrorInfo<'a> {
    ErrorInfo { info, arguments }
}
