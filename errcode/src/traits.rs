use crate::{Error, ErrorCode, ErrorInfo};
use core::any::{Any, type_name};

pub trait IntoErrorHelper {
    type OutputType;
    fn convert(self, info: ErrorInfo) -> Result<Self::OutputType, Error>;
    fn convert_code<C: ErrorCode>(self, code: C) -> Result<Self::OutputType, Error>;
}

pub trait ConvertErrorHelper {
    fn with_context(self, info: ErrorInfo) -> Self;
    fn with_context_code<C: ErrorCode>(self, code: C) -> Self;
}

impl<T> IntoErrorHelper for Option<T> {
    type OutputType = T;

    #[inline(always)]
    #[track_caller]
    fn convert(self, info: ErrorInfo) -> Result<Self::OutputType, Error> {
        match self {
            None => Err(Error::from_info(info)),
            Some(v) => Ok(v),
        }
    }

    #[inline(always)]
    #[track_caller]
    fn convert_code<C: ErrorCode>(self, code: C) -> Result<Self::OutputType, Error> {
        match self {
            None => Err(Error::from_code(code)),
            Some(v) => Ok(v),
        }
    }
}

impl<T, E: Any> IntoErrorHelper for Result<T, E> {
    type OutputType = T;

    #[inline(always)]
    #[track_caller]
    fn convert(self, info: ErrorInfo) -> Result<Self::OutputType, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(name_and_info(type_name::<E>(), info)),
        }
    }

    #[inline(always)]
    #[track_caller]
    fn convert_code<C: ErrorCode>(self, code: C) -> Result<Self::OutputType, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::from_type_with_code(type_name::<E>(), code)),
        }
    }
}

#[inline(never)]
fn name_and_info(name: &'static str, info: ErrorInfo) -> Error {
    Error::from_type(name).with_context(info)
}

impl<T> ConvertErrorHelper for Result<T, Error> {
    #[inline(always)]
    #[track_caller]
    fn with_context(self, info: ErrorInfo) -> Self {
        self.map_err(|x| x.with_context(info))
    }

    #[inline(always)]
    #[track_caller]
    fn with_context_code<C: ErrorCode>(self, code: C) -> Self {
        self.map_err(|x| x.with_context_code(code))
    }
}
