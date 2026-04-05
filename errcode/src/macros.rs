use crate::error_code::{ErrorCodeInfo, ErrorCodePrivate};
use crate::error_impl::{ErrorInfoImpl, StaticMessageInfo};

#[cfg(doc)]
use crate::{Error, ErrorInfo};

/// Creates a new [`ErrorInfo`].
///
/// TODO: Document
#[macro_export]
macro_rules! error_info {
    () => {
        $crate::error_info!("error encountered")
    };
    ($format:literal) => {
        $crate::error_info!($format,)
    };
    ($format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorInfoImpl {
                error_code: $crate::__macro_export::None,
                message_static: const {
                    $crate::__macro_export::static_message(
                        $format,
                        $crate::__macro_export::core::stringify!($format),
                    )
                },
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::file!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::Some(
                $crate::__macro_export::core::format_args!($format, $($arguments)*),
            ),
        )
    };
    ($code:path $(,)?) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorInfoImpl {
                error_code: const {
                    $crate::__macro_export::Some(
                        $crate::__macro_export::get_helper(&$code).info($code),
                    )
                },
                message_static: $crate::__macro_export::StaticMessageInfo::None,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::file!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::None,
        )
    };
    ($code:path, $format:literal) => {
        $crate::error_info!($code, $format,)
    };
    ($code:path, $format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorInfoImpl {
                error_code: const {
                    $crate::__macro_export::Some(
                        $crate::__macro_export::get_helper(&$code).info($code),
                    )
                },
                message_static: const {
                    $crate::__macro_export::static_message(
                        $format,
                        $crate::__macro_export::core::stringify!($format),
                    )
                },
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::file!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::Some(
                $crate::__macro_export::core::format_args!($format, $($arguments)*),
            ),
        )
    };
}

const fn is_argument_str(string: &'static str) -> bool {
    let bytes = string.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'{' && idx != bytes.len() - 1 {
            if bytes[idx + 1] == b'{' {
                idx += 1;
            } else {
                return true;
            }
        }
        idx += 1;
    }
    false
}

pub const fn static_message(format: &'static str, stringified: &'static str) -> StaticMessageInfo {
    if is_argument_str(format) {
        StaticMessageInfo::Unformatted(stringified)
    } else {
        StaticMessageInfo::NoFormat(format)
    }
}

pub const fn get_helper<T: ErrorCodePrivate>(_t: &T) -> T::ConstHelper {
    T::CONST_HELPER_INSTANCE
}

pub const fn wrap_code(code: &'static ErrorCodeInfo) -> ErrorInfoImpl {
    ErrorInfoImpl {
        error_code: Some(code),
        message_static: StaticMessageInfo::None,
        location: None,
    }
}

/// Constructs a new [`Error`].
///
/// This uses the same syntax as [`error_info!`].
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::Error::from_info($crate::error_info!($($args)*))
    }
}

/// Returns from the function with a newly constructed [`Error`].
///
/// This uses the same syntax as [`error_info!`]. The error is immediately wrapped in an
/// [`Err`] and returned with the `?` operator.
#[macro_export]
macro_rules! bail {
    ($($args:tt)*) => {{
        $crate::__macro_export::core::result::Result::Err($crate::error!($($args)*))?;
        $crate::__macro_export::core::unreachable!()
    }};
}

/// Returns an [`Error`] from the function if a condition is true.
///
/// This uses the same syntax as [`error_info!`]. The error is immediately wrapped in an
/// [`Err`] and returned with the `?` operator if the condition isn't met.
#[macro_export]
macro_rules! ensure {
    ($condition:expr) => {
        if !$condition {
            $crate::bail!();
        }
    };
    ($condition:expr, $($args:tt)*) => {
        if !$condition {
            $crate::bail!($($args)*);
        }
    };
}
