/// Creates a new [`ErrorInfo`].
///
/// TODO: Document
#[macro_export]
macro_rules! error_info {
    ($format:literal $(,)?) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::None,
                message_static: $crate::__macro_export::Some($format),
                is_static_message_incomplete: false,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::module_path!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::None,
        )
    };
    ($format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::None,
                message_static: $crate::__macro_export::Some($format),
                is_static_message_incomplete: true,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::module_path!(),
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
                message_static: None,
                is_static_message_incomplete: false,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::module_path!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::None,
        )
    };
    ($code:path, $format:literal $(,)?) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::Some(
                    $crate::__macro_export::ErrorCodePrivate::info($code),
                ),
                message_static: $crate::__macro_export::Some($format),
                is_static_message_incomplete: false,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::module_path!(),
                        line: $crate::__macro_export::core::line!(),
                        column: $crate::__macro_export::core::column!(),
                    },
                ),
            },
            $crate::__macro_export::None,
        )
    };
    ($code:path, $format:literal, $($arguments:tt)*) => {
        $crate::__macro_export::new_error_info(
            &$crate::__macro_export::ErrorSourceStatic {
                error_code: $crate::__macro_export::Some(
                    $crate::__macro_export::ErrorCodePrivate::info($code),
                ),
                message_static: $crate::__macro_export::Some($format),
                is_static_message_incomplete: true,
                location: $crate::__macro_export::Some(
                    &$crate::__macro_export::DecodedLocation {
                        module: $crate::__macro_export::core::module_path!(),
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
