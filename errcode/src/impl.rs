//! This module contains the internal guts of the error type.

use crate::errorcode::ErrorCodeInfo;
use core::fmt::Arguments;
use core::panic::Location;

/// Error type implementation for when `alloc` is enabled.
///
/// TODO: Document
#[cfg(feature = "repr_full")]
mod implementation {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    #[repr(transparent)]
    pub struct ErrorImpl {
        inner: Box<ErrorImplInner>,
    }
    struct ErrorImplInner {
        original: ErrorSourceStep,
        steps: Vec<ErrorSourceStep>,
    }
    impl ErrorImpl {
        #[track_caller]
        pub fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl {
            ErrorImpl {
                inner: Box::new(ErrorImplInner {
                    original: ErrorSourceStep {
                        static_info: source,
                        formatted_message: args.map(|x| x.to_string().into()),
                        location: Location::caller(),
                    },
                    steps: Vec::new(),
                }),
            }
        }

        #[track_caller]
        pub fn push_context(
            &mut self,
            source: &'static ErrorSourceStatic,
            args: Option<&Arguments<'_>>,
        ) {
            let step = ErrorSourceStep {
                static_info: ErrorOrigin::StaticOrigin(source),
                formatted_message: args.map(|x| x.to_string().into()),
                location: Location::caller(),
            };
        }
    }

    pub struct ErrorSourceStep {
        static_info: ErrorOrigin,
        formatted_message: Option<Cow<'static, str>>,
        location: &'static Location<'static>,
    }
}

/// Implementation for `repr_unboxed` and `repr_unboxed_location`.
///
/// TODO: Document
#[cfg(any(
    feature = "repr_unboxed",
    feature = "repr_unboxed_location",
    not(any(feature = "repr_full"))
))]
mod implementation {
    use super::*;
    use core::hint::unreachable_unchecked;
    use core::num::NonZeroUsize;

    pub struct ErrorImpl {
        origin_info: PackedOriginInfo,
        #[cfg(feature = "repr_unboxed_location")]
        original_location: &'static Location<'static>,
    }
    impl ErrorImpl {
        #[cfg_attr(feature = "repr_unboxed_location", track_caller)]
        pub fn new(source: ErrorOrigin, _args: Option<&Arguments<'_>>) -> ErrorImpl {
            ErrorImpl {
                origin_info: PackedOriginInfo::for_origin(source),
                #[cfg(feature = "repr_unboxed_location")]
                original_location: Location::caller(),
            }
        }

        pub fn push_context(
            &mut self,
            source: &'static ErrorSourceStatic,
            _args: Option<&Arguments<'_>>,
        ) {
            self.origin_info = self.origin_info.with_context(source);
        }
    }

    const TAG_STATIC_ORIGINAL: usize = 0;
    const TAG_STATIC_TYPE_ONLY: usize = 1;
    const TAG_STATIC_CONTEXT_ONLY: usize = 2;
    const TAG_MASK: usize = 0b11;

    const MAX_TYPE_LEN: usize = (usize::MAX >> 2) + 1;
    const OMITTED_BIT_MASK: usize = 0b1;

    #[derive(Copy, Clone)]
    struct PackedOriginInfo {
        /// A pointer to something that also stores a tag in the lower 2 bits. The fact that this
        /// is valid (due to alignment) is enforced by the const check in this module.
        ///
        /// This is nonzero to allow for niche optimization on the error type.
        ///
        /// For `TAG_STATIC_ORIGINAL` and `TAG_STATIC_CONTEXT_ONLY`, this is a pointer to an
        /// `ErrorSourceStatic`. It is enforced nonzero because pointers cannot be zero.
        ///
        /// For `TAG_STATIC_TYPE_ONLY`, this is the length of the type string, with the pointer
        /// itself stored in `additional`. It is enforced nonzero because the tag is nonzero.
        tag: NonZeroUsize,

        /// Additional tag information.
        ///
        /// For `TAG_STATIC_ORIGINAL` and `TAG_STATIC_CONTEXT_ONLY` this is a pointer to an
        /// `ErrorSourceStatic`, or zero (for no latest context). The lowest bit is used to store
        /// a flag for whether frames have been omitted from this context.
        ///
        /// For `TAG_STATIC_TYPE_ONLY`, this is the pointer to the string.
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
                    ErrorOrigin::TypeOrigin(ptr) => {
                        assert!(ptr.len() < MAX_TYPE_LEN);
                        PackedOriginInfo {
                            tag: NonZeroUsize::new_unchecked(ptr.len() | TAG_STATIC_TYPE_ONLY),
                            additional: ptr.as_ptr() as usize,
                        }
                    }
                }
            }
        }

        fn tag(&self) -> usize {
            self.tag.get() & TAG_MASK
        }

        fn with_context(mut self, source: &'static ErrorSourceStatic) -> Self {
            unsafe {
                match self.tag() {
                    TAG_STATIC_ORIGINAL | TAG_STATIC_CONTEXT_ONLY => {
                        if self.additional == 0 {
                            self.additional = source as *const _ as usize;
                            self
                        } else {
                            let original = &*(self.additional as *const ErrorSourceStatic);
                            if original.error_code.is_none() || source.error_code.is_some() {
                                self.additional = source as *const _ as usize;
                                self.additional |= OMITTED_BIT_MASK;
                                self
                            } else {
                                self.additional |= OMITTED_BIT_MASK;
                                self
                            }
                        }
                    }
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

        fn has_omitted_context(self) -> bool {
            if self.tag() == TAG_STATIC_CONTEXT_ONLY || self.tag() == TAG_STATIC_ORIGINAL {
                self.additional & OMITTED_BIT_MASK == OMITTED_BIT_MASK
            } else {
                false
            }
        }
    }

    const _CHECK_REQUIRED_ALIGNMENT: () = {
        let required_alignment = 4;
        assert!(align_of::<&'static ErrorSourceStatic>() >= required_alignment);
        assert!(align_of::<&'static str>() >= required_alignment);
    };
}

pub use implementation::ErrorImpl;

pub struct ErrorSourceStatic {
    pub error_code: Option<&'static ErrorCodeInfo>,
    pub message_static: Option<&'static str>,
    pub is_static_message_incomplete: bool,
    pub location: Option<&'static DecodedLocation>,
}

pub struct DecodedLocation {
    pub module: &'static str,
    pub line: u32,
    pub column: u32,
}

pub enum ErrorOrigin {
    StaticOrigin(&'static ErrorSourceStatic),
    TypeOrigin(&'static str),
}

/// A decoded frame of error information, retrieved from an [`ErrorImpl`].
pub struct ErrorFrame {
    pub data: ErrorFrameData,
    pub location: Option<&'static DecodedLocation>,
}

/// The data represented by an error frame.
pub enum ErrorFrameData {
    /// Used to represent a frame of context that doesn't "really" exist, but should be reported
    /// to the user anyway.
    InternalContext(InternalContextType),

    /// Used to represent a frame where the only information known is the type of a converted
    /// error.
    TypeFrame(&'static str),

    /// Used to represent a static message given by the user.
    StaticMessage(&'static str),

    /// Used to represent a static message that couldn't be formatted.
    IncompleteStaticMessage(&'static str),

    /// Used to represent a formatted message given by the user.
    #[cfg(feature = "repr_unboxed")]
    UserMessage(alloc::str::String),
}

pub enum InternalContextType {
    /// Used to represent when an error type is constructed at a significantly different location
    /// from the `Location` stored in the error type.
    ///
    /// This often implies that `#[track_caller]` was used, though it could also just be a code
    /// style that broke the `error!` macro call into a different line.
    ErrorTypeConstructed,

    /// Used to represent when the original type the error type was converted from was lost. This
    /// occurs on the compact representation used when `alloc` isn't set fairly often.
    OriginalTypeLost,

    /// Used to note to the user that additional frames of context may have been omitted from the
    /// trace. This occurs on the compact representation used when `alloc` isn't set.
    FurtherFramesOmitted,
}
