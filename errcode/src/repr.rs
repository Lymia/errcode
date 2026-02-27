use crate::errorcode::ErrorCodeInfo;
use core::fmt::Arguments;
use core::panic::Location;

#[cfg(feature = "alloc")]
mod implementation {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    #[repr(transparent)]
    pub struct ErrorImpl {
        inner: Box<ErrorSource>,
    }
    impl ErrorImpl {
        pub fn wrap(source: ErrorSource) -> Self {
            ErrorImpl { inner: Box::new(source) }
        }
    }

    pub struct ErrorSource {
        original: ErrorSourceStep,
        steps: Vec<ErrorSourceStep>,
    }
    impl ErrorSource {
        #[track_caller]
        pub fn new(source: &'static ErrorSourceStatic, args: &Arguments<'_>) -> ErrorSource {
            ErrorSource {
                original: ErrorSourceStep {
                    static_info: source,
                    formatted_message: Some(args.to_string().into()),
                    location: Location::caller(),
                },
                steps: Vec::new(),
            }
        }

        pub fn push_step(&mut self, step: ErrorSourceStep, args: &Arguments<'_>) {
            let step = ErrorSourceStep {
                static_info: &ErrorSourceStatic {},
                formatted_message: None,
                location: &(),
            };
        }
    }

    pub struct ErrorSourceStep {
        static_info: &'static ErrorSourceStatic,
        formatted_message: Option<Cow<'static, str>>,
        location: &'static Location<'static>,
    }
}

#[cfg(not(feature = "alloc"))]
mod implementation {
    use core::hint::unreachable_unchecked;
    use super::*;
    use core::num::NonZeroUsize;

    pub struct ErrorSource {
        origin_info: PackedOriginInfo,
        #[cfg(feature = "location_no_alloc")]
        original_location: &'static Location<'static>,
    }
    impl ErrorSource {
        #[track_caller]
        pub fn new(source: ErrorOrigin, _args: &Arguments<'_>) -> ErrorSource {
            ErrorSource {
                origin_info: PackedOriginInfo::for_origin(source),
                #[cfg(feature = "location_no_alloc")]
                original_location: Location::caller(),
            }
        }

        pub fn push_context(&mut self, source: &'static ErrorSourceStatic, _args: &Arguments<'_>) {
            self.origin_info = self.origin_info.with_context(source);
        }
    }

    const TAG_STATIC_ORIGINAL: usize = 0;
    const TAG_STATIC_TYPE_ONLY: usize = 1;
    const TAG_STATIC_CONTEXT_ONLY: usize = 2;
    const TAG_MASK: usize = 0b11;

    #[derive(Copy, Clone)]
    struct PackedOriginInfo {
        tag: NonZeroUsize, // non-zero to allow a niche for option optimization.
        additional: usize,
    }
    impl PackedOriginInfo {
        fn for_origin(e: ErrorOrigin) -> Self {
            unsafe {
                match e {
                    ErrorOrigin::StaticOrigin(ptr) => PackedOriginInfo {
                        tag: NonZeroUsize::new_unchecked(
                            (ptr as *const _ as usize) | TAG_STATIC_ORIGINAL,
                        ),
                        additional: 0,
                    },
                    ErrorOrigin::TypeOrigin(ptr) => PackedOriginInfo {
                        tag: NonZeroUsize::new_unchecked(
                            (ptr.as_ptr() as usize) | TAG_STATIC_TYPE_ONLY,
                        ),
                        additional: ptr.len(),
                    },
                }
            }
        }

        fn tag(&self) -> usize {
            self.tag.get() & TAG_MASK
        }

        fn with_context(mut self, source: &'static ErrorSourceStatic) -> Self {
            unsafe {
                match self.tag() {
                    TAG_STATIC_ORIGINAL | TAG_STATIC_CONTEXT_ONLY => if self.additional == 0 {
                        self.additional = source as *const _ as usize;
                        self
                    } else {
                        let original = &*(self.additional as *const ErrorSourceStatic);
                        if original.error_code.is_none() || source.error_code.is_some() {
                            self.additional = source as *const _ as usize;
                            self
                        } else {
                            self
                        }
                    },
                    TAG_STATIC_TYPE_ONLY => PackedOriginInfo {
                        tag: NonZeroUsize::new_unchecked(
                            (source as *const _ as usize) | TAG_STATIC_CONTEXT_ONLY,
                        ),
                        additional: 0,
                    },
                    _ => unreachable_unchecked(),
                }
            }
        }
    }

    const _CHECK_REQUIRED_ALIGNMENT: () = {
        let required_alignment = 4;
        assert!(align_of::<&'static ErrorSourceStatic>() >= required_alignment);
        assert!(align_of::<&'static str>() >= required_alignment);
    };
}

pub struct ErrorSourceStatic {
    pub error_code: Option<&'static ErrorCodeInfo>,
    pub message_static: Option<&'static str>,
    pub location: Option<&'static MacroLocation>,
}

pub struct MacroLocation {
    pub module: &'static str,
    pub line: u32,
    pub column: u32,
}

pub enum ErrorOrigin {
    StaticOrigin(&'static ErrorSourceStatic),
    TypeOrigin(&'static str),
}
