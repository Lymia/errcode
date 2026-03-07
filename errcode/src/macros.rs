use crate::error_impl::StaticMessageInfo;

/// Creates a new [`ErrorInfo`].
///
/// TODO: Document
#[macro_export]
macro_rules! error_info {
    ($format:literal) => {
        $crate::error_info!($format,)
    };
    ($format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
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
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::Some(
                    $crate::__macro_export::ErrorCodePrivate::info($code),
                ),
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
        $crate::error_code!($code, $format,)
    };
    ($code:path, $format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::Some(
                    $crate::__macro_export::ErrorCodePrivate::info($code),
                ),
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

pub const fn is_argument_str(string: &'static str) -> bool {
    let bytes = string.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'{' && idx != bytes.len() - 1 {
            if bytes[idx] + 1 == b'{' {
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
