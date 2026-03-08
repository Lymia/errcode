use crate::error_code::ErrorCode;
use crate::error_impl::{
    ErrorFrameImpl, ErrorImpl, ErrorImplFunctions, ErrorInfoImpl, ErrorOrigin,
};
use core::any::{TypeId, type_name};
use core::fmt::{Arguments, Debug, Display, Formatter};

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

    #[inline(never)]
    #[track_caller]
    pub fn from_type(name: &'static str) -> Self {
        Error { underlying: ErrorImpl::new(ErrorOrigin::TypeOrigin(name, None), None) }
    }

    #[inline(never)]
    #[track_caller]
    pub fn from_type_with_code<T: ErrorCode>(name: &'static str, code: T) -> Self {
        Error {
            underlying: ErrorImpl::new(
                ErrorOrigin::TypeOrigin(name, Some(T::error_source(code))),
                None,
            ),
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
    pub fn with_context(mut self, info: ErrorInfo) -> Self {
        self.underlying
            .push_context(&info.info, info.arguments.as_ref());
        self
    }

    /// Adds a new context frame to this error type.
    #[inline(never)]
    #[track_caller]
    pub fn with_context_code<T: ErrorCode>(mut self, info: T) -> Self {
        self.underlying.push_context(T::error_source(info), None);
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
impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for frame in self.underlying.iter() {
            list.entry(&format_args!("{:?}", frame));
        }
        list.finish()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut iter = self.underlying.iter();
        if let Some(frame) = iter.next() {
            write!(f, "{frame}")?;
        }
        for frame in iter {
            write!(f, "\n    caused by: {frame}")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ErrorFrame {
    inner: ErrorFrameImpl,
}
impl Debug for ErrorFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
impl Display for ErrorFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

pub struct ErrorFrameIter<'a> {
    iter: <ErrorImpl as ErrorImplFunctions>::FrameIter<'a>,
}
impl Iterator for ErrorFrameIter<'_> {
    type Item = ErrorFrame;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| ErrorFrame { inner: x })
    }
}

#[derive(Copy, Clone)]
pub struct ErrorInfo<'a> {
    info: &'static ErrorInfoImpl,
    arguments: Option<Arguments<'a>>,
}

#[inline(never)]
fn error_code_for_error<T>(_value: &T) -> Option<&'static ErrorInfoImpl> {
    None
}

pub const fn new_error_info<'a>(
    info: &'static ErrorInfoImpl,
    arguments: Option<Arguments<'a>>,
) -> ErrorInfo<'a> {
    ErrorInfo { info, arguments }
}
